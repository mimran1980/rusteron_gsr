/*
 * rusteron-media-driver DPDK transport — Aeron counters (plan §9).
 *
 * A per-role-port counter set surfaced through the driver's counters manager.
 * The type IDs (75..=95) are the driver's reserved range; no other media-driver
 * code allocates in it. Every counter carries a label with the role, PCI BDF,
 * DPDK port, queue and direction so a cluster-admin dashboard can attribute a
 * value to a specific ENA without extra state.
 *
 * Hot path: `rusteron_dpdk_counters_add` is a single relaxed atomic add into a
 * value address resolved at registration — no lookup, no allocation, no syscall.
 * Allocations happen only at registration (one pass over the PMD xstats) and on
 * the 1 Hz extended-statistics sample, which reads the PMD's values into the
 * pre-resolved slots.
 */
#ifndef RUSTERON_DPDK_COUNTERS_H
#define RUSTERON_DPDK_COUNTERS_H

#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "rusteron_dpdk_port_ops.h" /* RUSTERON_DPDK_XSTAT_NAME_LEN */

/* Forward-declared; counters.c includes the real header for the manager
 * functions. The hot-path helpers only touch int64_t* addresses. */
typedef struct aeron_counters_manager_stct aeron_counters_manager_t;
struct rusteron_dpdk_port_stct; /* defined in rusteron_dpdk_internal.h, which
                                   includes this header first */

