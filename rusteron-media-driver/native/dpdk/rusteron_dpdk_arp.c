/*
 * rusteron-media-driver DPDK transport — ARP table (plan §7.5).
 *
 * See rusteron_dpdk_arp.h for the behaviour contract. This file references the
 * full transport/port layout (via internal.h) only to reach the port ops for
 * allocating, building and transmitting ARP frames — it has no libdpdk symbols.
 */
#include "rusteron_dpdk_internal.h"

#include <string.h>

uint64_t rusteron_dpdk_clock_ms(void); /* runtime.c */

/* Mixer so consecutive next-hop IPs spread across the open-addressed table. */
static uint32_t rusteron_dpdk_arp_hash(uint32_t ip)
{
    uint32_t h = ip;
    h = (h ^ (h >> 16)) * 0x85ebca6bu;
    h = (h ^ (h >> 13)) * 0xc2b2ae35u;
    return h ^ (h >> 16);
}

/* Open-addressed probe. Returns the entry holding `ip`, or the first EMPTY slot
 * when the entry is absent (candidate insert point), or NULL when the probe
 * chain is full. */
static rusteron_dpdk_arp_entry_t *rusteron_dpdk_arp_slot_for(
    rusteron_dpdk_arp_table_t *table, uint32_t ip)
{
    const uint32_t mask = RUSTERON_DPDK_ARP_TABLE_SIZE - 1;
    size_t slot = (size_t)(rusteron_dpdk_arp_hash(ip) & mask);

    for (size_t i = 0; i < RUSTERON_DPDK_ARP_TABLE_SIZE; i++)
    {
        rusteron_dpdk_arp_entry_t *e = &table->entries[(slot + i) & mask];
        if (RUSTERON_DPDK_ARP_EMPTY == e->state)
        {
            return e; /* empty slot: absent, insert here */
        }
        if (e->ip == ip)
        {
            return e;
        }
    }
    return NULL; /* table full */
}

static void rusteron_dpdk_arp_send_frame(
    rusteron_dpdk_transport_t *runtime, rusteron_dpdk_port_t *port, rusteron_dpdk_mbuf_t *m)
{
    const rusteron_dpdk_port_ops_t *ops = runtime->ops;
    rusteron_dpdk_mbuf_t *burst[1] = { m };
    /* tx_burst takes ownership of the view: accepted frames belong to the NIC,
     * rejected ones are released by the impl. */
    ops->tx_burst(port->port_id, port->tx_queue_id, burst, 1);
}

static void rusteron_dpdk_arp_send_request(
    rusteron_dpdk_transport_t *runtime, rusteron_dpdk_port_t *port, uint32_t next_hop_ip)
{
    const rusteron_dpdk_port_ops_t *ops = runtime->ops;
    rusteron_dpdk_mbuf_t m;
    if (ops->mbuf_alloc(port->mempool, &m) < 0)
    {
        return; /* pool exhausted: the entry stays INCOMPLETE and is retried */
    }
    if (rusteron_dpdk_packet_build_arp_request(&m, port->mac, port->local_ip, next_hop_ip) < 0)
    {
        ops->mbuf_release(&m);
        return;
    }
    rusteron_dpdk_arp_send_frame(runtime, port, &m);
}

static void rusteron_dpdk_arp_send_reply(
    rusteron_dpdk_transport_t *runtime, rusteron_dpdk_port_t *port,
    const uint8_t requester_mac[RUSTERON_DPDK_ETH_ADDR_LEN], uint32_t requester_ip)
{
    const rusteron_dpdk_port_ops_t *ops = runtime->ops;
    rusteron_dpdk_mbuf_t m;
    if (ops->mbuf_alloc(port->mempool, &m) < 0)
    {
        return;
    }
    if (rusteron_dpdk_packet_build_arp_reply(&m, requester_mac, port->mac, port->local_ip, requester_ip) < 0)
    {
        ops->mbuf_release(&m);
        return;
    }
    rusteron_dpdk_arp_send_frame(runtime, port, &m);
}

