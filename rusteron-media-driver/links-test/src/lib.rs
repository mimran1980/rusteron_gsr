//! Guards `rusteron-media-driver`'s `links` export. A custom UDP transport (kernel bypass,
//! for example) is C compiled against the Aeron header paths that export hands to its build
//! script. This crate's build script does the same with `transport.c`, so a broken export
//! fails here rather than in a transport crate.

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