#ifdef __cplusplus
extern "C" {
#endif

/* Aeron counter type IDs (plan §9). */
#define RUSTERON_DPDK_TYPE_PORT_INFO                75
#define RUSTERON_DPDK_TYPE_TRANSPORT                76
#define RUSTERON_DPDK_TYPE_NOBUFS                   77
#define RUSTERON_DPDK_TYPE_TX_EAGAIN                78
#define RUSTERON_DPDK_TYPE_ERROR                    79
#define RUSTERON_DPDK_TYPE_PKTS                     82
#define RUSTERON_DPDK_TYPE_BYTES                    83
#define RUSTERON_DPDK_TYPE_MISSED_PACKETS           84
#define RUSTERON_DPDK_TYPE_ARP_MISS                 85
#define RUSTERON_DPDK_TYPE_RX_SENDER_DISCARD        86
#define RUSTERON_DPDK_TYPE_POLLER                   87
#define RUSTERON_DPDK_TYPE_QUEUE_DROP               88
#define RUSTERON_DPDK_TYPE_CHECKSUM_FAILURE         89
#define RUSTERON_DPDK_TYPE_FRAGMENTED               90
#define RUSTERON_DPDK_TYPE_MEMPOOL_AVAILABLE        91
#define RUSTERON_DPDK_TYPE_EXTENDED_STATS           92
#define RUSTERON_DPDK_TYPE_RX_UNSUPPORTED_ETHERTYPE 93
#define RUSTERON_DPDK_TYPE_RX_UNSUPPORTED_PROTOCOL  94
#define RUSTERON_DPDK_TYPE_RX_RECEIVER_DISCARD      95

/* A port with no configured extended stats is a no-op for the 1 Hz sample. */
#define RUSTERON_DPDK_MAX_XSTATS 256
#define RUSTERON_DPDK_XSTAT_SAMPLE_MS 1000

/* Fixed-counter kinds. Indexes into rusteron_dpdk_counters_t.id[]/addr[];
 * the type IDs, names and directions live in the table in counters.c. */
enum rusteron_dpdk_counter_kind
{
    RD_COUNTER_PORT_INFO = 0,        /* 75   port info (driver + MAC in label) */
    RD_COUNTER_TRANSPORT,            /* 76   open transports on this port */
    RD_COUNTER_TX_PKTS,              /* 82   datagrams sent */
    RD_COUNTER_TX_BYTES,             /* 83   payload bytes sent */
    RD_COUNTER_RX_PKTS,              /* 82   datagrams dispatched */
    RD_COUNTER_RX_BYTES,             /* 83   payload bytes dispatched */
    RD_COUNTER_NOBUFS,               /* 77   TX mbuf pool exhausted */
    RD_COUNTER_TX_EAGAIN,            /* 78   TX bursts truncated by NIC */
    RD_COUNTER_ERROR,                /* 79   rejected datagrams */
    RD_COUNTER_ARP_MISS,             /* 85   unresolved next hop */
    RD_COUNTER_DISCARD,              /* 86/95 role discard */
    RD_COUNTER_QUEUE_DROP,           /* 88   endpoint-map miss */
    RD_COUNTER_CHECKSUM,             /* 89   RX checksum failures */
    RD_COUNTER_FRAGMENTED,           /* 90   RX IP fragments */
    RD_COUNTER_MISSED_PACKETS,       /* 84   ENA missed RX (mirrored xstat) */
    RD_COUNTER_UNSUPPORTED_ETHERTYPE, /* 93   RX unsupported ethertype */
    RD_COUNTER_UNSUPPORTED_PROTOCOL, /* 94   RX unsupported IP protocol */
    RD_COUNTER_POLLER,               /* 87   frames processed per poll */
    RD_COUNTER_MEMPOOL_AVAILABLE,    /* 91   free mbufs (1 Hz) */
    RD_COUNTER_COUNT
};

/* One mirrored PMD extended stat (type 92). */
typedef struct rusteron_dpdk_xstat_stct
{
    int32_t id;      /* aeron counter id, or -1 */
    int64_t *addr;   /* resolved value slot, or NULL */
    char *name;      /* cached PMD name (points into xstat_names) */
} rusteron_dpdk_xstat_t;

/* Per-port counter set, embedded in rusteron_dpdk_port_t. */
typedef struct rusteron_dpdk_counters_stct
{
    int registered;              /* counters allocated for this port */
    aeron_counters_manager_t *manager; /* borrowed; for free() */
    int32_t id[RD_COUNTER_COUNT];
    int64_t *addr[RD_COUNTER_COUNT];

    /* Extended statistics (type 92): one Aeron counter per PMD xstat, cached at
     * registration so the 1 Hz sample is a plain store into pre-resolved
     * addresses (plan §9). */
    uint32_t xstat_count;
    rusteron_dpdk_xstat_t *xstats;
    char *xstat_names;           /* backing store for name pointers */
    uint32_t missed_xstat_index; /* xstat whose name contains "missed" (UINT32_MAX = none) */
    uint64_t last_sample_ms;
} rusteron_dpdk_counters_t;

/* Hot-path bump: a single relaxed atomic add into a pre-resolved value slot.
 * NULL-safe so a port without a counters manager (e.g. a NULL-context test) is
 * a no-op. */
static inline void rusteron_dpdk_counters_add(rusteron_dpdk_counters_t *c, int kind, int64_t n)
{
    if (NULL != c && NULL != c->addr[kind])
    {
        __atomic_add_fetch(c->addr[kind], n, __ATOMIC_RELAXED);
    }
}

/* Allocate every fixed counter plus the PMD xstat mirrors for `port` from
 * `manager`. Best effort: a missing counters manager (or a NULL context) is a
 * no-op returning 0, and a failed single allocation leaves that counter at -1
 * (its bumps are then no-ops) without failing the transport. */
int rusteron_dpdk_counters_register(
    rusteron_dpdk_counters_t *c, aeron_counters_manager_t *manager,
    const struct rusteron_dpdk_port_stct *port, const struct rusteron_dpdk_port_ops_stct *ops);

/* Release every allocated counter back to the manager (no-op when nothing was
 * registered). */
void rusteron_dpdk_counters_free(rusteron_dpdk_counters_t *c);

/* Refresh the mempool-availability counter and the PMD xstat mirrors, at most
 * once per second (plan §9: no metrics thread). Called from the owning agent's
 * hot path (transport_send / poller_receive); the gate makes it cheap. */
void rusteron_dpdk_counters_sample(
    rusteron_dpdk_counters_t *c, const struct rusteron_dpdk_port_stct *port,
    const struct rusteron_dpdk_port_ops_stct *ops);

#ifdef __cplusplus
}
#endif

#endif /* RUSTERON_DPDK_COUNTERS_H */
