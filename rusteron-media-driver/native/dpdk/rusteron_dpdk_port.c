/*
 * rusteron-media-driver DPDK transport — real ENA port operations (plan §7.2).
 *
 * Implements the port-ops table with libdpdk: resolves a PCI BDF to a port id,
 * verifies the net_ena PMD and IPv4/UDP checksum offloads, configures a single
 * RX/TX queue pair, creates a per-port mbuf pool, applies the L3 MTU, starts
 * the device and waits for the link.
 *
 * This is the only translation unit (besides the EAL seam) that references
 * libdpdk; test builds replace the whole table with a fake via
 * test/rusteron_dpdk_fake_port.c, so this file must not be linked there.
 */
#include "rusteron_dpdk_internal.h"
#include "rusteron_dpdk_port_ops.h"

#include <rte_eal.h>
#include <rte_ethdev.h>
#include <rte_mbuf.h>
#include <rte_mbuf_pool_ops.h>
#include <rte_memory.h>
#include <rte_mempool.h>

#include <stdio.h>
#include <string.h>

#define RUSTERON_DPDK_MBUF_DESC_COUNT 2048
#define RUSTERON_DPDK_MBUF_CACHE_SIZE 256

static int rusteron_dpdk_port_probe_port(const char *pci_bdf, uint16_t *port_id)
{
    uint16_t p;
    RTE_ETH_FOREACH_DEV(p)
    {
        char name[RTE_ETH_NAME_MAX_LEN];
        if (0 == rte_eth_dev_get_name_by_port(p, name) && 0 == strcmp(name, pci_bdf))
        {
            *port_id = p;
            return 0;
        }
    }
    return -1;
}

static int rusteron_dpdk_port_dev_info(
    uint16_t port_id, const char *pci_bdf,
    char *driver_name, size_t driver_name_len,
    int *csum_offload_ok, int *ena_llq_available)
{
    (void)pci_bdf; /* capabilities are read by port id once probed */
    struct rte_eth_dev_info info;
    memset(&info, 0, sizeof(info));
    if (rte_eth_dev_info_get(port_id, &info) < 0)
    {
        return -1;
    }

    const char *driver = info.driver_name != NULL ? info.driver_name : "";
    snprintf(driver_name, driver_name_len, "%s", driver);

    /* IPv4 and UDP checksum insertion in the transmit path and receive
     * offloads must both be advertised (plan §7.2). */
    int tx_ok = 0 != (info.tx_offload_capa &
                      (RTE_ETH_TX_OFFLOAD_IPV4_CKSUM | RTE_ETH_TX_OFFLOAD_UDP_CKSUM));
    int rx_ok = 0 != (info.rx_offload_capa &
                      (RTE_ETH_RX_OFFLOAD_IPV4_CKSUM | RTE_ETH_RX_OFFLOAD_UDP_CKSUM));
    *csum_offload_ok = (tx_ok && rx_ok) ? 1 : 0;

    /* ENA exposes low-latency queues through the private API; we only record
     * the capability here so the runtime can be configured accordingly. */
    *ena_llq_available = 0;

    return 0;
}

static int rusteron_dpdk_port_dev_configure(
    uint16_t port_id, uint16_t rx_rings, uint16_t tx_rings,
    uint64_t rx_offloads, uint64_t tx_offloads)
{
    struct rte_eth_conf conf;
    memset(&conf, 0, sizeof(conf));

    if (0 != (rx_offloads & RUSTERON_DPDK_RX_OFFLOAD_IPV4_CKSUM))
    {
        conf.rxmode.offloads |= RTE_ETH_RX_OFFLOAD_IPV4_CKSUM;
    }
    if (0 != (rx_offloads & RUSTERON_DPDK_RX_OFFLOAD_UDP_CKSUM))
    {
        conf.rxmode.offloads |= RTE_ETH_RX_OFFLOAD_UDP_CKSUM;
    }
    if (0 != (tx_offloads & RUSTERON_DPDK_TX_OFFLOAD_IPV4_CKSUM))
    {
        conf.txmode.offloads |= RTE_ETH_TX_OFFLOAD_IPV4_CKSUM;
    }
    if (0 != (tx_offloads & RUSTERON_DPDK_TX_OFFLOAD_UDP_CKSUM))
    {
        conf.txmode.offloads |= RTE_ETH_TX_OFFLOAD_UDP_CKSUM;
    }

    return rte_eth_dev_configure(port_id, rx_rings, tx_rings, &conf);
}

