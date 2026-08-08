/*
 * rusteron-media-driver DPDK transport — internal shared definitions.
 *
 * Includes the opaque transport handle's full layout, the role-port state, and
 * the seams the runtime goes through for EAL initialization and port
 * operations. This header is internal to the native DPDK sources and the test
 * fakes; it is not part of the Rust/native ABI.
 */
#ifndef RUSTERON_DPDK_INTERNAL_H
#define RUSTERON_DPDK_INTERNAL_H

/* clock_gettime/CLOCK_MONOTONIC (runtime.c), inet_pton (transport.c) and
 * htonl/htons (packet.c) are POSIX-199309/200112 symbols glibc hides under
 * strict -std=c11 without a feature macro. Every native DPDK source includes
 * this header first, so declaring it here (before any system include) covers
 * them uniformly. The vendored Aeron util files compiled alongside declare
 * their own macro and must not receive a -D from the build script. */
#define _POSIX_C_SOURCE 200809L

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "rusteron_dpdk_transport.h"
#include "rusteron_dpdk_port_ops.h"
#include "rusteron_dpdk_counters.h"
#include "rusteron_dpdk_arp.h"
#include "rusteron_dpdk_endpoint_map.h"

/* Aeron types for the poller callbacks and the shared receive loop (poller.c):
 * the affinity enum, the transport struct, the recv/recvmmsg function typedefs
 * and the (forward-declared) poller struct. transport.c already pulls these
 * transitively via media/aeron_udp_channel_transport.h; declaring them here
 * keeps poller.c self-contained. */
#include "media/aeron_udp_channel_transport_bindings.h"

