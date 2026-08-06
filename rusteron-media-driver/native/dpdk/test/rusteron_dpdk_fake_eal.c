/*
 * rusteron-media-driver DPDK transport — fake EAL seam for tests.
 *
 * Provides the same rusteron_dpdk_eal_init / rusteron_dpdk_eal_is_initialized
 * symbols the production rusteron_dpdk_eal.c defines, so test binaries can link
 * the runtime without libdpdk. The singleton semantics mirror the real EAL: a
 * second init in the same process is rejected by the runtime's
 * ever-initialized guard before this seam is consulted.
 */
#include "rusteron_dpdk_fake.h"
#include "rusteron_dpdk_internal.h"

static int rusteron_dpdk_fake_eal_initialized = 0;

void rusteron_dpdk_fake_eal_reset(void)
{
    rusteron_dpdk_fake_eal_initialized = 0;
}

int rusteron_dpdk_eal_is_initialized(void)
{
    return rusteron_dpdk_fake_eal_initialized ? 1 : 0;
}

int rusteron_dpdk_eal_init(const rusteron_dpdk_eal_params_t *params, char *errbuf, size_t errlen)
{
    (void)params;
    (void)errbuf;
    (void)errlen;
    rusteron_dpdk_fake_eal_initialized = 1;
    return 0;
}
