/*
 * rusteron-media-driver DPDK transport — fake port ops for tests.
 *
 * Implements the same symbol rusteron_dpdk_port_ops_real() that the production
 * rusteron_dpdk_port.c defines, so the linker resolves the runtime's ops lookup
 * to this table in test binaries. Drives the full init/teardown sequence with
 * a call log (for reverse-order teardown assertions) and a failure-step counter
 * (1..=18: 1-9 sender steps, 10-18 receiver steps) so every failure path in the
 * runtime is reachable deterministically without libdpdk.
 */
#include "rusteron_dpdk_fake.h"

#include <stdarg.h>
#include <stdio.h>
#include <string.h>

#define RUSTERON_DPDK_FAKE_LOG_MAX 64
#define RUSTERON_DPDK_FAKE_LOG_ENTRY_LEN 128

static int rusteron_dpdk_fake_failure_step = 0;
static int rusteron_dpdk_fake_call_count = 0;
static char rusteron_dpdk_fake_driver[64] = "net_ena";
static int rusteron_dpdk_fake_csum_ok = 1;
static uint16_t rusteron_dpdk_fake_next_port = 0;
static uintptr_t rusteron_dpdk_fake_next_pool = 0x1000;
static int rusteron_dpdk_fake_log_len = 0;
static char rusteron_dpdk_fake_log[RUSTERON_DPDK_FAKE_LOG_MAX][RUSTERON_DPDK_FAKE_LOG_ENTRY_LEN];

static void rusteron_dpdk_fake_logf(const char *fmt, ...)
{
    if (rusteron_dpdk_fake_log_len >= RUSTERON_DPDK_FAKE_LOG_MAX)
    {
        return;
    }
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(rusteron_dpdk_fake_log[rusteron_dpdk_fake_log_len++],
              RUSTERON_DPDK_FAKE_LOG_ENTRY_LEN, fmt, ap);
    va_end(ap);
}

/* Advance the call counter; fail when the configured step is reached. */
static int rusteron_dpdk_fake_step(void)
{
    rusteron_dpdk_fake_call_count++;
    if (rusteron_dpdk_fake_failure_step != 0 &&
        rusteron_dpdk_fake_call_count == rusteron_dpdk_fake_failure_step)
    {
        return -1;
    }
    return 0;
}

void rusteron_dpdk_fake_reset(void)
{
    rusteron_dpdk_fake_failure_step = 0;
    rusteron_dpdk_fake_call_count = 0;
    strcpy(rusteron_dpdk_fake_driver, "net_ena");
    rusteron_dpdk_fake_csum_ok = 1;
    rusteron_dpdk_fake_next_port = 0;
    rusteron_dpdk_fake_next_pool = 0x1000;
    rusteron_dpdk_fake_log_len = 0;
}

void rusteron_dpdk_fake_set_failure(int step)
{
    rusteron_dpdk_fake_failure_step = step;
}

void rusteron_dpdk_fake_set_driver(const char *driver)
{
    snprintf(rusteron_dpdk_fake_driver, sizeof(rusteron_dpdk_fake_driver), "%s", driver);
}

void rusteron_dpdk_fake_set_csum_ok(int ok)
{
    rusteron_dpdk_fake_csum_ok = ok;
}

int rusteron_dpdk_fake_log_count(void)
{
    return rusteron_dpdk_fake_log_len;
}

void rusteron_dpdk_fake_log_at(int index, char *buf, size_t buflen)
{
    if (index >= 0 && index < rusteron_dpdk_fake_log_len)
    {
        snprintf(buf, buflen, "%s", rusteron_dpdk_fake_log[index]);
    }
    else if (buflen > 0)
    {
        buf[0] = '\0';
    }
}

static int rusteron_dpdk_fake_probe_port(const char *pci_bdf, uint16_t *port_id)
{
    rusteron_dpdk_fake_logf("probe %s", pci_bdf);
    if (rusteron_dpdk_fake_step() < 0)
    {
        return -1;
    }
    *port_id = rusteron_dpdk_fake_next_port++;
    return 0;
}

