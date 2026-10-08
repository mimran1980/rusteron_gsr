//! One message through IPC on one thread: offer it, then poll it back. Measures the
//! C publish and receive paths per message without cross-thread scheduling noise.

use criterion::{Criterion, criterion_group, criterion_main};
use rusteron_client::*;
use rusteron_media_driver::{AeronDriver, AeronDriverContext};
use std::hint::black_box;
use std::sync::atomic::Ordering;
use std::time::Duration;

const STREAM_ID: i32 = 7002;

fn offer_poll(c: &mut Criterion) {
    let driver = AeronDriverContext::new().unwrap();
    driver.set_dir_delete_on_start(true).unwrap();
    driver.set_dir_delete_on_shutdown(true).unwrap();
    driver
        .set_dir(&format!("{}bench-offer-poll", driver.get_dir()).into_c_string())
        .unwrap();
    let (stop, handle) = AeronDriver::launch_embedded(driver.clone(), false);

    let ctx = AeronContext::new().unwrap();
    ctx.set_dir(&driver.get_dir().into_c_string()).unwrap();
    let aeron = Aeron::new(&ctx).unwrap();
    aeron.start().unwrap();
    let timeout = Duration::from_secs(5);
    let publication = aeron
        .add_exclusive_publication(AERON_IPC_STREAM, STREAM_ID, timeout)
        .unwrap();
    let subscription = aeron
        .add_subscription(AERON_IPC_STREAM, STREAM_ID, Handlers::NONE, Handlers::NONE, timeout)
        .unwrap();
    while !publication.is_connected() {
        std::thread::sleep(Duration::from_millis(1));
    }

    let payload = [7u8; 32];
    let mut received = 0u64;
    c.bench_function("offer_poll_ipc_32b", |b| {
        b.iter(|| {
            while publication.offer(black_box(&payload)).is_err() {}
            while subscription
                .poll_fn(|message, _| received += message.len() as u64, 1)
                .unwrap_or(0)
                == 0
            {}
        })
    });
    black_box(received);

    stop.store(true, Ordering::SeqCst);
    let _ = handle.join();
}

criterion_group!(benches, offer_poll);
criterion_main!(benches);
