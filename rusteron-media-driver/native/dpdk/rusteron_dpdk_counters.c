/*
 * rusteron-media-driver DPDK transport — Aeron counters (plan §9).
 *
 * Registration resolves every counter's value address up front; the hot path
 * (transport.c / poller.c) only ever calls the inline `rusteron_dpdk_counters_add`.
 * Extended statistics are cached at registration (names for the labels, ids for
 * the sample) and refreshed at most once per second by
 * `rusteron_dpdk_counters_sample`, which the owning agent's hot path invokes
 * behind a millisecond gate — no metrics thread is ever created.
 */
#include "rusteron_dpdk_internal.h" /* port struct + rusteron_dpdk_clock_ms */
#include "rusteron_dpdk_counters.h"

#include <inttypes.h>
#include <stdio.h>
#include <string.h>

#include "concurrent/aeron_counters_manager.h"
#include "aeron_alloc.h"

/* Fixed-counter metadata: type ID, base name and direction for the label. The
 * discard counter's type ID is role-dependent (86 sender / 95 receiver). */
typedef struct rusteron_dpdk_counter_spec_stct
{
    int32_t type_id;
    const char *name;
    const char *dir;
} rusteron_dpdk_counter_spec_t;

static const rusteron_dpdk_counter_spec_t rusteron_dpdk_counter_specs[RD_COUNTER_COUNT] =
{
    [RD_COUNTER_PORT_INFO] = { RUSTERON_DPDK_TYPE_PORT_INFO, "port-info", "-" },
    [RD_COUNTER_TRANSPORT] = { RUSTERON_DPDK_TYPE_TRANSPORT, "transport", "-" },
    [RD_COUNTER_TX_PKTS] = { RUSTERON_DPDK_TYPE_PKTS, "pkts", "tx" },
    [RD_COUNTER_TX_BYTES] = { RUSTERON_DPDK_TYPE_BYTES, "bytes", "tx" },
    [RD_COUNTER_RX_PKTS] = { RUSTERON_DPDK_TYPE_PKTS, "pkts", "rx" },
    [RD_COUNTER_RX_BYTES] = { RUSTERON_DPDK_TYPE_BYTES, "bytes", "rx" },
    [RD_COUNTER_NOBUFS] = { RUSTERON_DPDK_TYPE_NOBUFS, "nobufs", "tx" },
    [RD_COUNTER_TX_EAGAIN] = { RUSTERON_DPDK_TYPE_TX_EAGAIN, "tx-eagain", "tx" },
    [RD_COUNTER_ERROR] = { RUSTERON_DPDK_TYPE_ERROR, "error", "-" },
    [RD_COUNTER_ARP_MISS] = { RUSTERON_DPDK_TYPE_ARP_MISS, "arp-miss", "tx" },
    [RD_COUNTER_DISCARD] = { RUSTERON_DPDK_TYPE_RX_RECEIVER_DISCARD, "discard", "rx" },
    [RD_COUNTER_QUEUE_DROP] = { RUSTERON_DPDK_TYPE_QUEUE_DROP, "queue-drop", "rx" },
    [RD_COUNTER_CHECKSUM] = { RUSTERON_DPDK_TYPE_CHECKSUM_FAILURE, "checksum-failure", "rx" },
    [RD_COUNTER_FRAGMENTED] = { RUSTERON_DPDK_TYPE_FRAGMENTED, "fragmented", "rx" },
    [RD_COUNTER_MISSED_PACKETS] = { RUSTERON_DPDK_TYPE_MISSED_PACKETS, "missed-packets", "rx" },
    [RD_COUNTER_UNSUPPORTED_ETHERTYPE] =
        { RUSTERON_DPDK_TYPE_RX_UNSUPPORTED_ETHERTYPE, "unsupported-ethertype", "rx" },
    [RD_COUNTER_UNSUPPORTED_PROTOCOL] =
        { RUSTERON_DPDK_TYPE_RX_UNSUPPORTED_PROTOCOL, "unsupported-protocol", "rx" },
    [RD_COUNTER_POLLER] = { RUSTERON_DPDK_TYPE_POLLER, "poller", "rx" },
    [RD_COUNTER_MEMPOOL_AVAILABLE] =
        { RUSTERON_DPDK_TYPE_MEMPOOL_AVAILABLE, "mempool-available", "-" },
};

