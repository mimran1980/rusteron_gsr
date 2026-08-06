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

/* Data-path knobs (plan §7.4): cap the frames accepted per tx_burst, and cap
 * the number of mbufs that may be live at once (pool exhaustion). */
void rusteron_dpdk_fake_set_tx_burst_cap(uint16_t n);
void rusteron_dpdk_fake_set_pool_avail(int n);

/* A frame accepted by the fake tx_burst (the mbuf is owned by the "NIC"). */
typedef struct rusteron_dpdk_fake_capture_stct
{
    uint8_t data[2048];
    uint32_t len;
    uint32_t ol_flags;
    uint16_t l2_len, l3_len, l4_len;
    uint16_t udp_pseudo_csum;
    uint16_t port_id;
} rusteron_dpdk_fake_capture_t;

int rusteron_dpdk_fake_capture_count(void);
int rusteron_dpdk_fake_capture_at(int index, rusteron_dpdk_fake_capture_t *out);

/* Live-mbuf accounting for leak assertions: after any send, every allocated
 * mbuf has been released (sent or unsent), so allocated() == released(). */
int rusteron_dpdk_fake_allocated(void);
int rusteron_dpdk_fake_released(void);

/* RX injector (plan §7.6): queue a raw frame for the next rx_burst on the
 * given port. `rx_ol_flags`/`nb_segs` seed the delivered view so tests can
 * exercise NIC-reported checksum verdicts and multi-segment rejection.
 * Returns 0 on success, -1 when the queue is full or the port is out of range. */
int rusteron_dpdk_fake_rx_inject(
    uint16_t port_id, const uint8_t *frame, size_t len,
    uint32_t rx_ol_flags, uint32_t nb_segs);

/* Number of frames still queued for the port's rx_burst (test assertion). */
int rusteron_dpdk_fake_rx_queued(uint16_t port_id);

/* Fake EAL seam reset (rusteron_dpdk_fake_eal.c). */
void rusteron_dpdk_fake_eal_reset(void);

#ifdef __cplusplus
}
#endif

#endif /* RUSTERON_DPDK_FAKE_H */
