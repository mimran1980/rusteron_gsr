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

#ifdef __cplusplus
extern "C" {
#endif

typedef struct rusteron_dpdk_port_ops_stct
{
    /* Resolve a PCI BDF to a DPDK port id. -1 if the device is not probed. */
    int (*probe_port)(const char *pci_bdf, uint16_t *port_id);

    /*
     * Read the device driver name and record whether IPv4/UDP checksum offloads
     * and ENA LLQ/write-combining are supported. 0 on success.
     */
    int (*dev_info)(
        uint16_t port_id,
        const char *pci_bdf,
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
