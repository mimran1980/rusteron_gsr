//! Reading a fragment's header fields: in place, against the same reads through Aeron C
//! (one `aeron_header_values` copy per field, and `aeron_header_position`).

use criterion::{Criterion, criterion_group, criterion_main};
use rusteron_client::bindings::{aeron_data_header_t, aeron_header_position, aeron_header_t};
use rusteron_client::{AeronHeader, header_layout};
use std::hint::black_box;

fn header_reads(c: &mut Criterion) {
    let mut frame = aeron_data_header_t::default();
    frame.frame_header.frame_length = 64;
    frame.term_offset = 4_096;
    frame.term_id = 7;
    frame.session_id = 11;
    frame.stream_id = 13;
    frame.reserved_value = 17;
    let mut layout = header_layout::aeron_header_stct {
        frame: &mut frame,
        initial_term_id: 3,
        position_bits_to_shift: 16,
        fragmented_frame_length: -1,
        context: std::ptr::null_mut(),
    };
    let header = AeronHeader::from((&raw mut layout).cast::<aeron_header_t>());

    let mut group = c.benchmark_group("header_reads");
    group.bench_function("in_place", |b| {
        b.iter(|| {
            let header = black_box(&header);
            black_box((
                header.session_id(),
                header.stream_id(),
                header.reserved_value(),
                header.position(),
            ))
        })
    });
    group.bench_function("through_c", |b| {
        b.iter(|| {
            let header = black_box(&header);
            let field = |read: fn(&rusteron_client::AeronHeaderValuesFrame) -> i64| {
                header.get_values().ok().map(|values| read(&values.frame()))
            };
            black_box((
                field(|f| f.session_id().into()),
                field(|f| f.stream_id().into()),
                field(|f| f.reserved_value()),
                // SAFETY: `header` points at a complete header whose frame outlives the call.
                unsafe { aeron_header_position(header.get_inner()) },
            ))
        })
    });
    group.finish();
}

criterion_group!(benches, header_reads);
criterion_main!(benches);
