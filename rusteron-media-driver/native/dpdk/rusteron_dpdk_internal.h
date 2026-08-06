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

#include <stddef.h>
#include <stdint.h>

#include "rusteron_dpdk_transport.h"
#include "rusteron_dpdk_port_ops.h"

#ifdef __cplusplus
extern "C" {
#endif

#define RUSTERON_DPDK_ROLE_SENDER 0
#define RUSTERON_DPDK_ROLE_RECEIVER 1

/* EAL modes selected by rusteron_dpdk_test_set_eal_mode (production = REAL). */
#define RUSTERON_DPDK_EAL_REAL 0    /* --huge-dir <configured path> */
#define RUSTERON_DPDK_EAL_NO_HUGE 1 /* --no-huge (tests without hugetlbfs) */
#define RUSTERON_DPDK_EAL_SKIP 2    /* skip the EAL seam entirely (tests) */

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
} rusteron_dpdk_port_t;

struct rusteron_dpdk_transport_stct
{
    rusteron_dpdk_config_t config;
    const rusteron_dpdk_port_ops_t *ops; /* active port ops (real or fake) */
    int eal_up;                          /* this transport owns EAL up */
    rusteron_dpdk_port_t sender;
    rusteron_dpdk_port_t receiver;
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

/* Test inspection of the two role ports (distinct ports / mempools). */
void rusteron_dpdk_transport_test_dump(
    const rusteron_dpdk_transport_t *native,
    uint16_t *sender_port, uintptr_t *sender_pool,
    uint16_t *receiver_port, uintptr_t *receiver_pool);

/* Thread-local error buffer (defined in rusteron_dpdk_transport.c). */
void rusteron_dpdk_set_error(const char *message);
void rusteron_dpdk_set_error_code(const char *message, int code);

#ifdef __cplusplus
}
#endif

#endif /* RUSTERON_DPDK_INTERNAL_H */
