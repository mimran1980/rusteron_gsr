/*
 * rusteron-media-driver DPDK transport — test fakes, shared header.
 *
 * Only test binaries link these archives (rusteron_dpdk_fake.a for the port
 * ops, rusteron_dpdk_fake_eal.a for the EAL seam). Both are DPDK-free so the
 * integration tests run on any host without libdpdk.
 */
#ifndef RUSTERON_DPDK_FAKE_H
#define RUSTERON_DPDK_FAKE_H

#include <stddef.h>
#include <stdint.h>

#include "rusteron_dpdk_port_ops.h"

#ifdef __cplusplus
extern "C" {
#endif

/* Reset all fake state (call via the Rust test setup between tests). */
void rusteron_dpdk_fake_reset(void);

/* Fail the n-th init step (1..=18; 1-9 hit the sender, 10-18 the receiver). */
void rusteron_dpdk_fake_set_failure(int step);

/* Override the reported driver name and checksum capability (0 clears). */
void rusteron_dpdk_fake_set_driver(const char *driver);
void rusteron_dpdk_fake_set_csum_ok(int ok);

/* The fake's call log, for reverse-teardown ordering assertions. */
int rusteron_dpdk_fake_log_count(void);
void rusteron_dpdk_fake_log_at(int index, char *buf, size_t buflen);

/* Fake EAL seam reset (rusteron_dpdk_fake_eal.c). */
void rusteron_dpdk_fake_eal_reset(void);

#ifdef __cplusplus
}
#endif

#endif /* RUSTERON_DPDK_FAKE_H */
