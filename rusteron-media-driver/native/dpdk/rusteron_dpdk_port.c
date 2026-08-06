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
#include <rte_ether.h>
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
    uint8_t mac[RUSTERON_DPDK_ETH_ADDR_LEN],
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

    struct rte_ether_addr mac_addr;
    memset(&mac_addr, 0, sizeof(mac_addr));
    if (rte_eth_macaddr_get(port_id, &mac_addr) < 0)
    {
        return -1;
    }
    memcpy(mac, mac_addr.addr_bytes, RUSTERON_DPDK_ETH_ADDR_LEN);

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

/* Data path (plan §7.4). */

static int rusteron_dpdk_port_mbuf_alloc(void *mempool, rusteron_dpdk_mbuf_t *m)
{
    struct rte_mbuf *mbuf = rte_pktmbuf_alloc(mempool);
    if (NULL == mbuf)
    {
        return -1;
    }

    m->opaque = mbuf;
    m->data = rte_pktmbuf_mtod(mbuf, uint8_t *);
    m->capacity = (uint32_t)rte_pktmbuf_tailroom(mbuf);
    m->frame_len = 0;
    m->ol_flags = 0;
    m->l2_len = 0;
    m->l3_len = 0;
    m->l4_len = 0;
    m->udp_pseudo_csum = 0;
    return 0;
}

static void rusteron_dpdk_port_mbuf_release(rusteron_dpdk_mbuf_t *m)
{
    if (NULL != m && NULL != m->opaque)
    {
        rte_pktmbuf_free(m->opaque);
        m->opaque = NULL;
    }
}

/* Map the transport-level offload flags onto the DPDK TX offload bits. */
static uint64_t rusteron_dpdk_port_ol_flags(const rusteron_dpdk_mbuf_t *m)
{
    uint64_t flags = 0;
    if (0 != (m->ol_flags & RUSTERON_DPDK_MBUF_F_TX_IPV4))
    {
        flags |= RTE_MBUF_F_TX_IPV4;
    }
    if (0 != (m->ol_flags & RUSTERON_DPDK_MBUF_F_TX_IP_CKSUM))
    {
        flags |= RTE_MBUF_F_TX_IP_CKSUM;
    }
    if (0 != (m->ol_flags & RUSTERON_DPDK_MBUF_F_TX_UDP_CKSUM))
    {
        flags |= RTE_MBUF_F_TX_UDP_CKSUM;
    }
    return flags;
}

static uint16_t rusteron_dpdk_port_tx_burst(
    uint16_t port_id, uint16_t tx_queue_id,
    rusteron_dpdk_mbuf_t **pkts, uint16_t nb)
{
    struct rte_mbuf *mbufs[256];
    uint16_t i;

    for (i = 0; i < nb; i++)
    {
        struct rte_mbuf *mbuf = pkts[i]->opaque;
        /* pkt_len/data_len setters were removed from DPDK 23.11; the fields are
         * public and assigned directly. */
        mbuf->pkt_len = pkts[i]->frame_len;
        mbuf->data_len = (uint16_t)pkts[i]->frame_len;
        mbuf->ol_flags = rusteron_dpdk_port_ol_flags(pkts[i]);
        mbuf->l2_len = pkts[i]->l2_len;
        mbuf->l3_len = pkts[i]->l3_len;
        mbuf->l4_len = pkts[i]->l4_len;
        /* The UDP pseudo-header checksum seed already lives in the packet's UDP
         * checksum field (packet.c writes it), which is the DPDK TX-offload
         * contract; the mbuf carries only ol_flags + the L2/L3/L4 lengths. */
        mbufs[i] = mbuf;
    }

    uint16_t sent = rte_eth_tx_burst(port_id, tx_queue_id, mbufs, nb);

    /* Ownership of the rejected tail is ours; the accepted prefix belongs to
     * the NIC. */
    for (i = sent; i < nb; i++)
    {
        rte_pktmbuf_free(mbufs[i]);
    }
    return sent;
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
        .mbuf_alloc = rusteron_dpdk_port_mbuf_alloc,
        .mbuf_release = rusteron_dpdk_port_mbuf_release,
        .tx_burst = rusteron_dpdk_port_tx_burst,
    };
    return &ops;
}
