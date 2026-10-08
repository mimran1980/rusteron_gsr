//! Checks that `rusteron-media-driver` exports its Aeron header paths through its
//! `links` key: this crate's build script compiles `transport.c` against them.

#[cfg(test)]
mod tests {
    unsafe extern "C" {
        fn links_test_transport_bindings_size() -> usize;
    }

    #[test]
    fn transport_header_compiles_against_exported_paths() {
        // SAFETY: a pure function that build.rs compiled from transport.c.
        assert!(unsafe { links_test_transport_bindings_size() } > 0);
    }
}