int rusteron_dpdk_arp_resolve(
    rusteron_dpdk_arp_table_t *table,
    rusteron_dpdk_transport_t *runtime,
    rusteron_dpdk_port_t *port,
    uint32_t next_hop_ip,
    uint8_t out_mac[RUSTERON_DPDK_ETH_ADDR_LEN])
{
    uint64_t now = rusteron_dpdk_clock_ms();
    rusteron_dpdk_arp_entry_t *e = rusteron_dpdk_arp_slot_for(table, next_hop_ip);

    if (NULL == e)
    {
        return 0; /* table full; retry next send */
    }

    if (RUSTERON_DPDK_ARP_REACHABLE == e->state &&
        now - e->last_seen_ms < RUSTERON_DPDK_ARP_EXPIRE_MS)
    {
        memcpy(out_mac, e->mac, RUSTERON_DPDK_ETH_ADDR_LEN);
        return 1;
    }

    /* Empty (fresh insert) or stale/incomplete: (re)start resolution. */
    e->state = RUSTERON_DPDK_ARP_INCOMPLETE;
    e->ip = next_hop_ip;
    if (now - e->last_request_ms >= RUSTERON_DPDK_ARP_RETRY_MS)
    {
        e->last_request_ms = now;
        rusteron_dpdk_arp_send_request(runtime, port, next_hop_ip);
    }
    return 0;
}

/* Read a 4-byte network-order address from the frame. */
static uint32_t rusteron_dpdk_frame_read_ip(const uint8_t *p)
{
    uint32_t v;
    memcpy(&v, p, 4);
    return v;
}

static uint16_t rusteron_dpdk_frame_read_u16(const uint8_t *p)
{
    return (uint16_t)((p[0] << 8) | p[1]);
}

int rusteron_dpdk_arp_handle_frame(
    rusteron_dpdk_arp_table_t *table,
    rusteron_dpdk_transport_t *runtime,
    rusteron_dpdk_port_t *rx_port,
    const uint8_t *frame, size_t frame_len)
{
    /* Ethernet + ARP = 42 bytes minimum. */
    if (NULL == frame || frame_len < RUSTERON_DPDK_ETH_HDR_LEN + 28)
    {
        return 0;
    }

    /* EtherType 0x0806 (ARP). */
    if (frame[12] != 0x08 || frame[13] != 0x06)
    {
        return 0;
    }

    const uint8_t *a = frame + RUSTERON_DPDK_ETH_HDR_LEN;
    if (rusteron_dpdk_frame_read_u16(a + 0) != RUSTERON_DPDK_ARP_HTYPE_ETHERNET ||
        rusteron_dpdk_frame_read_u16(a + 2) != RUSTERON_DPDK_ARP_PTYPE_IPV4 ||
        a[4] != RUSTERON_DPDK_ARP_HLEN ||
        a[5] != RUSTERON_DPDK_ARP_PLEN)
    {
        return 0;
    }

    uint16_t oper = rusteron_dpdk_frame_read_u16(a + 6);
    const uint8_t *sha = a + 8;               /* sender hardware address */
    uint32_t spa = rusteron_dpdk_frame_read_ip(a + 14); /* sender protocol */
    uint32_t tpa = rusteron_dpdk_frame_read_ip(a + 24); /* target protocol */

    if (RUSTERON_DPDK_ARP_OPER_REQUEST == oper)
    {
        /* Respond only to requests for this role's configured local IPv4. */
        if (tpa == rx_port->local_ip)
        {
            rusteron_dpdk_arp_send_reply(runtime, rx_port, sha, spa);
            return 1;
        }
        return 0;
    }

    if (RUSTERON_DPDK_ARP_OPER_REPLY == oper)
    {
        /* Learn only when the reply is addressed to this role's local MAC and
         * matches an outstanding (INCOMPLETE) request. A reachable entry is
         * never overwritten, so gratuitous/unrelated ARP cannot poison it. */
        if (0 == memcmp(frame, rx_port->mac, RUSTERON_DPDK_ETH_ADDR_LEN))
        {
            rusteron_dpdk_arp_entry_t *e = rusteron_dpdk_arp_slot_for(table, spa);
            if (NULL != e && RUSTERON_DPDK_ARP_INCOMPLETE == e->state)
            {
                memcpy(e->mac, sha, RUSTERON_DPDK_ETH_ADDR_LEN);
                e->state = RUSTERON_DPDK_ARP_REACHABLE;
                e->last_seen_ms = rusteron_dpdk_clock_ms();
                return 1;
            }
        }
        return 0;
    }

    return 0;
}
