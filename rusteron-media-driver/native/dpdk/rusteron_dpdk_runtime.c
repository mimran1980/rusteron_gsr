/*
 * rusteron-media-driver DPDK transport — runtime orchestration (plan §7.2).
 *
 * Owns the process-lifetime EAL singleton and drives both role ports through
 * the injectable port-ops seam: probe -> dev_info (net_ena + offloads) ->
 * dev_configure -> mempool -> rx/tx queue -> set_mtu -> start -> link. Any
 * failure cleans up prior resources in reverse initialization order and the
 * error is recorded for rusteron_dpdk_last_error()/last_error_code().
 *
 * This translation unit references no libdpdk symbols directly: EAL goes
 * through the rusteron_dpdk_eal_* seam and ports through the port-ops table, so
 * it can be linked and deterministically tested without libdpdk.
 */

/* clock_gettime/CLOCK_MONOTONIC need POSIX 199309; the strict -std=c11 build
 * hides them unless this is declared before any libc header is included. */
#define _POSIX_C_SOURCE 200809L

#include "rusteron_dpdk_internal.h"

#include <arpa/inet.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define RUSTERON_DPDK_IP_UDP_OVERHEAD 28 /* IPv4 + UDP headers above the payload */
#define RUSTERON_DPDK_LINK_TIMEOUT_MS 5000

/* Process-lifetime state. EAL cannot be reinitialized (plan §7.2), so the
 * ever-initialized flag is never cleared outside the test reset hook. */
static int rusteron_dpdk_test_eal_mode = RUSTERON_DPDK_EAL_REAL;
static int rusteron_dpdk_eal_ever_initialized = 0;
static const rusteron_dpdk_port_ops_t *rusteron_dpdk_port_ops_override = NULL;
static uint64_t rusteron_dpdk_test_clock_ms_value = 0;
static int rusteron_dpdk_test_clock_ms_pinned = 0;

void rusteron_dpdk_test_set_eal_mode(int mode)
{
    rusteron_dpdk_test_eal_mode = mode;
}

void rusteron_dpdk_test_set_clock_ms(uint64_t ms)
{
    rusteron_dpdk_test_clock_ms_value = ms;
    rusteron_dpdk_test_clock_ms_pinned = 1;
}

uint64_t rusteron_dpdk_clock_ms(void)
{
    if (rusteron_dpdk_test_clock_ms_pinned)
    {
        return rusteron_dpdk_test_clock_ms_value;
    }
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000u + (uint64_t)ts.tv_nsec / 1000000u;
}

void rusteron_dpdk_test_reset(void)
{
    rusteron_dpdk_test_eal_mode = RUSTERON_DPDK_EAL_REAL;
    rusteron_dpdk_eal_ever_initialized = 0;
    rusteron_dpdk_port_ops_override = NULL;
    rusteron_dpdk_test_clock_ms_value = 0;
    rusteron_dpdk_test_clock_ms_pinned = 0;
    rusteron_dpdk_transport_test_reset();
    /* Clear the thread-local error state so last_error()/last_error_code()
     * start from "no error" in the next test (tests run single-threaded). */
    rusteron_dpdk_set_error_code("", RUSTERON_DPDK_ERR_OK);
}

void rusteron_dpdk_set_port_ops(const rusteron_dpdk_port_ops_t *ops)
{
    rusteron_dpdk_port_ops_override = ops;
}

static const rusteron_dpdk_port_ops_t *rusteron_dpdk_active_ops(void)
{
    if (NULL != rusteron_dpdk_port_ops_override)
    {
        return rusteron_dpdk_port_ops_override;
    }
    return rusteron_dpdk_port_ops_real();
}

/* Parse a dotted-quad into a network-order uint32_t; 0 on success. */
static int rusteron_dpdk_parse_ipv4(const char *text, uint32_t *out)
{
    struct in_addr addr;
    if (NULL == text || 1 != inet_pton(AF_INET, text, &addr))
    {
        return -1;
    }
    *out = addr.s_addr;
    return 0;
}