static void rusteron_dpdk_counters_label(
    char *label, size_t label_len, const rusteron_dpdk_port_t *port,
    const char *name, const char *dir)
{
    const char *role = port->role == RUSTERON_DPDK_ROLE_SENDER ? "sender" : "receiver";
    int32_t queue = (dir[0] == 't') ? port->tx_queue_id : port->rx_queue_id;
    snprintf(
        label, label_len, "rusteron-dpdk role=%s bdf=%s port=%u q=%" PRId32 " dir=%s %s",
        role, port->pci, (unsigned)port->port_id, queue, dir, name);
}

/* Cache the PMD's extended stats: one Aeron counter (type 92) per xstat, names
 * copied for the labels, plus an initial sample so the counters read a value
 * before the first 1 Hz tick. */
static void rusteron_dpdk_counters_cache_xstats(
    rusteron_dpdk_counters_t *c, const rusteron_dpdk_port_t *port,
    const rusteron_dpdk_port_ops_t *ops)
{
    if (NULL == ops || NULL == ops->xstats_count || NULL == ops->xstats_names ||
        NULL == ops->xstats_get)
    {
        return;
    }

    const uint32_t n = ops->xstats_count(port->port_id);
    if (n == 0 || n > RUSTERON_DPDK_MAX_XSTATS)
    {
        return;
    }

    const size_t name_bytes = (size_t)n * RUSTERON_DPDK_XSTAT_NAME_LEN;
    char *names = NULL;
    rusteron_dpdk_xstat_t *xstats = NULL;
    if (aeron_alloc((void **)&names, name_bytes) < 0 ||
        aeron_alloc((void **)&xstats, (size_t)n * sizeof(rusteron_dpdk_xstat_t)) < 0)
    {
        aeron_free(names);
        aeron_free(xstats);
        return;
    }
    memset(names, 0, name_bytes);
    memset(xstats, 0, (size_t)n * sizeof(rusteron_dpdk_xstat_t));

    if (ops->xstats_names(port->port_id, names, n) < 0)
    {
        aeron_free(names);
        aeron_free(xstats);
        return;
    }

    uint64_t values[RUSTERON_DPDK_MAX_XSTATS];
    if (ops->xstats_get(port->port_id, values, n) != 0)
    {
        aeron_free(names);
        aeron_free(xstats);
        return;
    }

    for (uint32_t i = 0; i < n; i++)
    {
        xstats[i].name = names + ((size_t)i * RUSTERON_DPDK_XSTAT_NAME_LEN);
        xstats[i].id = aeron_counters_manager_allocate(
            c->manager, RUSTERON_DPDK_TYPE_EXTENDED_STATS, NULL, 0,
            xstats[i].name, strlen(xstats[i].name));
        if (xstats[i].id >= 0)
        {
            xstats[i].addr = aeron_counters_manager_addr(c->manager, xstats[i].id);
            __atomic_store_n(xstats[i].addr, (int64_t)values[i], __ATOMIC_RELAXED);
        }
        /* ENA missed RX packets (type 84): mirror whichever xstat names itself
         * "missed" (the ENA PMD exposes rx_missed / ena_*_rx_missed variants). */
        if (UINT32_MAX == c->missed_xstat_index &&
            NULL != strstr(xstats[i].name, "missed"))
        {
            c->missed_xstat_index = i;
        }
    }

    c->xstat_count = n;
    c->xstats = xstats;
    c->xstat_names = names;
}

