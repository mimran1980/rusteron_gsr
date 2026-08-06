/*
 * rusteron-media-driver DPDK transport — ARP table (plan §7.5).
 *
 * A fixed 1024-entry open-addressed next-hop cache shared by both role ENAs.
 * Entries move EMPTY -> INCOMPLETE (request sent) -> REACHABLE (reply learned);
 * a reachable entry expires after 30 s and is re-resolved. Requests are
 * rate-limited to one per 100 ms per next hop; no packets are ever queued —
 * Aeron retries after a zero send result.
 *
 * Learning is strict: only replies addressed to a role's local MAC/IP and
 * relevant to an outstanding request update the table, and a reachable entry is
 * never overwritten (gratuitous/unrelated ARP cannot poison a live next hop).
 */
#ifndef RUSTERON_DPDK_ARP_H
#define RUSTERON_DPDK_ARP_H

#include <stddef.h>
#include <stdint.h>

#include "rusteron_dpdk_packet.h"

#ifdef __cplusplus
extern "C" {
#endif

#define RUSTERON_DPDK_ARP_TABLE_SIZE 1024
#define RUSTERON_DPDK_ARP_RETRY_MS 100
#define RUSTERON_DPDK_ARP_EXPIRE_MS 30000

/* Opaque structs defined in rusteron_dpdk_internal.h; only the pointers are
 * needed by the arp.h declarations (arp.c includes internal.h). */
typedef struct rusteron_dpdk_transport_stct rusteron_dpdk_transport_t;
typedef struct rusteron_dpdk_port_stct rusteron_dpdk_port_t;

typedef enum rusteron_dpdk_arp_state_en
{
    RUSTERON_DPDK_ARP_EMPTY = 0,
    RUSTERON_DPDK_ARP_INCOMPLETE,
    RUSTERON_DPDK_ARP_REACHABLE
} rusteron_dpdk_arp_state_t;

typedef struct rusteron_dpdk_arp_entry_stct
{
    uint32_t ip;                 /* next-hop IPv4, network order; 0 = EMPTY */
    uint8_t mac[RUSTERON_DPDK_ETH_ADDR_LEN];
    uint8_t state;               /* rusteron_dpdk_arp_state_t */
    uint64_t last_request_ms;    /* clock ms of the most recent request */
    uint64_t last_seen_ms;       /* clock ms when learned/refreshed */
} rusteron_dpdk_arp_entry_t;

typedef struct rusteron_dpdk_arp_table_stct
{
    rusteron_dpdk_arp_entry_t entries[RUSTERON_DPDK_ARP_TABLE_SIZE];
} rusteron_dpdk_arp_table_t;

/*
 * Resolve a next hop. Returns 1 and copies the MAC when the entry is fresh and
 * reachable; otherwise it advances the entry to INCOMPLETE, sends a rate-limited
 * ARP request through the port (unless the table is full or a request went out
 * within the retry window) and returns 0 so the caller reports a retryable
 * zero-send result.
 */
int rusteron_dpdk_arp_resolve(
    rusteron_dpdk_arp_table_t *table,
    rusteron_dpdk_transport_t *runtime,
    rusteron_dpdk_port_t *port,
    uint32_t next_hop_ip,
    uint8_t out_mac[RUSTERON_DPDK_ETH_ADDR_LEN]);

/*
 * Handle an incoming ARP frame (request or reply). Replies to requests for a
 * role's local IPv4 are sent; replies addressed to a role's local MAC/IP that
 * match an outstanding request are learned. Returns 1 when the frame was
 * consumed (a reply was sent or a request learned), 0 when ignored.
 */
int rusteron_dpdk_arp_handle_frame(
    rusteron_dpdk_arp_table_t *table,
    rusteron_dpdk_transport_t *runtime,
    rusteron_dpdk_port_t *rx_port,
    const uint8_t *frame, size_t frame_len);

#ifdef __cplusplus
}
#endif

#endif /* RUSTERON_DPDK_ARP_H */