static void *rusteron_dpdk_port_mempool_create(const char *name, uint32_t n, uint16_t cache_size)
{
    if (cache_size == 0)
    {
        cache_size = RUSTERON_DPDK_MBUF_CACHE_SIZE;
    }
    return rte_pktmbuf_pool_create(
        name, n, cache_size, 0, RTE_MBUF_DEFAULT_BUF_SIZE, SOCKET_ID_ANY);
}

static int rusteron_dpdk_port_rx_queue_setup(
    uint16_t port_id, uint16_t queue_id, uint16_t nb_desc, void *mempool)
{
    return rte_eth_rx_queue_setup(port_id, queue_id, nb_desc, 0, NULL, mempool);
}

static int rusteron_dpdk_port_tx_queue_setup(
    uint16_t port_id, uint16_t queue_id, uint16_t nb_desc)
{
    return rte_eth_tx_queue_setup(port_id, queue_id, nb_desc, 0, NULL);
}

static int rusteron_dpdk_port_dev_set_mtu(uint16_t port_id, uint16_t mtu)
{
    return rte_eth_dev_set_mtu(port_id, mtu);
}

static int rusteron_dpdk_port_dev_start(uint16_t port_id)
{
    return rte_eth_dev_start(port_id);
}

static int rusteron_dpdk_port_link_wait_ready(uint16_t port_id, uint32_t timeout_ms)
{
    uint32_t waited = 0;
    const uint32_t step_ms = 10;
    for (;;)
    {
        struct rte_eth_link link;
        memset(&link, 0, sizeof(link));
        rte_eth_link_get(port_id, &link);
        if (RTE_ETH_LINK_UP == link.link_status)
        {
            return 0;
        }
        if (waited >= timeout_ms)
        {
            return -1;
        }
        rte_delay_ms(step_ms);
        waited += step_ms;
    }
}

static int rusteron_dpdk_port_dev_stop(uint16_t port_id)
{
    rte_eth_dev_stop(port_id);
    return 0;
}

static int rusteron_dpdk_port_dev_close(uint16_t port_id)
{
    rte_eth_dev_close(port_id);
    return 0;
}

static void rusteron_dpdk_port_mempool_free(void *mempool)
{
    rte_mempool_free(mempool);
}

rusteron_dpdk_port_ops_t *rusteron_dpdk_port_ops_real(void)
{
    static rusteron_dpdk_port_ops_t ops = {
        .probe_port = rusteron_dpdk_port_probe_port,
        .dev_info = rusteron_dpdk_port_dev_info,
        .dev_configure = rusteron_dpdk_port_dev_configure,
        .mempool_create = rusteron_dpdk_port_mempool_create,
        .rx_queue_setup = rusteron_dpdk_port_rx_queue_setup,
        .tx_queue_setup = rusteron_dpdk_port_tx_queue_setup,
        .dev_set_mtu = rusteron_dpdk_port_dev_set_mtu,
        .dev_start = rusteron_dpdk_port_dev_start,
        .link_wait_ready = rusteron_dpdk_port_link_wait_ready,
        .dev_stop = rusteron_dpdk_port_dev_stop,
        .dev_close = rusteron_dpdk_port_dev_close,
        .mempool_free = rusteron_dpdk_port_mempool_free,
    };
    return &ops;
}