/* Bring EAL up once per process (or skip it under the test hook). The EAL
 * singleton guard fires in every mode, including the test SKIP mode, so a
 * second transport in one process always reports ALREADY_INITIALIZED. */
static int rusteron_dpdk_runtime_eal_up(rusteron_dpdk_transport_t *native)
{
    if (rusteron_dpdk_eal_ever_initialized || rusteron_dpdk_eal_is_initialized())
    {
        rusteron_dpdk_set_error_code(
            "DPDK EAL is already initialized in this process (only one EAL runtime is permitted)",
            RUSTERON_DPDK_ERR_ALREADY_INITIALIZED);
        return -1;
    }

    if (RUSTERON_DPDK_EAL_SKIP == rusteron_dpdk_test_eal_mode)
    {
        native->eal_up = 1;
        rusteron_dpdk_eal_ever_initialized = 1;
        return 0;
    }

    rusteron_dpdk_eal_params_t params;
    memset(&params, 0, sizeof(params));
    params.config = &native->config;
    params.mode = rusteron_dpdk_test_eal_mode;

    char errbuf[512] = "";
    if (rusteron_dpdk_eal_init(&params, errbuf, sizeof(errbuf)) < 0)
    {
        rusteron_dpdk_set_error(errbuf[0] != '\0' ? errbuf : "DPDK EAL initialization failed");
        return -1;
    }

    native->eal_up = 1;
    rusteron_dpdk_eal_ever_initialized = 1;
    return 0;
}

