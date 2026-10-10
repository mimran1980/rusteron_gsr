//! UDP ping/pong RTT, as Aeron's `cping`/`cpong`, with pong echoing from another thread.

mod ping_pong_common;

use criterion::{Criterion, criterion_group, criterion_main};
use ping_pong_common::{Channels, run_ping, run_pong};
use rusteron_media_driver::testing::EmbeddedDriver;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

const CHANNELS: Channels = Channels {
    ping: c"aeron:udp?endpoint=localhost:20123",
    pong: c"aeron:udp?endpoint=localhost:20124",
};

fn criterion_benchmark(c: &mut Criterion) {
    // declared first so it drops last: it stops, joins and deletes its dir
    let driver = EmbeddedDriver::launch_with(|ctx| {
        ctx.set_print_configuration(true)?;
        // as Aeron's EmbeddedPingPong: a parked sender or receiver would add up to 1ms per hop
        ctx.set_sender_idle_strategy(c"noop")?;
        ctx.set_receiver_idle_strategy(c"noop")?;
        Ok(())
    })
    .expect("launch embedded driver");

    let running = Arc::new(AtomicBool::new(true));
    let pong_thread = {
        let running = Arc::clone(&running);
        let dir = driver.dir().to_string();
        thread::Builder::new()
            .name("pong".to_string())
            .spawn(move || run_pong(&dir, &CHANNELS, || running.load(Ordering::Acquire)))
            .expect("spawn pong thread")
    };

    let ping = run_ping(
        c,
        driver.dir(),
        &CHANNELS,
        "ping_pong_udp_benchmark",
        Duration::from_secs(5),
    );
    running.store(false, Ordering::Release);
    pong_thread.join().expect("pong thread panicked").expect("pong failed");
    ping.expect("ping failed");
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
