//! Release compiler flags for the vendored Aeron C build.
//!
//! The choice lives here rather than in `build_common.rs`, which each crate's
//! build script `include!`s and so cannot be unit-tested in place.

/// Which CPUs a release build of Aeron C must run on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuTarget {
    /// Built from source for this CPU (`-march=native`).
    ThisCpu,
    /// A prebuilt library published for `precompile` users, which must run on any CPU of
    /// the architecture.
    Baseline,
}

/// `CMAKE_C_FLAGS_RELEASE` for the Aeron C build of `arch` (`CARGO_CFG_TARGET_ARCH`)
/// and `vendor` (`CARGO_CFG_TARGET_VENDOR`) for `target`. `march` (`RUSTERON_C_MARCH`)
/// overrides the target's `-march`.
pub fn release_c_flags(arch: &str, vendor: &str, target: CpuTarget, march: Option<&str>) -> String {
    let mut flags = String::from("-O3 -DNDEBUG -funroll-loops");
    match (march.filter(|m| !m.is_empty()), target) {
        (Some(march), _) => flags.push_str(&format!(" -march={march}")),
        (None, CpuTarget::ThisCpu) => flags.push_str(" -march=native"),
        (None, CpuTarget::Baseline) => match arch {
            "x86_64" => flags.push_str(" -march=x86-64 -mtune=generic"),
            "aarch64" if vendor != "apple" => flags.push_str(" -march=armv8-a"),
            // Apple's default CPU is already the platform baseline.
            _ => {}
        },
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::{CpuTarget, release_c_flags};

    #[test]
    fn from_source_targets_this_cpu() {
        assert_eq!(
            release_c_flags("x86_64", "unknown", CpuTarget::ThisCpu, None),
            "-O3 -DNDEBUG -funroll-loops -march=native"
        );
    }

    #[test]
    fn published_libraries_target_the_architecture_baseline() {
        assert_eq!(
            release_c_flags("x86_64", "unknown", CpuTarget::Baseline, None),
            "-O3 -DNDEBUG -funroll-loops -march=x86-64 -mtune=generic"
        );
        assert_eq!(
            release_c_flags("aarch64", "unknown", CpuTarget::Baseline, None),
            "-O3 -DNDEBUG -funroll-loops -march=armv8-a"
        );
        assert_eq!(
            release_c_flags("aarch64", "apple", CpuTarget::Baseline, None),
            "-O3 -DNDEBUG -funroll-loops"
        );
    }

    #[test]
    fn march_override_wins() {
        assert_eq!(
            release_c_flags("x86_64", "unknown", CpuTarget::ThisCpu, Some("x86-64-v3")),
            "-O3 -DNDEBUG -funroll-loops -march=x86-64-v3"
        );
        assert_eq!(
            release_c_flags("x86_64", "unknown", CpuTarget::Baseline, Some("x86-64-v2")),
            "-O3 -DNDEBUG -funroll-loops -march=x86-64-v2"
        );
        assert_eq!(
            release_c_flags("x86_64", "unknown", CpuTarget::ThisCpu, Some("")),
            "-O3 -DNDEBUG -funroll-loops -march=native"
        );
    }
}
