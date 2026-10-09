//! # Duty cycle
//!
//! One thread drives an Aeron client with no conductor thread (the agent invoker), its
//! archive client and a persistent subscription. Each cycle calls `archive.do_work()`, which
//! runs the client's conductor and hands recording signals to their consumer, then polls the
//! persistent subscription and offers the next message. Setup uses blocking calls, which run
//! the conductor themselves.
//!
//! Requires `java` on `PATH`: the `rusteron_archive::testing` harness starts a Java Archive.
//!
//! ```bash
//! cargo run --release --features "static precompile" --example duty_cycle
//! ```

use rusteron_archive::testing::{
    EmbeddedArchiveMediaDriverProcess, find_counter_id_by_session_blocking, find_unused_udp_port,
};
use rusteron_archive::*;
use std::thread::yield_now;
use std::time::{Duration, Instant};

const STREAM_ID: i32 = 1301;
const MESSAGES: usize = 100;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    EmbeddedArchiveMediaDriverProcess::kill_all_java_processes().ok();
    let id = Aeron::nano_clock();
    let aeron_dir = format!("target/aeron/{id}_example/shm");
    let req_port = find_unused_udp_port(9000).ok_or("no free port")?;
    let resp_port = find_unused_udp_port(req_port + 1).ok_or("no free port")?;
    let events_port = find_unused_udp_port(resp_port + 1).ok_or("no free port")?;
    let (request, response) = (
        cformat!("aeron:udp?endpoint=localhost:{req_port}"),
        cformat!("aeron:udp?endpoint=localhost:{resp_port}"),
    );
    let _process = EmbeddedArchiveMediaDriverProcess::build_and_start(
        &aeron_dir,
        &format!("target/aeron/{id}_example/archive"),
        request.to_str()?,
        response.to_str()?,
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
    archive_ctx.set_control_request_channel(&request)?;
    archive_ctx.set_control_response_channel(&response)?;
    archive_ctx.set_error_handler(Some(|code: i32, msg: &str| eprintln!("[archive] {code}: {msg}")))?;
    archive_ctx.set_recording_signal_consumer(Some(|s: AeronArchiveRecordingSignal| {
        println!("recording signal {:?}", s.signal());
    }))?;
    let archive =
        AeronArchiveAsyncConnect::new_with_aeron(&archive_ctx, &aeron)?.poll_blocking(Duration::from_secs(20))?;

    archive.start_recording(AERON_IPC_STREAM, STREAM_ID, SOURCE_LOCATION_LOCAL, true)?;
    let publication = aeron
        .async_add_exclusive_publication(AERON_IPC_STREAM, STREAM_ID)?
        .poll_blocking(Duration::from_secs(10))?;
    let counters = aeron.counters_reader();
    let session_id = publication.get_constants()?.session_id;
    let counter_id = find_counter_id_by_session_blocking(&counters, session_id, Duration::from_secs(10))?;
    let ps = PersistentSubscriptionBuilder::new_with_aeron(&archive_ctx, &aeron)?
        .live_channel(AERON_IPC_STREAM.to_str()?)?
        .live_stream_id(STREAM_ID)?
        .replay_channel("aeron:udp?endpoint=localhost:0")?
        .replay_stream_id(STREAM_ID + 1)?
        .start_from_beginning()?
        .recording_id(RecordingPos::get_recording_id(&counters, counter_id)?)?
        .build()?;

    let (mut sent, mut received) = (0, 0);
    let deadline = Instant::now() + Duration::from_secs(30);
    while received < MESSAGES {
        if Instant::now() > deadline {
            return Err(format!("received {received} of {MESSAGES}").into());
        }
        let mut work = archive.do_work()?;
        // a lost archive is not an error from do_work
        if !archive.get_control_response_subscription().is_connected() {
            return Err("lost the archive".into());
        }
        if ps.has_failed() {
            return Err(format!("persistent subscription failed: {:?}", ps.get_failure_reason()).into());
        }
        work += ps.poll_fn(|_message, _header| received += 1, 100)?;
        if sent < MESSAGES {
            match publication.offer(format!("message {sent}").as_bytes()) {
                Ok(_) => {
                    sent += 1;
                    work += 1;
                }
                Err(e) if e.is_retryable() => {}
                Err(e) => return Err(e.into()),
            }
        }
        // idle only when the cycle did nothing; a production agent would back off further
        if work == 0 {
            yield_now();
        }
    }
    println!("sent {sent}, received {received} through one duty cycle");
    ps.close()?;
    Ok(())
}
