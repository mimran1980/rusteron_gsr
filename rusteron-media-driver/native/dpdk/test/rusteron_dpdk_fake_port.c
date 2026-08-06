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

#define RUSTERON_DPDK_FAKE_BUFFER_COUNT 256
#define RUSTERON_DPDK_FAKE_BUFFER_CAPACITY 2048
#define RUSTERON_DPDK_FAKE_POOL_AVAIL_DEFAULT 64
#define RUSTERON_DPDK_FAKE_TX_BURST_CAP_DEFAULT 64
#define RUSTERON_DPDK_FAKE_CAPTURE_MAX 1024
#define RUSTERON_DPDK_FAKE_PORTS 4
#define RUSTERON_DPDK_FAKE_RX_DEPTH 256

typedef struct rusteron_dpdk_fake_buffer_stct
{
    uint8_t storage[RUSTERON_DPDK_FAKE_BUFFER_CAPACITY];
    int used;
} rusteron_dpdk_fake_buffer_t;

static int rusteron_dpdk_fake_failure_step = 0;
static int rusteron_dpdk_fake_call_count = 0;
static char rusteron_dpdk_fake_driver[64] = "net_ena";
static int rusteron_dpdk_fake_csum_ok = 1;
static uint16_t rusteron_dpdk_fake_next_port = 0;
static uintptr_t rusteron_dpdk_fake_next_pool = 0x1000;
static int rusteron_dpdk_fake_log_len = 0;
static char rusteron_dpdk_fake_log[RUSTERON_DPDK_FAKE_LOG_MAX][RUSTERON_DPDK_FAKE_LOG_ENTRY_LEN];

/* Data path (plan §7.4): a pool of fixed-capacity buffers, a tx-burst accept
 * cap, live-mbuf accounting, and a frame capture for golden-vector asserts. */
static rusteron_dpdk_fake_buffer_t rusteron_dpdk_fake_buffers[RUSTERON_DPDK_FAKE_BUFFER_COUNT];
static int rusteron_dpdk_fake_pool_avail = RUSTERON_DPDK_FAKE_POOL_AVAIL_DEFAULT;
static uint16_t rusteron_dpdk_fake_tx_burst_cap = RUSTERON_DPDK_FAKE_TX_BURST_CAP_DEFAULT;
static int rusteron_dpdk_fake_allocated_count = 0;
static int rusteron_dpdk_fake_released_count = 0;
static int rusteron_dpdk_fake_capture_len = 0;
static rusteron_dpdk_fake_capture_t rusteron_dpdk_fake_capture[RUSTERON_DPDK_FAKE_CAPTURE_MAX];

/* RX path (plan §7.6): a per-port ring of injected frames that rx_burst drains
 * into freshly allocated mbuf views, so receive traffic flows through the same
 * pool and leak accounting as transmit. */
typedef struct rusteron_dpdk_fake_rx_entry_stct
{
    uint8_t data[RUSTERON_DPDK_FAKE_BUFFER_CAPACITY];
    uint32_t len;
    uint32_t rx_ol_flags;
    uint32_t nb_segs;
} rusteron_dpdk_fake_rx_entry_t;

static int rusteron_dpdk_fake_rx_head[RUSTERON_DPDK_FAKE_PORTS];
static int rusteron_dpdk_fake_rx_tail[RUSTERON_DPDK_FAKE_PORTS];
static rusteron_dpdk_fake_rx_entry_t rusteron_dpdk_fake_rx_q[RUSTERON_DPDK_FAKE_PORTS][RUSTERON_DPDK_FAKE_RX_DEPTH];

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

    memset(rusteron_dpdk_fake_buffers, 0, sizeof(rusteron_dpdk_fake_buffers));
    rusteron_dpdk_fake_pool_avail = RUSTERON_DPDK_FAKE_POOL_AVAIL_DEFAULT;
    rusteron_dpdk_fake_tx_burst_cap = RUSTERON_DPDK_FAKE_TX_BURST_CAP_DEFAULT;
    rusteron_dpdk_fake_allocated_count = 0;
    rusteron_dpdk_fake_released_count = 0;
    rusteron_dpdk_fake_capture_len = 0;
    memset(rusteron_dpdk_fake_rx_head, 0, sizeof(rusteron_dpdk_fake_rx_head));
    memset(rusteron_dpdk_fake_rx_tail, 0, sizeof(rusteron_dpdk_fake_rx_tail));
    memset(rusteron_dpdk_fake_rx_q, 0, sizeof(rusteron_dpdk_fake_rx_q));
}

