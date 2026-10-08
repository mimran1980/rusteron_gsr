//! UDP ping/pong RTT, as Aeron's `cping`/`cpong`: ping stamps a time into each message,
//! pong echoes it from another thread, and ping polls its image until the echo is back.

use criterion::{Criterion, criterion_group, criterion_main};
use rusteron_client::*;
use rusteron_media_driver::testing::EmbeddedDriver;
use std::error::Error;
use std::hint::{black_box, spin_loop};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};

const PING_STREAM_ID: i32 = 1002;
const PONG_STREAM_ID: i32 = 1003;
const PING_CHANNEL: &std::ffi::CStr = c"aeron:udp?endpoint=localhost:20123";
const PONG_CHANNEL: &std::ffi::CStr = c"aeron:udp?endpoint=localhost:20124";
const MESSAGE_LENGTH: usize = 32;
const FRAGMENT_COUNT_LIMIT: usize = 10;

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
            .spawn(move || run_pong(&running, &dir))
            .expect("spawn pong thread")
    };

    let ping = run_ping(c, driver.dir());
    running.store(false, Ordering::Release);
    pong_thread.join().expect("pong thread panicked").expect("pong failed");
    ping.expect("ping failed");
}

fn run_ping(c: &mut Criterion, dir: &str) -> Result<(), Box<dyn Error>> {
    let context = AeronContext::new()?;
    context.set_dir(&cformat!("{dir}"))?;
    let aeron = Aeron::new(&context)?;
    aeron.start()?;

    let pong_publication = aeron
        .async_add_exclusive_publication(PONG_CHANNEL, PONG_STREAM_ID)?
        .poll_blocking(Duration::from_secs(5))?;
    let ping_subscription = aeron
        .async_add_subscription(PING_CHANNEL, PING_STREAM_ID, Handlers::NONE, Handlers::NONE)?
        .poll_blocking(Duration::from_secs(5))?;

    println!("PING: pong publisher {PONG_CHANNEL:?} {PONG_STREAM_ID}");
    println!("PING: ping subscriber {PING_CHANNEL:?} {PING_STREAM_ID}");

    let image = wait_for_pong(&pong_publication, &ping_subscription)?;
    let mut buffer = [0u8; MESSAGE_LENGTH];
    c.bench_function("ping_pong_udp_benchmark", |b| {
        b.iter(|| record_rtt(&pong_publication, &image, &mut buffer).expect("round trip"));
    });
    Ok(())
}

/// Both directions must be up: our pings reach pong, and pong's echoes reach our image.
fn wait_for_pong(
    publication: &AeronExclusivePublication,
    subscription: &AeronSubscription,
) -> Result<AeronImage, Box<dyn Error>> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if publication.is_connected()
            && let Some(image) = subscription.image_at_index(0)
        {
            return Ok(image);
        }
        if Instant::now() > deadline {
            return Err("pong never connected".into());
        }
        sleep(Duration::from_millis(10));
    }
}

fn run_pong(running: &AtomicBool, dir: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let context = AeronContext::new()?;
    context.set_dir(&cformat!("{dir}"))?;
    let aeron = Aeron::new(&context)?;
    aeron.start()?;
    let ping_publication = aeron
        .async_add_exclusive_publication(PING_CHANNEL, PING_STREAM_ID)?
        .poll_blocking(Duration::from_secs(5))?;
    let pong_subscription = aeron
        .async_add_subscription(PONG_CHANNEL, PONG_STREAM_ID, Handlers::NONE, Handlers::NONE)?
        .poll_blocking(Duration::from_secs(5))?;

    println!("PONG: ping publisher {PING_CHANNEL:?} {PING_STREAM_ID}");
    println!("PONG: pong subscriber {PONG_CHANNEL:?} {PONG_STREAM_ID}");

    // the poll callback cannot return an error, so it parks the first one here
    let mut failure = None;
    while running.load(Ordering::Acquire) {
        let fragments = pong_subscription.poll_fn(
            |buffer, _header| {
                if failure.is_none() {
                    failure = echo(&ping_publication, buffer).err();
                }
            },
            FRAGMENT_COUNT_LIMIT,
        )?;
        if let Some(e) = failure.take() {
            return Err(e);
        }
        if fragments == 0 {
            spin_loop();
        }
    }
    Ok(())
}

#[inline]
fn echo(publication: &AeronExclusivePublication, buffer: &[u8]) -> Result<(), Box<dyn Error + Send + Sync>> {
    loop {
        match publication.try_claim_owned(buffer.len()) {
            // pings never fragment, so the claim's own BEGIN|END flags are the right ones
            Ok(mut claim) => {
                claim.data().copy_from_slice(buffer);
                claim.commit()?;
                return Ok(());
            }
            Err(e) if e.is_retryable() => spin_loop(),
            Err(e) => return Err(e.into()),
        }
    }
}

#[inline]
fn record_rtt(
    publication: &AeronExclusivePublication,
    image: &AeronImage,
    buffer: &mut [u8],
) -> Result<(), Box<dyn Error>> {
    let position = loop {
        // stamp each attempt, so time spent back-pressured is not counted as RTT
        buffer[0..8].copy_from_slice(&Aeron::nano_clock().to_le_bytes());
        match publication.offer(buffer) {
            Ok(position) => break position,
            Err(e) if e.is_retryable() => spin_loop(),
            Err(e) => return Err(e.into()),
        }
    };

    // pong echoes at the same length, so the echo has been read once the image reaches our position
    while image.position() < position {
        let fragments = image.poll_fn(
            |reply, _header| {
                black_box(Aeron::nano_clock() - read_i64(reply));
            },
            FRAGMENT_COUNT_LIMIT,
        )?;
        if fragments == 0 {
            if image.is_closed() {
                return Err("pong went away".into());
            }
            spin_loop();
        }
    }
    Ok(())
}

fn read_i64(buffer: &[u8]) -> i64 {
    i64::from_le_bytes(buffer[0..8].try_into().expect("ping shorter than 8 bytes"))
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
