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
pub fn build_dpdk_native() {
    if !(cfg!(target_os = "linux") && cfg!(target_arch = "x86_64")) {
        panic!("the `dpdk` feature requires Linux x86_64 (Amazon Linux 2023 / EKS Nitro)");
    }

    // Prebuilt DPDK archives (the `precompile`/`static` path): link the
    // published `librusteron_dpdk*.a` from the artifacts dir instead of
    // compiling the native transport (and the Aeron C below) from source. The
    // archives still reference `librte_*` shared symbols, so libdpdk must be
    // present on the consumer (pkg-config probe) — exactly like the `static`
    // feature needs system `uuid`/`bsd`.
    let prebuilt_active = cfg!(all(
        any(feature = "precompile", feature = "precompile-rustls"),
        feature = "static"
    )) && std::env::var_os("RUSTERON_BUILD_FROM_SOURCE").is_none()
        && ["librusteron_dpdk.a", "librusteron_dpdk_eal.a", "librusteron_dpdk_port.a"]
            .iter()
            .all(|a| get_artifact_path().join(a).exists());

    let dpdk = pkg_config::Config::new()
        .atleast_version("23.11")
        .probe("libdpdk")
        .unwrap_or_else(|e| {
            panic!(
                "the `dpdk` feature requires libdpdk >= 23.11 (pkg-config probe failed: {e}). \
                 Install it with e.g. `dnf install dpdk-devel` on Amazon Linux 2023."
            )
        });

    if prebuilt_active {
        let artifacts_dir = get_artifact_path();
        println!("cargo:rustc-link-search=native={}", artifacts_dir.display());
        for lib in ["rusteron_dpdk", "rusteron_dpdk_eal", "rusteron_dpdk_port"] {
            println!("cargo:rustc-link-lib=static={lib}");
        }
        // librte_* shared libs (the prebuilt archives' unresolved symbols).
        for path in &dpdk.link_paths {
            println!("cargo:rustc-link-search=native={}", path.display());
        }
        for lib in &dpdk.libs {
            println!("cargo:rustc-link-lib={lib}");
        }
        return;
    }

    let aeron_path = std::fs::canonicalize(std::path::Path::new("./aeron"))
        .expect("aeron submodule missing — run `git submodule update --init`");

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
        // Aeron counters (plan §9): the counter-manager alloc/free/label helper
        // and its cached clock live in the client, not the driver-only dylib.
        // Compiled into the core archive so the counters test binary (which
        // links only the archive) resolves aeron_counters_manager_* and
        // aeron_clock_*; the manager uses fmin -> tests link libm.
        .file(native_dir.join("rusteron_dpdk_counters.c"))
        .file(aeron_path.join("aeron-client/src/main/c/concurrent/aeron_counters_manager.c"))
        .file(aeron_path.join("aeron-client/src/main/c/util/aeron_clock.c"))
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

    // Publish: cc writes the archives to OUT_DIR, but `publish_artifacts` (in
    // build_common.rs) only walks the cmake build dir — it never sees them.
    // Copy the production archives into the artifacts dir so the precompile
    // consumer (`static` + `dpdk`) can link them from `download_precompiled_binaries`.
    #[cfg(feature = "static")]
    if std::env::var("PUBLISH_ARTIFACTS").is_ok() {
        let publish_dir = get_artifact_path();
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        for archive in [
            "librusteron_dpdk.a",
            "librusteron_dpdk_eal.a",
            "librusteron_dpdk_port.a",
        ] {
            std::fs::copy(out_dir.join(archive), publish_dir.join(archive))
                .expect("failed to publish DPDK archive");
        }
        println!("DPDK artifacts published to: {}", publish_dir.display());
    }

    println!("cargo:rerun-if-changed={}", native_dir.display());
}
