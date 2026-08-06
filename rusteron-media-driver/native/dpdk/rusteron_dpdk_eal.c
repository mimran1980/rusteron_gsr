/*
 * rusteron-media-driver DPDK transport — real EAL seam (plan §7.2).
 *
 * Wraps rte_eal_init and tracks whether the EAL runtime is up (DPDK >= 23.11
 * exposes no public rte_eal_is_initialized). The argv is built from the typed
 * config: exactly two allow-listed devices (the sender and receiver) — a PCI
 * BDF via `-a <bdf>` for ENA, or a virtual-device name via `--vdev=<name>` for
 * the test/TAP path (plan §11.2) — plus a per-process file prefix,
 * primary-process mode, no telemetry, and either a hugepage directory
 * (production, so startup fails if hugetlbfs is unusable) or --no-huge (tests
 * without hugetlbfs).
 *
 * Only this translation unit and rusteron_dpdk_port.c reference libdpdk, so
 * the seam keeps every other translation unit DPDK-free and testable.
 */
#include "rusteron_dpdk_internal.h"

#include <rte_eal.h>
#include <rte_version.h>

#include <stdio.h>
#include <string.h>

/* DPDK >= 23.11 exposes no public rte_eal_is_initialized(); the runtime owns
 * the only EAL instance in the process, so track initialization here. */
static int rusteron_dpdk_eal_initialized = 0;

int rusteron_dpdk_eal_is_initialized(void)
{
    return rusteron_dpdk_eal_initialized ? 1 : 0;
}

int rusteron_dpdk_eal_init(const rusteron_dpdk_eal_params_t *params, char *errbuf, size_t errlen)
{
    const rusteron_dpdk_config_t *cfg = params->config;
    const char *selectors[2];
    size_t nselectors = 0;
    selectors[nselectors++] = cfg->sender_pci;
    selectors[nselectors++] = cfg->receiver_pci;

    const char *mem_option;
    if (RUSTERON_DPDK_EAL_NO_HUGE == params->mode)
    {
        mem_option = "--no-huge";
    }
    else
    {
        mem_option = "--huge-dir";
    }

    /* argv[0] is the program name; EAL scans argv[1..] for its options. A
     * canonical PCI BDF contains ':', so the shape distinguishes an allow-listed
     * ENA (`-a <bdf>`) from a virtual-device name (`--vdev=<name>`, plan §11.2). */
    char *argv[18];
    int argc = 0;
    argv[argc++] = (char *)"rusteron-dpdk";
    char vdev_opt[2][32];
    for (size_t i = 0; i < 2; i++)
    {
        const char *sel = selectors[i];
        if (rusteron_dpdk_selector_is_pci(sel))
        {
            argv[argc++] = (char *)"-a";
            argv[argc++] = (char *)sel;
        }
        else
        {
            snprintf(vdev_opt[i], sizeof(vdev_opt[i]), "--vdev=%s", sel);
            argv[argc++] = vdev_opt[i];
        }
    }
    argv[argc++] = (char *)"--file-prefix";
    argv[argc++] = (char *)cfg->file_prefix;
    argv[argc++] = (char *)"--proc-type";
    argv[argc++] = (char *)"primary";
    argv[argc++] = (char *)"--no-telemetry";
    argv[argc++] = (char *)mem_option;
    if (RUSTERON_DPDK_EAL_NO_HUGE != params->mode)
    {
        argv[argc++] = (char *)cfg->hugepage_dir;
    }

    char version[64] = "";
    const char *ver = rte_version();
    if (NULL != ver)
    {
        snprintf(version, sizeof(version), "%s", ver);
    }

    int rc = rte_eal_init(argc, argv);
    if (rc < 0)
    {
        snprintf(errbuf, errlen,
                 "rte_eal_init failed (rc=%d, DPDK %s): cannot initialize EAL "
                 "with devices %s,%s and hugepages %s; verify VFIO binding and hugetlbfs",
                 rc, version[0] != '\0' ? version : "<unknown>",
                 selectors[0], selectors[1],
                 RUSTERON_DPDK_EAL_NO_HUGE == params->mode ? "disabled" : cfg->hugepage_dir);
        return -1;
    }

    rusteron_dpdk_eal_initialized = 1;
    return 0;
}