static int rusteron_dpdk_init_port(rusteron_dpdk_transport_t *native, rusteron_dpdk_port_t *p)
{
    const rusteron_dpdk_port_ops_t *ops = native->ops;
    const rusteron_dpdk_config_t *cfg = &native->config;
    char message[512];

    if (ops->probe_port(p->pci, &p->port_id) < 0)
    {
        snprintf(message, sizeof(message),
                 "device %s is not probed by DPDK (allow-listed ENA must be bound via VFIO)", p->pci);
        rusteron_dpdk_set_error(message);
        return -1;
    }

    char driver[64] = "";
    if (ops->dev_info(p->port_id, p->pci, p->mac, driver, sizeof(driver),
                      &p->csum_offload_ok, &p->ena_llq_available) < 0)
    {
        snprintf(message, sizeof(message),
                 "cannot read capabilities of device %s (port %u)", p->pci, p->port_id);
        rusteron_dpdk_set_error(message);
        return -1;
    }
    if (0 == memcmp(p->mac, "\x00\x00\x00\x00\x00\x00", 6))
    {
        snprintf(message, sizeof(message),
                 "device %s (port %u) reports an all-zero MAC", p->pci, p->port_id);
        rusteron_dpdk_set_error(message);
        return -1;
    }
    /* Keep the PMD name for the port-info counter label (plan §9). */
    memcpy(p->driver, driver, sizeof(p->driver));
    /* The ENA-specific guarantees (net_ena PMD, IPv4/UDP checksum offload) are
     * required for a PCI ENA but not for a virtual/TAP device: vdev PMDs report
     * their own driver name and may software-checksum (plan §11.2). */
    if (rusteron_dpdk_selector_is_pci(p->pci))
    {
        if (0 != strcmp(driver, "net_ena"))
        {
            snprintf(message, sizeof(message),
                     "device %s reports driver %s, expected the net_ena PMD",
                     p->pci, driver[0] != '\0' ? driver : "<unknown>");
            rusteron_dpdk_set_error(message);
            return -1;
        }
        if (!p->csum_offload_ok)
        {
            snprintf(message, sizeof(message),
                     "device %s lacks the required IPv4/UDP checksum offloads", p->pci);
            rusteron_dpdk_set_error(message);
            return -1;
        }
    }

    uint16_t l3_mtu = (uint16_t)(cfg->max_aeron_mtu + RUSTERON_DPDK_IP_UDP_OVERHEAD);
    uint64_t rx_off = RUSTERON_DPDK_RX_OFFLOAD_IPV4_CKSUM | RUSTERON_DPDK_RX_OFFLOAD_UDP_CKSUM;
    uint64_t tx_off = RUSTERON_DPDK_TX_OFFLOAD_IPV4_CKSUM | RUSTERON_DPDK_TX_OFFLOAD_UDP_CKSUM;
    if (ops->dev_configure(p->port_id, 1, 1, rx_off, tx_off) < 0)
    {
        snprintf(message, sizeof(message), "failed to configure queues/offloads on %s", p->pci);
        rusteron_dpdk_set_error(message);
        return -1;
    }
    p->configured = 1;

    /* Keep the mempool name short: when a pool is split across memzones DPDK
     * names them MP_<pool>_<n>, and RTE_MEMZONE_NAMESIZE is 32 — the old
     * "rusteron_dpdk_receiver_pool" (27) produced a 32-char memzone name and
     * failed with ENAMETOOLONG on the split path (rte_mempool_populate_default). */
    char pool_name[64];
    snprintf(pool_name, sizeof(pool_name), "rusteron_%s_pool",
             p->role == RUSTERON_DPDK_ROLE_SENDER ? "sender" : "receiver");
    p->mempool = ops->mempool_create(pool_name, cfg->mbufs_per_port, cfg->mempool_cache);
    if (NULL == p->mempool)
    {
        /* The port op records the underlying rte error (rte_strerror); only
         * fall back to a generic message if the op did not set one. */
        if (RUSTERON_DPDK_ERR_OK == rusteron_dpdk_last_error_code())
        {
            snprintf(message, sizeof(message), "mempool creation failed for %s", p->pci);
            rusteron_dpdk_set_error(message);
        }
        return -1;
    }

    if (ops->rx_queue_setup(p->port_id, 0, cfg->rx_descriptors, p->mempool) < 0)
    {
        snprintf(message, sizeof(message), "rx queue setup failed on %s", p->pci);
        rusteron_dpdk_set_error(message);
        return -1;
    }
    p->rx_queue_id = 0;

    if (ops->tx_queue_setup(p->port_id, 0, cfg->tx_descriptors) < 0)
    {
        snprintf(message, sizeof(message), "tx queue setup failed on %s", p->pci);
        rusteron_dpdk_set_error(message);
        return -1;
    }
    p->tx_queue_id = 0;

    if (ops->dev_set_mtu(p->port_id, l3_mtu) < 0)
    {
        snprintf(message, sizeof(message),
                 "device %s rejected L3 MTU %u (max_aeron_mtu %zu)", p->pci, l3_mtu, cfg->max_aeron_mtu);
        rusteron_dpdk_set_error(message);
        return -1;
    }

    if (ops->dev_start(p->port_id) < 0)
    {
        snprintf(message, sizeof(message), "device %s failed to start", p->pci);
        rusteron_dpdk_set_error(message);
        return -1;
    }
    p->started = 1;

    if (ops->link_wait_ready(p->port_id, RUSTERON_DPDK_LINK_TIMEOUT_MS) < 0)
    {
        snprintf(message, sizeof(message),
                 "link on %s (port %u) did not come up within %d ms",
                 p->pci, p->port_id, RUSTERON_DPDK_LINK_TIMEOUT_MS);
        rusteron_dpdk_set_error(message);
        return -1;
    }
    p->link_up = 1;

    return 0;
}

