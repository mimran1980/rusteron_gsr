//! # Duty cycle
//!
//! One thread drives everything without blocking: an Aeron client with no conductor thread
//! (the agent invoker), its archive client, a publication and a persistent subscription.
//! Each cycle calls `archive.do_work()`, which runs the client's conductor and hands
//! recording signals to their consumer, then polls the persistent subscription and offers
//! the next message, and idles only when the cycle did no work.
//!
//! With the agent invoker nothing else runs the client's conductor: the async connect, the
//! publication add and the persistent subscription build make progress only because the
//! cycle calls `archive.do_work()` (or `aeron.main_do_work()` before the archive exists).
//! A persistent subscription poll runs the conductor too.
//!
//! Requires `java` on `PATH`: the `rusteron_archive::testing` harness starts a Java Archive.
//!
//! ```bash
//! cargo run --release --features "static precompile" --example duty_cycle
//! ```

use rusteron_archive::testing::{EmbeddedArchiveMediaDriverProcess, find_unused_udp_port};
use rusteron_archive::*;
use std::thread::yield_now;
use std::time::{Duration, Instant};

const STREAM_ID: i32 = 1301;
const MESSAGES: usize = 100;
/// Messages sent before the persistent subscription exists, which it reads by replay.
const HISTORY: usize = 50;
/// Pace of the live messages, slow enough for the replay to catch up with them.
const SEND_INTERVAL: Duration = Duration::from_millis(1);