void rusteron_dpdk_fake_set_tx_burst_cap(uint16_t n)
{
    rusteron_dpdk_fake_tx_burst_cap = n;
}

void rusteron_dpdk_fake_set_pool_avail(int n)
{
    rusteron_dpdk_fake_pool_avail = n;
}

int rusteron_dpdk_fake_capture_count(void)
{
    return rusteron_dpdk_fake_capture_len;
}

int rusteron_dpdk_fake_capture_at(int index, rusteron_dpdk_fake_capture_t *out)
{
    if (NULL == out || index < 0 || index >= rusteron_dpdk_fake_capture_len)
    {
        return -1;
    }
    *out = rusteron_dpdk_fake_capture[index];
    return 0;
}

int rusteron_dpdk_fake_allocated(void)
{
    return rusteron_dpdk_fake_allocated_count;
}

int rusteron_dpdk_fake_released(void)
{
    return rusteron_dpdk_fake_released_count;
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
    uint8_t mac[RUSTERON_DPDK_ETH_ADDR_LEN],
    char *driver_name, size_t driver_name_len,
    int *csum_offload_ok, int *ena_llq_available)
{
    rusteron_dpdk_fake_logf("info %s (port %u)", pci_bdf, port_id);
    if (rusteron_dpdk_fake_step() < 0)
    {
        return -1;
    }
    /* Deterministic, distinct locally-administered MACs: 02:00:00:00:00:01 for
     * the sender (port 0) and :02 for the receiver (port 1). */
    memset(mac, 0, RUSTERON_DPDK_ETH_ADDR_LEN);
    mac[0] = 0x02;
    mac[5] = (uint8_t)(port_id + 1);
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

/* Data path (plan §7.4). */

static void rusteron_dpdk_fake_release_buffer(rusteron_dpdk_mbuf_t *m)
{
    if (NULL != m && NULL != m->opaque)
    {
        /* opaque is a 1-based buffer index (see mbuf_alloc): 0 would alias NULL
         * and this guard would silently skip the release/count. */
        rusteron_dpdk_fake_buffers[(uintptr_t)m->opaque - 1].used = 0;
        m->opaque = NULL;
        rusteron_dpdk_fake_released_count++;
    }
}

static int rusteron_dpdk_fake_mbuf_alloc(void *mempool, rusteron_dpdk_mbuf_t *m)
{
    (void)mempool; /* one shared pool backing every role mempool in the fake */
    int live = rusteron_dpdk_fake_allocated_count - rusteron_dpdk_fake_released_count;
    if (live >= rusteron_dpdk_fake_pool_avail)
    {
        return -1; /* pool exhaustion */
    }

    for (int i = 0; i < RUSTERON_DPDK_FAKE_BUFFER_COUNT; i++)
    {
        if (!rusteron_dpdk_fake_buffers[i].used)
        {
            rusteron_dpdk_fake_buffers[i].used = 1;
            /* 1-based handle: index 0 would cast to NULL and defeat the fake's
             * own NULL-opaque release guard (and leak-accounting asserts). */
            m->opaque = (void *)(uintptr_t)(i + 1);
            m->data = rusteron_dpdk_fake_buffers[i].storage;
            m->capacity = RUSTERON_DPDK_FAKE_BUFFER_CAPACITY;
            m->frame_len = 0;
            m->ol_flags = 0;
            m->l2_len = 0;
            m->l3_len = 0;
            m->l4_len = 0;
            m->udp_pseudo_csum = 0;
            rusteron_dpdk_fake_allocated_count++;
            return 0;
        }
    }
    return -1;
}

static void rusteron_dpdk_fake_mbuf_release(rusteron_dpdk_mbuf_t *m)
{
    rusteron_dpdk_fake_release_buffer(m);
}

static uint16_t rusteron_dpdk_fake_tx_burst(
    uint16_t port_id, uint16_t tx_queue_id,
    rusteron_dpdk_mbuf_t **pkts, uint16_t nb)
{
    (void)tx_queue_id;
    uint16_t cap = rusteron_dpdk_fake_tx_burst_cap;
    uint16_t accepted = nb < cap ? nb : cap;

    /* Accepted prefix: the "NIC" owns these — record them and recycle the
     * buffers. Rejected tail: released, matching rte_eth_tx_burst semantics. */
    for (uint16_t i = 0; i < accepted; i++)
    {
        rusteron_dpdk_mbuf_t *m = pkts[i];
        if (rusteron_dpdk_fake_capture_len < RUSTERON_DPDK_FAKE_CAPTURE_MAX)
        {
            rusteron_dpdk_fake_capture_t *c = &rusteron_dpdk_fake_capture[rusteron_dpdk_fake_capture_len++];
            uint32_t len = m->frame_len;
            memset(c, 0, sizeof(*c));
            memcpy(c->data, m->data, len);
            c->len = len;
            c->ol_flags = m->ol_flags;
            c->l2_len = m->l2_len;
            c->l3_len = m->l3_len;
            c->l4_len = m->l4_len;
            c->udp_pseudo_csum = m->udp_pseudo_csum;
            c->port_id = port_id;
        }
        rusteron_dpdk_fake_release_buffer(m);
    }
    for (uint16_t i = accepted; i < nb; i++)
    {
        rusteron_dpdk_fake_release_buffer(pkts[i]);
    }

    return accepted;
}

int rusteron_dpdk_fake_rx_inject(
    uint16_t port_id, const uint8_t *frame, size_t len,
    uint32_t rx_ol_flags, uint32_t nb_segs)
{
    if (port_id >= RUSTERON_DPDK_FAKE_PORTS || NULL == frame ||
        len > RUSTERON_DPDK_FAKE_BUFFER_CAPACITY)
    {
        return -1;
    }
    int tail = rusteron_dpdk_fake_rx_tail[port_id];
    if ((tail + 1) % RUSTERON_DPDK_FAKE_RX_DEPTH == rusteron_dpdk_fake_rx_head[port_id])
    {
        return -1; /* queue full */
    }
    rusteron_dpdk_fake_rx_entry_t *entry = &rusteron_dpdk_fake_rx_q[port_id][tail];
    memcpy(entry->data, frame, len);
    entry->len = (uint32_t)len;
    entry->rx_ol_flags = rx_ol_flags;
    entry->nb_segs = nb_segs;
    rusteron_dpdk_fake_rx_tail[port_id] = (tail + 1) % RUSTERON_DPDK_FAKE_RX_DEPTH;
    return 0;
}

int rusteron_dpdk_fake_rx_queued(uint16_t port_id)
{
    if (port_id >= RUSTERON_DPDK_FAKE_PORTS)
    {
        return 0;
    }
    int head = rusteron_dpdk_fake_rx_head[port_id];
    int tail = rusteron_dpdk_fake_rx_tail[port_id];
    return (tail - head + RUSTERON_DPDK_FAKE_RX_DEPTH) % RUSTERON_DPDK_FAKE_RX_DEPTH;
}

static uint16_t rusteron_dpdk_fake_rx_burst(
    uint16_t port_id, uint16_t rx_queue_id,
    rusteron_dpdk_mbuf_t **pkts, uint16_t nb)
{
    (void)rx_queue_id;
    if (port_id >= RUSTERON_DPDK_FAKE_PORTS)
    {
        return 0;
    }

    uint16_t received = 0;
    while (received < nb &&
           rusteron_dpdk_fake_rx_head[port_id] != rusteron_dpdk_fake_rx_tail[port_id])
    {
        rusteron_dpdk_fake_rx_entry_t *entry = &rusteron_dpdk_fake_rx_q[port_id][rusteron_dpdk_fake_rx_head[port_id]];
        rusteron_dpdk_mbuf_t *m = pkts[received];
        if (rusteron_dpdk_fake_mbuf_alloc(NULL, m) < 0)
        {
            break; /* pool exhausted: deliver what we have */
        }
        memcpy(m->data, entry->data, entry->len);
        m->frame_len = entry->len;
        m->nb_segs = entry->nb_segs;
        m->rx_ol_flags = entry->rx_ol_flags;
        rusteron_dpdk_fake_rx_head[port_id] =
            (rusteron_dpdk_fake_rx_head[port_id] + 1) % RUSTERON_DPDK_FAKE_RX_DEPTH;
        received++;
    }
    return received;
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
        .mbuf_alloc = rusteron_dpdk_fake_mbuf_alloc,
        .mbuf_release = rusteron_dpdk_fake_mbuf_release,
        .tx_burst = rusteron_dpdk_fake_tx_burst,
        .rx_burst = rusteron_dpdk_fake_rx_burst,
    };
    return &ops;
}