int rusteron_dpdk_runtime_init(rusteron_dpdk_transport_t *native)
{
    native->ops = rusteron_dpdk_active_ops();

    native->sender.role = RUSTERON_DPDK_ROLE_SENDER;
    memcpy(native->sender.pci, native->config.sender_pci, sizeof(native->sender.pci));
    memcpy(native->sender.local_ipv4, native->config.sender_ipv4, sizeof(native->sender.local_ipv4));
    native->sender.prefix_len = native->config.sender_prefix_len;
    memcpy(native->sender.gateway_ipv4, native->config.sender_gateway, sizeof(native->sender.gateway_ipv4));

    native->receiver.role = RUSTERON_DPDK_ROLE_RECEIVER;
    memcpy(native->receiver.pci, native->config.receiver_pci, sizeof(native->receiver.pci));
    memcpy(native->receiver.local_ipv4, native->config.receiver_ipv4, sizeof(native->receiver.local_ipv4));
    native->receiver.prefix_len = native->config.receiver_prefix_len;
    memcpy(native->receiver.gateway_ipv4, native->config.receiver_gateway, sizeof(native->receiver.gateway_ipv4));

    if (rusteron_dpdk_parse_ipv4(native->config.sender_ipv4, &native->sender.local_ip) < 0 ||
        rusteron_dpdk_parse_ipv4(native->config.sender_gateway, &native->sender.gateway_ip) < 0)
    {
        rusteron_dpdk_set_error("sender IPv4 address or gateway is not a valid dotted quad");
        return -1;
    }
    if (rusteron_dpdk_parse_ipv4(native->config.receiver_ipv4, &native->receiver.local_ip) < 0 ||
        rusteron_dpdk_parse_ipv4(native->config.receiver_gateway, &native->receiver.gateway_ip) < 0)
    {
        rusteron_dpdk_set_error("receiver IPv4 address or gateway is not a valid dotted quad");
        return -1;
    }

    if (rusteron_dpdk_runtime_eal_up(native) < 0)
    {
        return -1;
    }

    if (rusteron_dpdk_init_port(native, &native->sender) < 0)
    {
        rusteron_dpdk_runtime_cleanup(native);
        return -1;
    }
    if (rusteron_dpdk_init_port(native, &native->receiver) < 0)
    {
        rusteron_dpdk_runtime_cleanup(native);
        return -1;
    }

    return 0;
}

/* Tear one role port down in reverse of its init order: stop, close, free pool. */
static void rusteron_dpdk_port_teardown(rusteron_dpdk_transport_t *native, rusteron_dpdk_port_t *p)
{
    const rusteron_dpdk_port_ops_t *ops = native->ops;
    if (p->started)
    {
        ops->dev_stop(p->port_id);
    }
    if (p->configured)
    {
        ops->dev_close(p->port_id);
    }
    if (NULL != p->mempool)
    {
        ops->mempool_free(p->mempool);
        p->mempool = NULL;
    }
}

/* Reverse of initialization order: the receiver is initialized second, so it is
 * torn down first, then the sender. */
void rusteron_dpdk_runtime_cleanup(rusteron_dpdk_transport_t *native)
{
    /* Release the Aeron counters back to the driver's counters manager first;
     * the manager outlives the transport and remains valid. */
    rusteron_dpdk_counters_free(&native->receiver.counters);
    rusteron_dpdk_counters_free(&native->sender.counters);

    rusteron_dpdk_port_teardown(native, &native->receiver);
    rusteron_dpdk_port_teardown(native, &native->sender);
    native->eal_up = 0;
}

int rusteron_dpdk_runtime_probe_device(
    rusteron_dpdk_transport_t *native, const char *pci_bdf, uint16_t *port_id)
{
    const rusteron_dpdk_config_t *cfg = &native->config;
    char message[512];

    if (NULL == pci_bdf || NULL == port_id)
    {
        rusteron_dpdk_set_error("probe_device requires non-NULL bdf and port_id");
        return -1;
    }

    int configured = (0 == strcmp(pci_bdf, cfg->sender_pci)) ||
                     (0 == strcmp(pci_bdf, cfg->receiver_pci));
    if (!configured)
    {
        snprintf(message, sizeof(message),
                 "device %s is not a configured ENA (only %s and %s are allow-listed)",
                 pci_bdf, cfg->sender_pci, cfg->receiver_pci);
        rusteron_dpdk_set_error(message);
        return -1;
    }

    if (native->ops->probe_port(pci_bdf, port_id) < 0)
    {
        snprintf(message, sizeof(message),
                 "device %s is allow-listed but not probed by DPDK", pci_bdf);
        rusteron_dpdk_set_error(message);
        return -1;
    }

    return 0;
}

