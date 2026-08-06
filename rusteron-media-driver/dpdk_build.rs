// DPDK ENA native transport build (plan §7.2), kept OUT of `build_common.rs`:
// that file is synced from rusteron-code-gen on every build, so any dpdk
// addition there is silently overwritten. This file and `build.rs` are
// media-driver-specific and never overwritten.
//
// `Path` is already in scope via `build_common.rs`, which `build.rs` includes
// first.

/// Compile `native/dpdk/*.c` into five static archives and emit link
/// directives. Linux x86_64 only; requires `libdpdk >= 23.11` (probed via
/// pkg-config). `lib.rs` carries the matching `compile_error!` for unsupported
/// targets; the panic here turns a missing/old libdpdk into a build-time error
/// with an install hint instead of an opaque link failure.
///
/// The archive split keeps the test binaries DPDK-free:
///
/// | archive            | contents                                   | linked into |
/// |--------------------|--------------------------------------------|-------------|
/// | `rusteron_dpdk`    | transport + runtime + packet + arp + endpoint map + poller | prod + tests |
/// | `rusteron_dpdk_eal`| real EAL seam (rte_eal_*)                  | prod        |
/// | `rusteron_dpdk_port`| real port ops (rte_eth_*)                 | prod        |
/// | `rusteron_dpdk_fake`| fake port ops (same symbol as port)        | tests       |
/// | `rusteron_dpdk_fake_eal`| fake EAL seam (same symbols as eal)    | tests       |
///
/// The fakes build with `cargo_metadata(false)` so their link-libs are never
/// emitted; test binaries reference them with explicit `#[link]` attributes
/// resolved via the core archive's link-search (cargo forwards link-search to
/// every target, but only forwards link-libs to the lib/bins/dependents).
#[cfg(feature = "dpdk")]
pub fn build_dpdk_native(aeron_path: &Path) {
    if !(cfg!(target_os = "linux") && cfg!(target_arch = "x86_64")) {
        panic!("the `dpdk` feature requires Linux x86_64 (Amazon Linux 2023 / EKS Nitro)");
    }

    let dpdk = pkg_config::Config::new()
        .atleast_version("23.11")
        .probe("libdpdk")
        .unwrap_or_else(|e| {
            panic!(
                "the `dpdk` feature requires libdpdk >= 23.11 (pkg-config probe failed: {e}). \
                 Install it with e.g. `dnf install dpdk-devel` on Amazon Linux 2023."
            )
        });

    let cargo_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let native_dir = cargo_dir.join("native/dpdk");
    let test_dir = native_dir.join("test");

    let base = || {
        let mut b = cc::Build::new();
        b.std("c11");
        // _POSIX_C_SOURCE is declared in rusteron_dpdk_internal.h, which every
        // native DPDK source includes first. It must NOT be a global -D: the
        // vendored Aeron util files (aeron_error.c etc.) declare their own
        // value and a command-line define would redefine it (benign warning).
        b.include(&native_dir);
        b.include(aeron_path.join("aeron-driver/src/main/c"));
        b.include(aeron_path.join("aeron-client/src/main/c"));
        b
    };
    let with_dpdk = || {
        let mut b = base();
        // DPDK's x86_64 baseline: rte_memcpy.h references SSSE3 intrinsics
        // (palignr) unconditionally, so the seam must be compiled with the same
        // ISA floor DPDK itself is built with.
        b.flag("-mssse3");
        for inc in &dpdk.include_paths {
            b.include(inc);
        }
        for (key, value) in &dpdk.defines {
            match value {
                Some(v) => {
                    b.define(key, Some(v.as_str()));
                }
                None => {
                    b.define(key, None);
                }
            }
        }
        b
    };

    // Core: ABI + runtime orchestration + frame/ARP encoders + receive path.
    // DPDK-free; its link-search reaches every target (including same-package
    // integration tests).
    base().file(native_dir.join("rusteron_dpdk_transport.c"))
        .file(native_dir.join("rusteron_dpdk_runtime.c"))
        .file(native_dir.join("rusteron_dpdk_packet.c"))
        .file(native_dir.join("rusteron_dpdk_arp.c"))
        .file(native_dir.join("rusteron_dpdk_endpoint_map.c"))
        .file(native_dir.join("rusteron_dpdk_poller.c"))
        // AERON_SET_ERR (transport.c) routes through aeron_err_set, which lives
        // in the client util and is NOT exported by the shared libaeron_driver.so
        // (that target builds DRIVER_ONLY_SOURCE). Compile the error + thread +
        // alloc helpers into the core archive so the symbol is always provided,
        // for both test binaries and production links.
        .file(aeron_path.join("aeron-client/src/main/c/util/aeron_error.c"))
        .file(aeron_path.join("aeron-client/src/main/c/concurrent/aeron_thread.c"))
        .file(aeron_path.join("aeron-client/src/main/c/aeron_alloc.c"))
        // aeron_error.c's AERON_FPRINTF references aeron_fprintf, which only
        // exists in the client (aeronc.c); provide the default-handler
        // behaviour locally (see the file for rationale).
        .file(native_dir.join("aeron_fprintf_shim.c"))
        .compile("rusteron_dpdk");

    // Real seams: production only. cc emits their link-libs for the lib/bins.
    with_dpdk().file(native_dir.join("rusteron_dpdk_eal.c")).compile("rusteron_dpdk_eal");
    with_dpdk().file(native_dir.join("rusteron_dpdk_port.c")).compile("rusteron_dpdk_port");

    // Fakes: DPDK-free; never emitted into production links. Test binaries
    // `#[link]` them explicitly.
    base().cargo_metadata(false)
        .file(test_dir.join("rusteron_dpdk_fake_port.c"))
        .compile("rusteron_dpdk_fake");
    base().cargo_metadata(false)
        .file(test_dir.join("rusteron_dpdk_fake_eal.c"))
        .compile("rusteron_dpdk_fake_eal");

    // The core archive's link-search is already emitted by cc; add the DPDK
    // search paths and libs so the real seams resolve their rte_* references.
    for path in &dpdk.link_paths {
        println!("cargo:rustc-link-search=native={}", path.display());
    }
    for lib in &dpdk.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }
    println!("cargo:rerun-if-changed={}", native_dir.display());
}