/// Offers message `seq`: `Ok(false)` when it should be retried next cycle.
fn send(publication: &AeronExclusivePublication, seq: usize) -> Result<bool, AeronOfferError> {
    match publication.offer(format!("message {seq}").as_bytes()) {
        Ok(_) => Ok(true),
        Err(e) if e.is_retryable() => Ok(false),
        Err(e) => Err(e),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    EmbeddedArchiveMediaDriverProcess::kill_all_java_processes().ok();

    let id = Aeron::nano_clock();
    let aeron_dir = format!("target/aeron/{id}_example/shm");
    let archive_dir = format!("target/aeron/{id}_example/archive");
    let req_port = find_unused_udp_port(9000).ok_or("no free port")?;
    let resp_port = find_unused_udp_port(req_port + 1).ok_or("no free port")?;
    let events_port = find_unused_udp_port(resp_port + 1).ok_or("no free port")?;
    let _process = EmbeddedArchiveMediaDriverProcess::build_and_start(
        &aeron_dir,
        &archive_dir,
        &format!("aeron:udp?endpoint=localhost:{req_port}"),
        &format!("aeron:udp?endpoint=localhost:{resp_port}"),
        &format!("aeron:udp?endpoint=localhost:{events_port}"),
    )?;

    let ctx = AeronContext::new()?;
    ctx.set_dir(&cformat!("{aeron_dir}"))?;
    ctx.set_use_conductor_agent_invoker(true)?;
    ctx.set_error_handler(Some(|code: i32, msg: &str| eprintln!("[client] {code}: {msg}")))?;
    let aeron = Aeron::new(&ctx)?;
    aeron.start()?;

    let archive_ctx = AeronArchiveContext::new()?;
    // without a client of its own, the archive would start one with a conductor thread
    archive_ctx.set_aeron(&aeron)?;
    archive_ctx.set_control_request_channel(&cformat!("aeron:udp?endpoint=localhost:{req_port}"))?;
    archive_ctx.set_control_response_channel(&cformat!("aeron:udp?endpoint=localhost:{resp_port}"))?;
    archive_ctx.set_error_handler(Some(|code: i32, msg: &str| eprintln!("[archive] {code}: {msg}")))?;
    archive_ctx.set_recording_signal_consumer(Some(|s: AeronArchiveRecordingSignal| {
        println!("recording signal {:?}", s.signal());
    }))?;

    // until the archive exists, the cycle runs the conductor itself
    let connect = AeronArchiveAsyncConnect::new_with_aeron(&archive_ctx, &aeron)?;
    let deadline = Instant::now() + Duration::from_secs(20);
    let archive = loop {
        aeron.main_do_work()?;
        if let Some(archive) = connect.poll()? {
            break archive;
        }
        if Instant::now() > deadline {
            return Err("archive connect timed out".into());
        }
        yield_now();
    };
    println!("connected to the archive");

    // record first, so every message the publication sends is in the recording
    archive.start_recording(AERON_IPC_STREAM, STREAM_ID, SOURCE_LOCATION_LOCAL, true)?;
    let adding = aeron.async_add_exclusive_publication(AERON_IPC_STREAM, STREAM_ID)?;
    let deadline = Instant::now() + Duration::from_secs(10);
    let publication = loop {
        archive.do_work()?;
        if let Some(publication) = adding.poll()? {
            break publication;
        }
        if Instant::now() > deadline {
            return Err("publication add timed out".into());
        }
        yield_now();
    };

    // the persistent subscription needs a recording id, even to start from live
    let session_id = publication.get_constants()?.session_id;
    let counters = aeron.counters_reader();
    let deadline = Instant::now() + Duration::from_secs(10);
    let recording_id = loop {
        archive.do_work()?;
        let counter_id = RecordingPos::find_counter_id_by_session(&counters, session_id);
        if counter_id >= 0 {
            break RecordingPos::get_recording_id(&counters, counter_id)?;
        }
        if Instant::now() > deadline {
            return Err("the recording never started".into());
        }
        yield_now();
    };
    println!("recording {recording_id} started");

    // history for the persistent subscription to replay
    let mut sent = 0;
    let deadline = Instant::now() + Duration::from_secs(10);
    while sent < HISTORY {
        archive.do_work()?;
        if send(&publication, sent)? {
            sent += 1;
        } else if Instant::now() > deadline {
            return Err("could not send the history".into());
        }
    }

    let mut building = PersistentSubscriptionBuilder::new_with_aeron(&archive_ctx, &aeron)?
        .live_channel(AERON_IPC_STREAM.to_str()?)?
        .live_stream_id(STREAM_ID)?
        .replay_channel("aeron:udp?endpoint=localhost:0")?
        .replay_stream_id(STREAM_ID + 1)?
        .start_from_beginning()?
        .recording_id(recording_id)?
        .build_async()?;

    let mut ps = None;
    let mut next_send = Instant::now();
    let mut received = 0;
    let mut was_live = false;
    let deadline = Instant::now() + Duration::from_secs(60);
    while !(was_live && sent >= MESSAGES && received >= sent) {
        if Instant::now() > deadline {
            return Err(format!("received {received} of {sent} messages, live: {was_live}").into());
        }
        // runs the conductor (which completes the async build) and delivers recording signals
        let mut work = archive.do_work()?;
        if !archive.get_control_response_subscription().is_connected() {
            return Err("lost the archive".into());
        }

        if ps.is_none() {
            ps = building.poll()?;
            if ps.is_some() {
                println!("persistent subscription built; {sent} messages recorded before it");
            }
        }
        if let Some(ps) = &ps {
            if ps.has_failed() {
                let (code, msg) = ps.get_failure_reason().unwrap_or((-1, "unknown".into()));
                return Err(format!("persistent subscription failed ({code}): {msg}").into());
            }
            work += ps.poll_fn(|_message, _header| received += 1, 100)?;
            if ps.is_live() && !was_live {
                was_live = true;
                println!("persistent subscription is live after {received} messages");
            }
        }

        // live messages, until it is live and every message is out
        let now = Instant::now();
        if ps.is_some() && (sent < MESSAGES || !was_live) && now >= next_send && send(&publication, sent)? {
            sent += 1;
            work += 1;
            next_send = now + SEND_INTERVAL;
        }
        // idle only when the cycle did nothing; a production agent would back off further
        if work == 0 {
            yield_now();
        }
    }
    if received != sent {
        return Err(format!("sent {sent} but received {received}").into());
    }
    println!("sent {sent}, received {received}, replay then live, through one duty cycle");

    if let Some(ps) = ps {
        ps.close()?;
    }
    Ok(())
}