int rusteron_dpdk_counters_register(
    rusteron_dpdk_counters_t *c, aeron_counters_manager_t *manager,
    const rusteron_dpdk_port_t *port, const rusteron_dpdk_port_ops_t *ops)
{
    char label[256];

    if (NULL == c || NULL == port)
    {
        return -1;
    }
    if (c->registered || NULL == manager)
    {
        return 0; /* already registered, or no manager to register into */
    }

    c->manager = manager;
    c->missed_xstat_index = UINT32_MAX;

    for (int32_t kind = 0; kind < RD_COUNTER_COUNT; kind++)
    {
        int32_t type_id = rusteron_dpdk_counter_specs[kind].type_id;
        const char *name = rusteron_dpdk_counter_specs[kind].name;
        const char *dir = rusteron_dpdk_counter_specs[kind].dir;

        if (RD_COUNTER_DISCARD == kind)
        {
            type_id = (port->role == RUSTERON_DPDK_ROLE_SENDER)
                ? RUSTERON_DPDK_TYPE_RX_SENDER_DISCARD
                : RUSTERON_DPDK_TYPE_RX_RECEIVER_DISCARD;
        }

        if (RD_COUNTER_PORT_INFO == kind)
        {
            /* port-info carries the driver + MAC in its label. */
            rusteron_dpdk_counters_label(label, sizeof(label), port, name, dir);
            size_t off = strlen(label);
            snprintf(
                label + off, sizeof(label) - off,
                " driver=%s mac=%02x:%02x:%02x:%02x:%02x:%02x",
                port->driver[0] != '\0' ? port->driver : "-",
                port->mac[0], port->mac[1], port->mac[2],
                port->mac[3], port->mac[4], port->mac[5]);
        }
        else
        {
            rusteron_dpdk_counters_label(label, sizeof(label), port, name, dir);
        }

        c->id[kind] = aeron_counters_manager_allocate(
            c->manager, type_id, NULL, 0, label, strlen(label));
        c->addr[kind] = c->id[kind] >= 0
            ? aeron_counters_manager_addr(c->manager, c->id[kind])
            : NULL;
    }

    rusteron_dpdk_counters_cache_xstats(c, port, ops);

    if (NULL != ops && NULL != ops->mempool_avail && NULL != port->mempool &&
        NULL != c->addr[RD_COUNTER_MEMPOOL_AVAILABLE])
    {
        __atomic_store_n(
            c->addr[RD_COUNTER_MEMPOOL_AVAILABLE],
            (int64_t)ops->mempool_avail(port->mempool), __ATOMIC_RELAXED);
    }
    if (NULL != c->addr[RD_COUNTER_PORT_INFO])
    {
        __atomic_store_n(c->addr[RD_COUNTER_PORT_INFO], 1, __ATOMIC_RELAXED);
    }

    c->last_sample_ms = rusteron_dpdk_clock_ms();
    c->registered = 1;
    return 0;
}

void rusteron_dpdk_counters_free(rusteron_dpdk_counters_t *c)
{
    if (NULL == c || !c->registered || NULL == c->manager)
    {
        return;
    }

    for (int32_t kind = 0; kind < RD_COUNTER_COUNT; kind++)
    {
        if (c->id[kind] >= 0)
        {
            aeron_counters_manager_free(c->manager, c->id[kind]);
            c->id[kind] = -1;
            c->addr[kind] = NULL;
        }
    }
    if (NULL != c->xstats)
    {
        for (uint32_t i = 0; i < c->xstat_count; i++)
        {
            if (c->xstats[i].id >= 0)
            {
                aeron_counters_manager_free(c->manager, c->xstats[i].id);
            }
        }
    }
    aeron_free(c->xstats);
    aeron_free(c->xstat_names);
    c->xstats = NULL;
    c->xstat_names = NULL;
    c->xstat_count = 0;
    c->missed_xstat_index = UINT32_MAX;
    c->registered = 0;
    c->manager = NULL;
}

void rusteron_dpdk_counters_sample(
    rusteron_dpdk_counters_t *c, const rusteron_dpdk_port_t *port,
    const rusteron_dpdk_port_ops_t *ops)
{
    if (NULL == c || !c->registered || NULL == port || NULL == ops)
    {
        return;
    }

    const uint64_t now = rusteron_dpdk_clock_ms();
    if (now < c->last_sample_ms || now - c->last_sample_ms < RUSTERON_DPDK_XSTAT_SAMPLE_MS)
    {
        return; /* at most once per second (plan §9) */
    }
    c->last_sample_ms = now;

    if (NULL != ops->mempool_avail && NULL != port->mempool &&
        NULL != c->addr[RD_COUNTER_MEMPOOL_AVAILABLE])
    {
        __atomic_store_n(
            c->addr[RD_COUNTER_MEMPOOL_AVAILABLE],
            (int64_t)ops->mempool_avail(port->mempool), __ATOMIC_RELAXED);
    }

    if (c->xstat_count > 0 && NULL != ops->xstats_get)
    {
        uint64_t values[RUSTERON_DPDK_MAX_XSTATS];
        if (0 == ops->xstats_get(port->port_id, values, c->xstat_count))
        {
            for (uint32_t i = 0; i < c->xstat_count; i++)
            {
                if (NULL != c->xstats[i].addr)
                {
                    __atomic_store_n(c->xstats[i].addr, (int64_t)values[i], __ATOMIC_RELAXED);
                }
            }
            if (UINT32_MAX != c->missed_xstat_index &&
                NULL != c->addr[RD_COUNTER_MISSED_PACKETS])
            {
                __atomic_store_n(
                    c->addr[RD_COUNTER_MISSED_PACKETS],
                    (int64_t)values[c->missed_xstat_index], __ATOMIC_RELAXED);
            }
        }
    }
}