void rusteron_dpdk_transport_test_dump(
    const rusteron_dpdk_transport_t *native,
    uint16_t *sender_port, uintptr_t *sender_pool,
    uint16_t *receiver_port, uintptr_t *receiver_pool)
{
    if (NULL != sender_port)
    {
        *sender_port = native->sender.port_id;
    }
    if (NULL != sender_pool)
    {
        *sender_pool = (uintptr_t)native->sender.mempool;
    }
    if (NULL != receiver_port)
    {
        *receiver_port = native->receiver.port_id;
    }
    if (NULL != receiver_pool)
    {
        *receiver_pool = (uintptr_t)native->receiver.mempool;
    }
}

/* Test-only ARP hooks (see internal.h). */
int rusteron_dpdk_transport_test_arp_seed(
    rusteron_dpdk_transport_t *native, const char *ip_str, const uint8_t mac[6])
{
    if (NULL == native || NULL == ip_str || NULL == mac)
    {
        rusteron_dpdk_set_error("arp_seed requires non-NULL transport, ip and mac");
        return -1;
    }

    uint32_t ip;
    if (rusteron_dpdk_parse_ipv4(ip_str, &ip) < 0)
    {
        rusteron_dpdk_set_error("arp_seed ip must be a valid dotted quad");
        return -1;
    }

    const uint32_t mask = RUSTERON_DPDK_ARP_TABLE_SIZE - 1;
    uint32_t h = ip;
    h = (h ^ (h >> 16)) * 0x85ebca6bu;
    h = (h ^ (h >> 13)) * 0xc2b2ae35u;
    size_t slot = (size_t)((h ^ (h >> 16)) & mask);

    rusteron_dpdk_arp_entry_t *e = NULL;
    for (size_t i = 0; i < RUSTERON_DPDK_ARP_TABLE_SIZE; i++)
    {
        rusteron_dpdk_arp_entry_t *candidate = &native->arp.entries[(slot + i) & mask];
        if (RUSTERON_DPDK_ARP_EMPTY == candidate->state || candidate->ip == ip)
        {
            e = candidate;
            break;
        }
    }
    if (NULL == e)
    {
        rusteron_dpdk_set_error("arp_seed: ARP table is full");
        return -1;
    }

    e->ip = ip;
    memcpy(e->mac, mac, RUSTERON_DPDK_ETH_ADDR_LEN);
    e->state = RUSTERON_DPDK_ARP_REACHABLE;
    e->last_seen_ms = rusteron_dpdk_clock_ms();
    return 0;
}

int rusteron_dpdk_transport_test_arp_rx(
    rusteron_dpdk_transport_t *native, int role, const uint8_t *frame, size_t frame_len)
{
    if (NULL == native)
    {
        rusteron_dpdk_set_error("arp_rx requires a non-NULL transport");
        return -1;
    }
    rusteron_dpdk_port_t *port =
        (RUSTERON_DPDK_ROLE_RECEIVER == role) ? &native->receiver : &native->sender;
    return rusteron_dpdk_arp_handle_frame(&native->arp, native, port, frame, frame_len);
}

void rusteron_dpdk_transport_test_rx_stats(
    const rusteron_dpdk_transport_t *native, rusteron_dpdk_rx_stats_t *out)
{
    if (NULL == native || NULL == out)
    {
        return;
    }
    *out = native->rx_stats;
}
