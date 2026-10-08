//! Release compiler flags for the vendored Aeron C build.
//!
//! The choice lives here rather than in `build_common.rs`, which each crate's
//! build script `include!`s and so cannot be unit-tested in place.

/// `CMAKE_C_FLAGS_RELEASE` for the Aeron C build of `arch` (`CARGO_CFG_TARGET_ARCH`)
/// and `vendor` (`CARGO_CFG_TARGET_VENDOR`).
///
/// Built from source, the C code targets this CPU (`-march=native`). The prebuilt
/// libraries published for `precompile` users (`publish`) target the architecture's
/// baseline, so they run on any CPU of it. `march` (`RUSTERON_C_MARCH`) overrides both.
pub fn release_c_flags(arch: &str, vendor: &str, publish: bool, march: Option<&str>) -> String {
    let mut flags = String::from("-O3 -DNDEBUG -funroll-loops");
    match (march.filter(|m| !m.is_empty()), publish) {
        (Some(march), _) => flags.push_str(&format!(" -march={march}")),
        (None, false) => flags.push_str(" -march=native"),
        (None, true) => match arch {
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
    use super::release_c_flags;

    #[test]
    fn from_source_targets_this_cpu() {
        assert_eq!(
            release_c_flags("x86_64", "unknown", false, None),
            "-O3 -DNDEBUG -funroll-loops -march=native"
        );
    }

    #[test]
    fn published_libraries_target_the_architecture_baseline() {
        assert_eq!(
            release_c_flags("x86_64", "unknown", true, None),
            "-O3 -DNDEBUG -funroll-loops -march=x86-64 -mtune=generic"
        );
        assert_eq!(
            release_c_flags("aarch64", "unknown", true, None),
            "-O3 -DNDEBUG -funroll-loops -march=armv8-a"
        );
        assert_eq!(
            release_c_flags("aarch64", "apple", true, None),
            "-O3 -DNDEBUG -funroll-loops"
        );
    }

    #[test]
    fn march_override_wins() {
        assert_eq!(
            release_c_flags("x86_64", "unknown", false, Some("x86-64-v3")),
            "-O3 -DNDEBUG -funroll-loops -march=x86-64-v3"
        );
        assert_eq!(
            release_c_flags("x86_64", "unknown", true, Some("x86-64-v2")),
            "-O3 -DNDEBUG -funroll-loops -march=x86-64-v2"
        );
        assert_eq!(
            release_c_flags("x86_64", "unknown", false, Some("")),
            "-O3 -DNDEBUG -funroll-loops -march=native"
        );
    }
}