static int rusteron_dpdk_fake_dev_info(
    uint16_t port_id, const char *pci_bdf,
    char *driver_name, size_t driver_name_len,
    int *csum_offload_ok, int *ena_llq_available)
{
    rusteron_dpdk_fake_logf("info %s (port %u)", pci_bdf, port_id);
    if (rusteron_dpdk_fake_step() < 0)
    {
        return -1;
    }
    snprintf(driver_name, driver_name_len, "%s", rusteron_dpdk_fake_driver);
    *csum_offload_ok = rusteron_dpdk_fake_csum_ok;
    *ena_llq_available = 0;
    return 0;
}

static int rusteron_dpdk_fake_dev_configure(
    uint16_t port_id, uint16_t rx_rings, uint16_t tx_rings,
    uint64_t rx_offloads, uint64_t tx_offloads)
{
    (void)rx_offloads;
    (void)tx_offloads;
    rusteron_dpdk_fake_logf("configure %u (rxr=%u txr=%u)", port_id, rx_rings, tx_rings);
    return rusteron_dpdk_fake_step();
}

static void *rusteron_dpdk_fake_mempool_create(const char *name, uint32_t n, uint16_t cache_size)
{
    (void)cache_size;
    rusteron_dpdk_fake_logf("pool %s (%u)", name, n);
    if (rusteron_dpdk_fake_step() < 0)
    {
        return NULL;
    }
    rusteron_dpdk_fake_next_pool += 0x10;
    return (void *)rusteron_dpdk_fake_next_pool;
}

static int rusteron_dpdk_fake_rx_queue_setup(
    uint16_t port_id, uint16_t queue_id, uint16_t nb_desc, void *mempool)
{
    (void)mempool;
    rusteron_dpdk_fake_logf("rx %u (q=%u desc=%u)", port_id, queue_id, nb_desc);
    return rusteron_dpdk_fake_step();
}

static int rusteron_dpdk_fake_tx_queue_setup(
    uint16_t port_id, uint16_t queue_id, uint16_t nb_desc)
{
    rusteron_dpdk_fake_logf("tx %u (q=%u desc=%u)", port_id, queue_id, nb_desc);
    return rusteron_dpdk_fake_step();
}

static int rusteron_dpdk_fake_dev_set_mtu(uint16_t port_id, uint16_t mtu)
{
    rusteron_dpdk_fake_logf("mtu %u (%u)", port_id, mtu);
    return rusteron_dpdk_fake_step();
}

static int rusteron_dpdk_fake_dev_start(uint16_t port_id)
{
    rusteron_dpdk_fake_logf("start %u", port_id);
    return rusteron_dpdk_fake_step();
}

static int rusteron_dpdk_fake_link_wait_ready(uint16_t port_id, uint32_t timeout_ms)
{
    (void)timeout_ms;
    rusteron_dpdk_fake_logf("link %u", port_id);
    return rusteron_dpdk_fake_step();
}

static int rusteron_dpdk_fake_dev_stop(uint16_t port_id)
{
    rusteron_dpdk_fake_logf("stop %u", port_id);
    return 0;
}

static int rusteron_dpdk_fake_dev_close(uint16_t port_id)
{
    rusteron_dpdk_fake_logf("close %u", port_id);
    return 0;
}

static void rusteron_dpdk_fake_mempool_free(void *mempool)
{
    rusteron_dpdk_fake_logf("free %p", mempool);
}

rusteron_dpdk_port_ops_t *rusteron_dpdk_port_ops_real(void)
{
    static rusteron_dpdk_port_ops_t ops = {
        .probe_port = rusteron_dpdk_fake_probe_port,
        .dev_info = rusteron_dpdk_fake_dev_info,
        .dev_configure = rusteron_dpdk_fake_dev_configure,
        .mempool_create = rusteron_dpdk_fake_mempool_create,
        .rx_queue_setup = rusteron_dpdk_fake_rx_queue_setup,
        .tx_queue_setup = rusteron_dpdk_fake_tx_queue_setup,
        .dev_set_mtu = rusteron_dpdk_fake_dev_set_mtu,
        .dev_start = rusteron_dpdk_fake_dev_start,
        .link_wait_ready = rusteron_dpdk_fake_link_wait_ready,
        .dev_stop = rusteron_dpdk_fake_dev_stop,
        .dev_close = rusteron_dpdk_fake_dev_close,
        .mempool_free = rusteron_dpdk_fake_mempool_free,
    };
    return &ops;
}
