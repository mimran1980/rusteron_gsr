/*
 * rusteron-media-driver DPDK transport — injectable port operations (plan §7.2).
 *
 * The EAL and port seams are deliberately split from the orchestration layer
 * (runtime.c) so the runtime can be linked and deterministically tested without
 * linking libdpdk:
 *
 *   - The default port-ops getter rusteron_dpdk_port_ops_real() is provided by
 *     rusteron_dpdk_port.c (real rte_eth_* calls) in production, and by
 *     test/rusteron_dpdk_fake_port.c in test builds. The linker picks the
 *     definition from whichever archive is linked, exactly one of the two.
 *
 * No DPDK type leaks through this header: handles are opaque pointers and
 * offload/queue values are plain integers so the fake never needs rte headers.
 */
#ifndef RUSTERON_DPDK_PORT_OPS_H
#define RUSTERON_DPDK_PORT_OPS_H

#include <stddef.h>
#include <stdint.h>

#include "rusteron_dpdk_packet.h" /* rusteron_dpdk_mbuf_t, RUSTERON_DPDK_ETH_ADDR_LEN */

#ifdef __cplusplus
extern "C" {
#endif

typedef struct rusteron_dpdk_port_ops_stct
{
    /* Resolve a PCI BDF to a DPDK port id. -1 if the device is not probed. */
    int (*probe_port)(const char *pci_bdf, uint16_t *port_id);

    /*
     * Read the device MAC, driver name, and whether IPv4/UDP checksum offloads
     * and ENA LLQ/write-combining are supported. 0 on success.
     */
    int (*dev_info)(
        uint16_t port_id,
        const char *pci_bdf,
        uint8_t mac[RUSTERON_DPDK_ETH_ADDR_LEN],
        char *driver_name, size_t driver_name_len,
        int *csum_offload_ok,
        int *ena_llq_available);

    /* Configure one RX and one TX queue plus the requested offloads. */
    int (*dev_configure)(
        uint16_t port_id, uint16_t rx_rings, uint16_t tx_rings,
        uint64_t rx_offloads, uint64_t tx_offloads);

    /* Create a packet mbuf pool; returns an opaque handle (NULL on failure). */
    void *(*mempool_create)(const char *name, uint32_t n, uint16_t cache_size);

    /* Queue 0 on each direction, backed by the role mempool for RX. */
    int (*rx_queue_setup)(
        uint16_t port_id, uint16_t queue_id, uint16_t nb_desc, void *mempool);
    int (*tx_queue_setup)(uint16_t port_id, uint16_t queue_id, uint16_t nb_desc);

    /* Apply the L3 MTU (payload + IPv4 + UDP). */
    int (*dev_set_mtu)(uint16_t port_id, uint16_t mtu);

    int (*dev_start)(uint16_t port_id);

    /* Poll the link until UP or `timeout_ms` elapses. 0 when UP. */
    int (*link_wait_ready)(uint16_t port_id, uint32_t timeout_ms);

    int (*dev_stop)(uint16_t port_id);
    int (*dev_close)(uint16_t port_id);
    void (*mempool_free)(void *mempool);

    /* Data path (plan §7.4). */

    /* Allocate one packet mbuf from the pool; fills the view's opaque/data/
     * capacity. 0 on success, -1 when the pool is exhausted. */
    int (*mbuf_alloc)(void *mempool, rusteron_dpdk_mbuf_t *m);

    /* Return a view's mbuf to the pool. Only called for views never handed to
     * tx_burst (e.g. a build failure before batching). */
    void (*mbuf_release)(rusteron_dpdk_mbuf_t *m);

    /*
     * Transmit a burst. The impl takes ownership of every view: the accepted
     * prefix becomes the NIC's and every rejected/unsent view is released, so
     * the caller never touches a view after the call. Returns the number of
     * frames accepted (0..=nb, a contiguous prefix from index 0).
     */
    uint16_t (*tx_burst)(
        uint16_t port_id, uint16_t tx_queue_id,
        rusteron_dpdk_mbuf_t **pkts, uint16_t nb);

    /*
     * Receive a burst. The impl fills every returned view (opaque/data/
     * capacity/frame_len/nb_segs/rx_ol_flags); the caller owns the views and
     * must release them via mbuf_release once processed. Returns the number of
     * frames received (0..=nb, a contiguous prefix from index 0).
     */
    uint16_t (*rx_burst)(
        uint16_t port_id, uint16_t rx_queue_id,
        rusteron_dpdk_mbuf_t **pkts, uint16_t nb);
} rusteron_dpdk_port_ops_t;

/*
 * The active port operations. Production links rusteron_dpdk_port.a (real);
 * test binaries link rusteron_dpdk_fake.a instead. Exactly one of the two
 * archives must define this symbol in any given link.
 */
rusteron_dpdk_port_ops_t *rusteron_dpdk_port_ops_real(void);

#ifdef __cplusplus
}
#endif

#endif /* RUSTERON_DPDK_PORT_OPS_H */
