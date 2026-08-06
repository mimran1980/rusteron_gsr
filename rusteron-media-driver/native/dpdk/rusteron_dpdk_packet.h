/*
 * rusteron-media-driver DPDK transport — frame encoding (plan §7.4).
 *
 * Builds Ethernet/IPv4/UDP frames and ARP frames into an opaque mbuf view so
 * the same encoders drive the real DPDK port ops and the test fake. The view
 * carries the checksum-offload metadata DPDK needs: L2/L3/L4 lengths, the
 * offload flags, and the UDP pseudo-header checksum seed (a raw one's-complement
 * sum, no final complement — that is what DPDK's TX checksum offload expects).
 *
 * This header is DPDK-free and included by the fake and by port ops.
 */
#ifndef RUSTERON_DPDK_PACKET_H
#define RUSTERON_DPDK_PACKET_H

#include <stddef.h>
#include <stdint.h>

#include <sys/uio.h> /* struct iovec */

#ifdef __cplusplus
extern "C" {
#endif

/* Frame layout constants (network byte order where multi-byte). */
#define RUSTERON_DPDK_ETH_ADDR_LEN 6
#define RUSTERON_DPDK_ETH_HDR_LEN 14
#define RUSTERON_DPDK_IPV4_HDR_LEN 20
#define RUSTERON_DPDK_UDP_HDR_LEN 8
#define RUSTERON_DPDK_FRAME_HDR_LEN \
    (RUSTERON_DPDK_ETH_HDR_LEN + RUSTERON_DPDK_IPV4_HDR_LEN + RUSTERON_DPDK_UDP_HDR_LEN)

#define RUSTERON_DPDK_ETH_TYPE_IPV4 0x0800
#define RUSTERON_DPDK_ETH_TYPE_ARP 0x0806
#define RUSTERON_DPDK_IP_PROTO_UDP 17
#define RUSTERON_DPDK_ARP_HTYPE_ETHERNET 1
#define RUSTERON_DPDK_ARP_PTYPE_IPV4 0x0800
#define RUSTERON_DPDK_ARP_HLEN 6
#define RUSTERON_DPDK_ARP_PLEN 4
#define RUSTERON_DPDK_ARP_OPER_REQUEST 1
#define RUSTERON_DPDK_ARP_OPER_REPLY 2

/* Offload flags carried on the mbuf view; the real port ops map them onto the
 * RTE_MBUF_F_TX_* bits. */
#define RUSTERON_DPDK_MBUF_F_TX_IPV4 (1u << 0)
#define RUSTERON_DPDK_MBUF_F_TX_IP_CKSUM (1u << 1)
#define RUSTERON_DPDK_MBUF_F_TX_UDP_CKSUM (1u << 2)

/*
 * Opaque-to-the-encoder mbuf view. The port ops fill `opaque`/`data`/`capacity`
 * at allocation and consume everything at transmit; the encoders only write
 * `data` and the metadata fields.
 */
typedef struct rusteron_dpdk_mbuf_stct
{
    void *opaque;            /* rte_mbuf* (or fake tag); owned by the port ops */
    uint8_t *data;           /* writable frame start */
    uint32_t capacity;       /* bytes available at data */
    uint32_t frame_len;      /* bytes written (eth+ip+udp+payload) */
    uint32_t ol_flags;       /* RUSTERON_DPDK_MBUF_F_TX_* */
    uint16_t l2_len;
    uint16_t l3_len;
    uint16_t l4_len;
    uint16_t udp_pseudo_csum; /* UDP pseudo-header checksum for TX offload */
} rusteron_dpdk_mbuf_t;

/* IPv4 helpers (addresses are network order). */
uint32_t rusteron_dpdk_ipv4_mask(uint8_t prefix_len);
int rusteron_dpdk_ipv4_is_multicast(uint32_t ip);

/*
 * Build a UDP datagram frame into `m` (already allocated with room). The iovec
 * bytes follow the Ethernet/IPv4/UDP headers; a payload longer than
 * `max_payload` (or the mbuf capacity) is rejected with -1 and the frame is not
 * touched. On success sets frame_len, ol_flags, the L2/L3/L4 lengths and the
 * UDP pseudo-header checksum seed. IPv4 DF is always set; never fragments.
 */
int rusteron_dpdk_packet_build_udp(
    rusteron_dpdk_mbuf_t *m,
    const uint8_t dst_mac[RUSTERON_DPDK_ETH_ADDR_LEN],
    const uint8_t src_mac[RUSTERON_DPDK_ETH_ADDR_LEN],
    uint32_t dst_ip, uint32_t src_ip,
    uint16_t src_port, uint16_t dst_port,
    const struct iovec *iov, size_t iov_count,
    size_t max_payload);

/* ARP request: broadcast dst, 00:00:00:00:00:00 target hw addr. */
int rusteron_dpdk_packet_build_arp_request(
    rusteron_dpdk_mbuf_t *m,
    const uint8_t src_mac[RUSTERON_DPDK_ETH_ADDR_LEN],
    uint32_t src_ip, uint32_t target_ip);

/* ARP reply: unicast to the requester's MAC. */
int rusteron_dpdk_packet_build_arp_reply(
    rusteron_dpdk_mbuf_t *m,
    const uint8_t dst_mac[RUSTERON_DPDK_ETH_ADDR_LEN],
    const uint8_t src_mac[RUSTERON_DPDK_ETH_ADDR_LEN],
    uint32_t sender_ip, uint32_t target_ip);

#ifdef __cplusplus
}
#endif

#endif /* RUSTERON_DPDK_PACKET_H */