#ifdef __cplusplus
extern "C" {
#endif

#define RUSTERON_DPDK_ROLE_SENDER 0
#define RUSTERON_DPDK_ROLE_RECEIVER 1

/* EAL modes selected by rusteron_dpdk_test_set_eal_mode (production = REAL). */
#define RUSTERON_DPDK_EAL_REAL 0    /* --huge-dir <configured path> */
#define RUSTERON_DPDK_EAL_NO_HUGE 1 /* --no-huge (tests without hugetlbfs) */
#define RUSTERON_DPDK_EAL_SKIP 2    /* skip the EAL seam entirely (tests) */

/* A canonical PCI BDF contains ':'; anything else is a virtual-device name
 * (test/TAP path, plan §11.2). Shared by the EAL argv builder and the port
 * bring-up checks so the two never disagree on the mode of a selector. */
static inline int rusteron_dpdk_selector_is_pci(const char *sel)
{
    return NULL != sel && NULL != strchr(sel, ':');
}

/* Transport-level offload bits passed to the port ops; the real port
 * implementation maps them onto the RTE_ETH_RX/TX_OFFLOAD_* masks. */
#define RUSTERON_DPDK_RX_OFFLOAD_IPV4_CKSUM (1ULL << 0)
#define RUSTERON_DPDK_RX_OFFLOAD_UDP_CKSUM (1ULL << 1)
#define RUSTERON_DPDK_TX_OFFLOAD_IPV4_CKSUM (1ULL << 0)
#define RUSTERON_DPDK_TX_OFFLOAD_UDP_CKSUM (1ULL << 1)

/* Error codes are defined in rusteron_dpdk_transport.h (public ABI). */

typedef struct rusteron_dpdk_port_stct
{
    uint8_t role;                 /* RUSTERON_DPDK_ROLE_* */
    char pci[16];                 /* dddd:bb:ss.f */
    char local_ipv4[16];          /* dotted quad */
    uint8_t prefix_len;           /* 1..=32 */
    char gateway_ipv4[16];        /* dotted quad */
    uint16_t port_id;             /* DPDK port id once probed */
    void *mempool;                /* opaque rte_mempool* (or fake handle) */
    uint16_t rx_queue_id;
    uint16_t tx_queue_id;
    int configured;               /* dev_configure succeeded */
    int started;                  /* dev_start succeeded */
    int link_up;                  /* link_wait_ready succeeded */
    int csum_offload_ok;          /* dev_info: IPv4/UDP checksum offloads */
    int ena_llq_available;        /* dev_info: ENA LLQ/write-combining */
    uint8_t mac[RUSTERON_DPDK_ETH_ADDR_LEN]; /* dev_info MAC, byte order */
    uint32_t local_ip;            /* parsed local IPv4, network order */
    uint32_t gateway_ip;          /* parsed gateway IPv4, network order */
    char driver[64];              /* dev_info PMD name (labels, port-info) */
    rusteron_dpdk_counters_t counters; /* Aeron counters (plan §9) */
} rusteron_dpdk_port_t;

/* Per-channel state attached to an Aeron transport via bindings_clientd
 * (transport.c). One instance per initialized Aeron transport/endpoint. */
typedef struct rusteron_dpdk_client_stct
{
    rusteron_dpdk_transport_t *runtime; /* owning native runtime (singleton) */
    rusteron_dpdk_port_t *port;         /* sender or receiver ENA by affinity */
    int affinity;                       /* AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_* */
    uint16_t local_udp_port;            /* local bind port, host order */
    size_t mtu;                         /* per-channel Aeron MTU cap (bytes) */
    struct sockaddr_storage connected_address; /* owned copy for NULL-address sends */
} rusteron_dpdk_client_t;

/* Receive counters (plan §7.6): one bucket per reject class, incremented in the
 * poll hot path. Surfaced as Aeron counters in Ticket 7. */
typedef struct rusteron_dpdk_rx_stats_stct
{
    uint64_t accepted;
    uint64_t arp;
    uint64_t ipv6;
    uint64_t multicast;
    uint64_t ethertype;
    uint64_t vlan;
    uint64_t ip_options;
    uint64_t fragment;
    uint64_t truncated;
    uint64_t protocol;
    uint64_t checksum;
    uint64_t multi_segment;
    uint64_t foreign_dst;
    uint64_t unknown_port;
} rusteron_dpdk_rx_stats_t;

struct rusteron_dpdk_transport_stct
{
    rusteron_dpdk_config_t config;
    const rusteron_dpdk_port_ops_t *ops; /* active port ops (real or fake) */
    int eal_up;                          /* this transport owns EAL up */
    rusteron_dpdk_port_t sender;
    rusteron_dpdk_port_t receiver;
    rusteron_dpdk_arp_table_t arp;       /* shared next-hop cache (plan §7.5) */
    rusteron_dpdk_rx_stats_t rx_stats;   /* receive counters (plan §7.6) */
};

/* EAL seam — implemented by rusteron_dpdk_eal.c (real rte_eal) in production
 * and by test/rusteron_dpdk_fake_eal.c in test builds. */
typedef struct rusteron_dpdk_eal_params_stct
{
    const rusteron_dpdk_config_t *config;
    int mode; /* RUSTERON_DPDK_EAL_* */
} rusteron_dpdk_eal_params_t;

int rusteron_dpdk_eal_init(const rusteron_dpdk_eal_params_t *params, char *errbuf, size_t errlen);
int rusteron_dpdk_eal_is_initialized(void);

/* Register/unregister the calling thread with DPDK (plan §7.2: "Register Aeron
 * network threads with DPDK"). Real impl in rusteron_dpdk_eal.c, no-op in
 * test/rusteron_dpdk_fake_eal.c. Both are safe on already-registered and EAL
 * threads. */
int rusteron_dpdk_eal_thread_register(void);
int rusteron_dpdk_eal_thread_unregister(void);

/* Runtime orchestration (rusteron_dpdk_runtime.c). */
int rusteron_dpdk_runtime_init(rusteron_dpdk_transport_t *native);
void rusteron_dpdk_runtime_cleanup(rusteron_dpdk_transport_t *native);

/* Resolve a device BDF against the allow-list. Only the two configured ENA
 * devices may be opened; anything else is rejected (plan §7.2). */
int rusteron_dpdk_runtime_probe_device(
    rusteron_dpdk_transport_t *native, const char *pci_bdf, uint16_t *port_id);

/* Test hooks (runtime.c). Production behaviour is the default (REAL, no
 * override, reset is a no-op for a fresh process). */
void rusteron_dpdk_test_reset(void);
void rusteron_dpdk_test_set_eal_mode(int mode);
void rusteron_dpdk_set_port_ops(const rusteron_dpdk_port_ops_t *ops);

/* Test reset for the transport.c singleton (cleared by rusteron_dpdk_test_reset). */
void rusteron_dpdk_transport_test_reset(void);

/* Monotonic milliseconds clock (runtime.c). Tests pin it with the setter so
 * ARP rate limiting and expiry are deterministic; reset clears the pin. */
uint64_t rusteron_dpdk_clock_ms(void);
void rusteron_dpdk_test_set_clock_ms(uint64_t ms);

/* The process-lifetime native runtime singleton (transport.c), or NULL when no
 * transport has been created. EAL can be initialized once, so there is at most
 * one. */
rusteron_dpdk_transport_t *rusteron_dpdk_active_runtime(void);

/* Test inspection of the two role ports (distinct ports / mempools). */
void rusteron_dpdk_transport_test_dump(
    const rusteron_dpdk_transport_t *native,
    uint16_t *sender_port, uintptr_t *sender_pool,
    uint16_t *receiver_port, uintptr_t *receiver_pool);

/* Test-only ARP helpers (runtime.c) so the tx tests can drive the next-hop
 * cache directly: seed a reachable entry, or feed an incoming ARP frame through
 * the handler as if it arrived on the given role (0 = sender, 1 = receiver). */
int rusteron_dpdk_transport_test_arp_seed(
    rusteron_dpdk_transport_t *native,
    const char *ip_str, const uint8_t mac[6]);
int rusteron_dpdk_transport_test_arp_rx(
    rusteron_dpdk_transport_t *native,
    int role, const uint8_t *frame, size_t frame_len);

/* Thread-local error buffer (defined in rusteron_dpdk_transport.c). */
void rusteron_dpdk_set_error(const char *message);
void rusteron_dpdk_set_error_code(const char *message, int code);

/* Aeron poller callbacks (rusteron_dpdk_poller.c, plan §7.6/§7.7). */
int rusteron_dpdk_poller_init(
    aeron_udp_transport_poller_t *poller,
    aeron_driver_context_t *context,
    aeron_udp_channel_transport_affinity_t affinity);
int rusteron_dpdk_poller_close(aeron_udp_transport_poller_t *poller);
int rusteron_dpdk_poller_add(
    aeron_udp_transport_poller_t *poller, aeron_udp_channel_transport_t *transport);
int rusteron_dpdk_poller_remove(
    aeron_udp_transport_poller_t *poller, aeron_udp_channel_transport_t *transport);
int rusteron_dpdk_poller_poll(
    aeron_udp_transport_poller_t *poller,
    struct mmsghdr *msgvec,
    size_t vlen,
    int64_t *bytes_rcved,
    aeron_udp_transport_recv_func_t recv_func,
    aeron_udp_channel_transport_recvmmsg_func_t recvmmsg_func,
    void *clientd);

/* Shared receive loop (rusteron_dpdk_poller.c): poll `client`'s port and
 * dispatch each valid frame to recv_func. When `endpoints` is non-NULL the
 * target transport is chosen by the map; otherwise only frames addressed to
 * `only_transport`'s own local endpoint are dispatched. Returns the number of
 * datagrams dispatched; accumulates payload bytes in *bytes_rcved. */
int rusteron_dpdk_poller_receive(
    rusteron_dpdk_client_t *client,
    aeron_udp_channel_transport_t *only_transport,
    const rusteron_dpdk_endpoint_map_t *endpoints,
    struct mmsghdr *msgvec,
    size_t vlen,
    int64_t *bytes_rcved,
    aeron_udp_transport_recv_func_t recv_func,
    void *clientd);

/* Test inspection of the receive counters (runtime.c). */
void rusteron_dpdk_transport_test_rx_stats(
    const rusteron_dpdk_transport_t *native, rusteron_dpdk_rx_stats_t *out);

#ifdef __cplusplus
}
#endif

#endif /* RUSTERON_DPDK_INTERNAL_H */
