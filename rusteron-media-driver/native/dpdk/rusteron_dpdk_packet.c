/*
 * rusteron-media-driver DPDK transport — frame encoding (plan §7.4).
 *
 * Pure encoders: no libdpdk, no port ops, no allocation. Every address and
 * length handled here is either already network order (IPv4 addresses) or
 * written byte-wise into the frame so endianness never matters.
 */
#include "rusteron_dpdk_packet.h"

#include <arpa/inet.h>
#include <string.h>

uint32_t rusteron_dpdk_ipv4_mask(uint8_t prefix_len)
{
    /* htonl of the host mask yields the same bits in network order, so a
     * byte-wise AND with a network-order address extracts the subnet. */
    return htonl(0xFFFFFFFFu << (32 - prefix_len));
}

int rusteron_dpdk_ipv4_is_multicast(uint32_t ip)
{
    /* The first network-order byte holds the class; 224/8..239/8 is multicast. */
    const uint8_t *b = (const uint8_t *)&ip;
    return (b[0] & 0xF0) == 0xE0;
}

/* One's-complement sum of 16-bit big-endian words (DPDK rte_raw_cksum form),
 * folded but WITHOUT the final complement: the pseudo-header seed DPDK's TX
 * checksum offload expects in the UDP checksum field. */
static uint16_t rusteron_dpdk_udp_pseudo_csum(uint32_t src_ip, uint32_t dst_ip, uint16_t udp_len)
{
    const uint8_t *s = (const uint8_t *)&src_ip;
    const uint8_t *d = (const uint8_t *)&dst_ip;
    uint32_t sum = 0;

    for (int i = 0; i < 4; i += 2)
    {
        sum += (uint16_t)((s[i] << 8) | s[i + 1]);
        sum += (uint16_t)((d[i] << 8) | d[i + 1]);
    }
    sum += RUSTERON_DPDK_IP_PROTO_UDP;
    sum += udp_len;

    while (sum >> 16)
    {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    return (uint16_t)sum;
}

static size_t rusteron_dpdk_iov_total(const struct iovec *iov, size_t iov_count)
{
    size_t total = 0;
    for (size_t i = 0; i < iov_count; i++)
    {
        total += iov[i].iov_len;
    }
    return total;
}

static void rusteron_dpdk_write_eth(
    uint8_t *dst,
    const uint8_t dst_mac[RUSTERON_DPDK_ETH_ADDR_LEN],
    const uint8_t src_mac[RUSTERON_DPDK_ETH_ADDR_LEN],
    uint16_t ethertype)
{
    memcpy(dst, dst_mac, RUSTERON_DPDK_ETH_ADDR_LEN);
    memcpy(dst + RUSTERON_DPDK_ETH_ADDR_LEN, src_mac, RUSTERON_DPDK_ETH_ADDR_LEN);
    dst[12] = (uint8_t)(ethertype >> 8);
    dst[13] = (uint8_t)(ethertype & 0xFF);
}

/* IPv4 header with DF set, checksum field zeroed for hardware offload. The
 * identification field is left zero; the NIC does not require it. */
static void rusteron_dpdk_write_ipv4(
    uint8_t *dst, uint32_t dst_ip, uint32_t src_ip, uint16_t total_len, uint8_t ttl)
{
    dst[0] = 0x45;                                  /* IPv4, 20-byte header */
    dst[1] = 0;                                     /* DSCP/ECN */
    dst[2] = (uint8_t)(total_len >> 8);
    dst[3] = (uint8_t)(total_len & 0xFF);
    dst[4] = 0;                                     /* identification (unused) */
    dst[5] = 0;
    dst[6] = 0x40;                                  /* DF, no fragmentation */
    dst[7] = 0;
    dst[8] = ttl;
    dst[9] = RUSTERON_DPDK_IP_PROTO_UDP;
    dst[10] = 0;                                    /* checksum: hw offload */
    dst[11] = 0;
    memcpy(dst + 12, &src_ip, 4);
    memcpy(dst + 16, &dst_ip, 4);
}

static void rusteron_dpdk_write_udp(
    uint8_t *dst, uint16_t src_port, uint16_t dst_port, uint16_t udp_len, uint16_t pseudo_csum)
{
    dst[0] = (uint8_t)(src_port >> 8);
    dst[1] = (uint8_t)(src_port & 0xFF);
    dst[2] = (uint8_t)(dst_port >> 8);
    dst[3] = (uint8_t)(dst_port & 0xFF);
    dst[4] = (uint8_t)(udp_len >> 8);
    dst[5] = (uint8_t)(udp_len & 0xFF);
    dst[6] = (uint8_t)(pseudo_csum >> 8); /* seed for hw offload, not final */
    dst[7] = (uint8_t)(pseudo_csum & 0xFF);
}

int rusteron_dpdk_packet_build_udp(
    rusteron_dpdk_mbuf_t *m,
    const uint8_t dst_mac[RUSTERON_DPDK_ETH_ADDR_LEN],
    const uint8_t src_mac[RUSTERON_DPDK_ETH_ADDR_LEN],
    uint32_t dst_ip, uint32_t src_ip,
    uint16_t src_port, uint16_t dst_port,
    const struct iovec *iov, size_t iov_count,
    size_t max_payload)
{
    if (NULL == m || NULL == m->data)
    {
        return -1;
    }

    size_t payload_len = rusteron_dpdk_iov_total(iov, iov_count);
    if (payload_len > max_payload)
    {
        return -1;
    }
    size_t frame_len = RUSTERON_DPDK_FRAME_HDR_LEN + payload_len;
    if (frame_len > m->capacity)
    {
        return -1;
    }

    uint8_t *p = m->data;
    rusteron_dpdk_write_eth(p, dst_mac, src_mac, RUSTERON_DPDK_ETH_TYPE_IPV4);
    rusteron_dpdk_write_ipv4(p + RUSTERON_DPDK_ETH_HDR_LEN, dst_ip, src_ip, (uint16_t)frame_len - RUSTERON_DPDK_ETH_HDR_LEN, 64);

    uint16_t udp_len = RUSTERON_DPDK_UDP_HDR_LEN + (uint16_t)payload_len;
    uint16_t pseudo_csum = rusteron_dpdk_udp_pseudo_csum(src_ip, dst_ip, udp_len);
    rusteron_dpdk_write_udp(p + RUSTERON_DPDK_ETH_HDR_LEN + RUSTERON_DPDK_IPV4_HDR_LEN, src_port, dst_port, udp_len, pseudo_csum);

    uint8_t *payload = p + RUSTERON_DPDK_FRAME_HDR_LEN;
    for (size_t i = 0; i < iov_count; i++)
    {
        memcpy(payload, iov[i].iov_base, iov[i].iov_len);
        payload += iov[i].iov_len;
    }

    m->frame_len = (uint32_t)frame_len;
    m->ol_flags = RUSTERON_DPDK_MBUF_F_TX_IPV4 | RUSTERON_DPDK_MBUF_F_TX_IP_CKSUM | RUSTERON_DPDK_MBUF_F_TX_UDP_CKSUM;
    m->l2_len = RUSTERON_DPDK_ETH_HDR_LEN;
    m->l3_len = RUSTERON_DPDK_IPV4_HDR_LEN;
    m->l4_len = RUSTERON_DPDK_UDP_HDR_LEN;
    m->udp_pseudo_csum = pseudo_csum;

    return 0;
}

int rusteron_dpdk_packet_build_arp_request(
    rusteron_dpdk_mbuf_t *m,
    const uint8_t src_mac[RUSTERON_DPDK_ETH_ADDR_LEN],
    uint32_t src_ip, uint32_t target_ip)
{
    if (NULL == m || NULL == m->data)
    {
        return -1;
    }
    const size_t frame_len = RUSTERON_DPDK_ETH_HDR_LEN + 28;
    if (frame_len > m->capacity)
    {
        return -1;
    }

    uint8_t broadcast[RUSTERON_DPDK_ETH_ADDR_LEN] = { 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF };
    uint8_t zero[RUSTERON_DPDK_ETH_ADDR_LEN] = { 0, 0, 0, 0, 0, 0 };

    uint8_t *p = m->data;
    rusteron_dpdk_write_eth(p, broadcast, src_mac, RUSTERON_DPDK_ETH_TYPE_ARP);
    uint8_t *a = p + RUSTERON_DPDK_ETH_HDR_LEN;
    a[0] = 0; a[1] = RUSTERON_DPDK_ARP_HTYPE_ETHERNET;
    a[2] = 0x08; a[3] = 0x00;                       /* 0x0800 */
    a[4] = RUSTERON_DPDK_ARP_HLEN;
    a[5] = RUSTERON_DPDK_ARP_PLEN;
    a[6] = 0; a[7] = RUSTERON_DPDK_ARP_OPER_REQUEST;
    memcpy(a + 8, src_mac, RUSTERON_DPDK_ETH_ADDR_LEN);   /* sender hw */
    memcpy(a + 14, &src_ip, 4);                            /* sender proto */
    memcpy(a + 18, zero, RUSTERON_DPDK_ETH_ADDR_LEN);      /* target hw */
    memcpy(a + 24, &target_ip, 4);                         /* target proto */

    m->frame_len = (uint32_t)frame_len;
    m->ol_flags = 0;
    m->l2_len = RUSTERON_DPDK_ETH_HDR_LEN;
    m->l3_len = 0;
    m->l4_len = 0;
    m->udp_pseudo_csum = 0;

    return 0;
}

int rusteron_dpdk_packet_build_arp_reply(
    rusteron_dpdk_mbuf_t *m,
    const uint8_t dst_mac[RUSTERON_DPDK_ETH_ADDR_LEN],
    const uint8_t src_mac[RUSTERON_DPDK_ETH_ADDR_LEN],
    uint32_t sender_ip, uint32_t target_ip)
{
    if (NULL == m || NULL == m->data)
    {
        return -1;
    }
    const size_t frame_len = RUSTERON_DPDK_ETH_HDR_LEN + 28;
    if (frame_len > m->capacity)
    {
        return -1;
    }

    uint8_t *p = m->data;
    rusteron_dpdk_write_eth(p, dst_mac, src_mac, RUSTERON_DPDK_ETH_TYPE_ARP);
    uint8_t *a = p + RUSTERON_DPDK_ETH_HDR_LEN;
    a[0] = 0; a[1] = RUSTERON_DPDK_ARP_HTYPE_ETHERNET;
    a[2] = 0x08; a[3] = 0x00;
    a[4] = RUSTERON_DPDK_ARP_HLEN;
    a[5] = RUSTERON_DPDK_ARP_PLEN;
    a[6] = 0; a[7] = RUSTERON_DPDK_ARP_OPER_REPLY;
    memcpy(a + 8, src_mac, RUSTERON_DPDK_ETH_ADDR_LEN);    /* sender hw (us) */
    memcpy(a + 14, &sender_ip, 4);                         /* sender proto (us) */
    memcpy(a + 18, dst_mac, RUSTERON_DPDK_ETH_ADDR_LEN);   /* target hw (requester) */
    memcpy(a + 24, &target_ip, 4);                         /* target proto (requester) */

    m->frame_len = (uint32_t)frame_len;
    m->ol_flags = 0;
    m->l2_len = RUSTERON_DPDK_ETH_HDR_LEN;
    m->l3_len = 0;
    m->l4_len = 0;
    m->udp_pseudo_csum = 0;

    return 0;
}
