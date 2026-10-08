//! Latency micro-benchmarks for the publish/claim hot paths.
//!
//! The typed `offer` / `try_claim_owned` must cost no more than the raw `i64`-returning
//! `offer_raw` / `try_claim_raw`, and those no more than the same Aeron C calls made directly.
//! Each arm publishes in batches; between them, untimed, it drains the subscription and waits
//! for room in the publication window, so every publish lands instead of returning back pressure.
//! Run with `cargo bench -p rusteron-client --bench offer_claim_poll`.

use criterion::{Criterion, criterion_group, criterion_main};
use rusteron_client::bindings::{aeron_buffer_claim_commit, aeron_publication_offer, aeron_publication_try_claim};
use rusteron_client::*;
use rusteron_media_driver::testing::EmbeddedDriver;
use std::ffi::CStr;
use std::hint::black_box;
use std::time::{Duration, Instant};

const STREAM_ID: i32 = 7001;
static CHANNEL: &CStr = AERON_IPC_STREAM;
const PAYLOAD_LEN: usize = 32;
// a 32-byte payload takes a 64-byte frame, so a batch is 256KB, far below the IPC publication window
const FRAME_LEN: i64 = 64;
const BATCH: u64 = 4096;

/// An embedded driver with a connected IPC publication and subscription.
struct Harness {
    publisher: AeronPublication,
    subscription: AeronSubscription,
    _aeron: Aeron,
    _ctx: AeronContext,
    // last, so the client closes before the driver stops
    _driver: EmbeddedDriver,
}

fn harness() -> Harness {
    let driver = EmbeddedDriver::launch().unwrap();
    let ctx = AeronContext::new().unwrap();
    ctx.set_dir(&driver.dir().into_c_string()).unwrap();
    let aeron = Aeron::new(&ctx).unwrap();
    aeron.start().unwrap();

    let publisher = aeron
        .async_add_publication(CHANNEL, STREAM_ID)
        .unwrap()
        .poll_blocking(Duration::from_secs(5))
        .unwrap();
    let subscription = aeron
        .async_add_subscription(CHANNEL, STREAM_ID, Handlers::NONE, Handlers::NONE)
        .unwrap()
        .poll_blocking(Duration::from_secs(5))
        .unwrap();

    // Wait until the publication sees the subscriber image.
    let deadline = Instant::now() + Duration::from_secs(5);
    while !publisher.is_connected() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(publisher.is_connected(), "publication never connected");

    Harness {
        publisher,
        subscription,
        _aeron: aeron,
        _ctx: ctx,
        _driver: driver,
    }
}

/// Times `iters` calls of `publish` in batches, draining and waiting for window between
/// batches untimed, and checks that the calls published.
fn timed(h: &Harness, iters: u64, mut publish: impl FnMut()) -> Duration {
    let mut elapsed = Duration::ZERO;
    let mut left = iters;
    while left > 0 {
        let batch = left.min(BATCH);
        // the driver reopens the window only as fast as it cleans old terms, which a
        // saturating publisher can outrun
        while h.publisher.position_limit() - h.publisher.position() < batch as i64 * FRAME_LEN {
            std::thread::yield_now();
        }
        let before = h.publisher.position();
        let start = Instant::now();
        for _ in 0..batch {
            publish();
        }
        elapsed += start.elapsed();
        // a term rotation turns one publish into ADMIN_ACTION; any more means back pressure
        let published = (h.publisher.position() - before) / FRAME_LEN;
        assert!(
            published + 1 >= batch as i64,
            "only {published} of {batch} publishes landed, so the bench timed the error path"
        );
        while h.subscription.poll_fn(|_, _| {}, 1024).expect("drain subscription") > 0 {}
        left -= batch;
    }
    elapsed
}

fn bench_offer(c: &mut Criterion) {
    let h = harness();
    let payload = vec![0u8; PAYLOAD_LEN];
    let claim_buf = AeronBufferClaim::default();
    // read once, as a C caller holds its pointers
    let publication = h.publisher.get_inner();
    let claim = claim_buf.get_inner();

    {
        let mut g = c.benchmark_group("offer");
        g.bench_function("c_offer", |b| {
            b.iter_custom(|iters| {
                timed(&h, iters, || {
                    // SAFETY: the publication outlives the bench and the payload outlives the call.
                    black_box(unsafe {
                        aeron_publication_offer(
                            publication,
                            black_box(payload.as_ptr()),
                            PAYLOAD_LEN,
                            None,
                            std::ptr::null_mut(),
                        )
                    });
                })
            })
        });
        g.bench_function("raw_offer_i64", |b| {
            b.iter_custom(|iters| {
                timed(&h, iters, || {
                    black_box(h.publisher.offer_raw(black_box(&payload), Handlers::NONE));
                })
            })
        });
        g.bench_function("offer", |b| {
            b.iter_custom(|iters| {
                timed(&h, iters, || {
                    let _ = black_box(h.publisher.offer(black_box(&payload)));
                })
            })
        });
        g.finish();
    }

    {
        let mut g = c.benchmark_group("claim");
        g.bench_function("c_try_claim_commit", |b| {
            b.iter_custom(|iters| {
                timed(&h, iters, || {
                    // SAFETY: the publication and the claim outlive the bench; the claim is
                    // committed only after a successful try_claim filled it.
                    unsafe {
                        if aeron_publication_try_claim(publication, PAYLOAD_LEN, claim) >= 0 {
                            aeron_buffer_claim_commit(claim);
                        }
                    }
                })
            })
        });
        g.bench_function("raw_try_claim_commit", |b| {
            b.iter_custom(|iters| {
                timed(&h, iters, || {
                    if h.publisher.try_claim_raw(PAYLOAD_LEN, &claim_buf) >= 0 {
                        let _ = claim_buf.commit();
                    }
                })
            })
        });
        g.bench_function("try_claim_owned_commit", |b| {
            b.iter_custom(|iters| {
                timed(&h, iters, || {
                    if let Ok(claim) = h.publisher.try_claim_owned(PAYLOAD_LEN) {
                        let _ = claim.commit();
                    }
                })
            })
        });
        g.finish();
    }
}

criterion_group!(benches, bench_offer);
criterion_main!(benches);
