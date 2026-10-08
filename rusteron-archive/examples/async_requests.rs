//! # Non-blocking archive requests
//!
//! A blocking archive call waits on the calling thread for the archive's answer, a full
//! round trip to an archive that may be remote. Its async form sends the request and
//! returns, and each `poll` takes only what has arrived, so a duty cycle never stalls.
//! This example times both for the same work: listing recordings, starting a replay, and
//! building a persistent subscription.
//!
//! Requires `java` on PATH (an embedded Java Archive is started for you).
//!
//! ```bash
//! cargo run --release --example async_requests
//! ```

use rusteron_archive::testing::{EmbeddedArchiveMediaDriverProcess, find_unused_udp_port};
use rusteron_archive::*;
use std::thread::sleep;
use std::time::{Duration, Instant};

const ROUNDS: i32 = 10;

/// Calls `poll` until it returns a result: the result, the longest single call, and the
/// time until the result.
fn poll_until_done<T>(
    mut poll: impl FnMut() -> Result<Option<T>, AeronCError>,
) -> Result<(T, Duration, Duration), AeronCError> {
    let (start, mut longest) = (Instant::now(), Duration::ZERO);
    loop {
        let call = Instant::now();
        let result = poll()?;
        longest = longest.max(call.elapsed());
        if let Some(value) = result {
            return Ok((value, longest, start.elapsed()));
        }
        std::thread::yield_now();
    }
}

/// The rounds of one request: the blocking call, and the async form's longest single call
/// (sending or polling) and time until its result.
#[derive(Default)]
struct Timings {
    blocking: Vec<Duration>,
    longest_call: Vec<Duration>,
    async_total: Vec<Duration>,
}

impl Timings {
    fn add_async(&mut self, sent: Duration, longest_poll: Duration, total: Duration) {
        self.longest_call.push(sent.max(longest_poll));
        self.async_total.push(sent + total);
    }

    /// Prints the median blocking call, the worst async call and the median async total.
    fn report(mut self, request: &str) {
        let median = |times: &mut Vec<Duration>| {
            times.sort();
            times[times.len() / 2]
        };
        let worst = self.longest_call.iter().max().copied().unwrap_or_default();
        println!(
            "{request:<24} {:>14.1?} {:>16.1?} {:>14.1?}",
            median(&mut self.blocking),
            worst,
            median(&mut self.async_total)
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    EmbeddedArchiveMediaDriverProcess::kill_all_java_processes().ok();

    let id = Aeron::nano_clock();
    let aeron_dir = format!("target/aeron/{id}_async/shm");
    let archive_dir = format!("target/aeron/{id}_async/archive");
    let req_port = find_unused_udp_port(9400).expect("no free port");
    let resp_port = find_unused_udp_port(req_port + 1).expect("no free port");
    let events_port = find_unused_udp_port(resp_port + 1).expect("no free port");

    // Keep `_process` in scope so the Java Archive lives for the whole example.
    let _process = EmbeddedArchiveMediaDriverProcess::build_and_start(
        &aeron_dir,
        &archive_dir,
        &format!("aeron:udp?endpoint=localhost:{req_port}"),
        &format!("aeron:udp?endpoint=localhost:{resp_port}"),
        &format!("aeron:udp?endpoint=localhost:{events_port}"),
    )?;

    let aeron_context = AeronContext::new()?;
    aeron_context.set_dir(&cformat!("{aeron_dir}"))?;
    let aeron = Aeron::new(&aeron_context)?;
    aeron.start()?;

    let archive_context = AeronArchiveContext::new()?;
    archive_context.set_aeron(&aeron)?;
    archive_context.set_control_request_channel(&cformat!("aeron:udp?endpoint=localhost:{req_port}"))?;
    archive_context.set_control_response_channel(&cformat!("aeron:udp?endpoint=localhost:{resp_port}"))?;
    archive_context.set_recording_events_channel(&cformat!("aeron:udp?endpoint=localhost:{events_port}"))?;
    let archive =
        AeronArchiveAsyncConnect::new_with_aeron(&archive_context, &aeron)?.poll_blocking(Duration::from_secs(20))?;

    // A recording to list, replay and subscribe to.
    let stream_id = 1101;
    archive.start_recording(c"aeron:ipc", stream_id, SOURCE_LOCATION_LOCAL, true)?;
    let publication = aeron
        .async_add_publication(c"aeron:ipc", stream_id)?
        .poll_blocking(Duration::from_secs(5))?;
    while !publication.is_connected() {
        sleep(Duration::from_millis(10));
    }
    for i in 0..10 {
        let message = format!("History-{i}");
        loop {
            match publication.offer(message.as_bytes()) {
                Ok(_) => break,
                Err(e) if e.is_retryable() => sleep(Duration::from_millis(1)),
                Err(e) => return Err(e.into()),
            }
        }
    }
    let counters = aeron.counters_reader();
    let counter_id = RecordingPos::find_counter_id_by_session(&counters, publication.get_constants()?.session_id);
    let recording_id = RecordingPos::get_recording_id_block(&counters, counter_id, Duration::from_secs(5))?;
    let deadline = Instant::now() + Duration::from_secs(5);
    while counters.get_counter_value(counter_id) < publication.position() && Instant::now() < deadline {
        sleep(Duration::from_millis(10));
    }

    println!("{ROUNDS} rounds each; times are medians, but the longest async call is the worst seen");
    println!(
        "{:<24} {:>14} {:>16} {:>14}",
        "request", "blocking call", "longest async call", "async total"
    );

    let mut list = Timings::default();
    for _ in 0..ROUNDS {
        let start = Instant::now();
        archive.list_recordings_fn(&mut 0, 0, 100, |_| {})?;
        list.blocking.push(start.elapsed());

        let start = Instant::now();
        let mut request = archive.async_list_recordings(0, 100, |_| {})?;
        let sent = start.elapsed();
        let (_, longest, total) = poll_until_done(|| request.poll())?;
        list.add_async(sent, longest, total);
    }
    list.report("list recordings");

    let params = AeronArchiveReplayParams::builder()
        .position(0)
        .length(publication.position())
        .build()?;
    let mut replay = Timings::default();
    for round in 0..ROUNDS {
        let start = Instant::now();
        let session = archive.start_replay(recording_id, c"aeron:ipc", 2001 + 2 * round, &params)?;
        replay.blocking.push(start.elapsed());
        let _ = archive.stop_replay(session);

        let start = Instant::now();
        let mut request = archive.async_start_replay(recording_id, c"aeron:ipc", 2002 + 2 * round, &params)?;
        let sent = start.elapsed();
        let (session, longest, total) = poll_until_done(|| request.poll())?;
        replay.add_async(sent, longest, total);
        drop(request);
        let _ = archive.stop_replay(session);
    }
    replay.report("start replay");

    let configured = |replay_stream_id: i32| {
        PersistentSubscriptionBuilder::new_with_aeron(&archive_context, &aeron)?
            .recording_id(recording_id)?
            .live_channel("aeron:ipc")?
            .live_stream_id(stream_id)?
            .replay_channel("aeron:ipc")?
            .replay_stream_id(replay_stream_id)
    };
    let mut subscribe = Timings::default();
    for round in 0..ROUNDS {
        let start = Instant::now();
        let subscription = configured(3001 + 2 * round)?.build()?;
        subscribe.blocking.push(start.elapsed());
        subscription.close()?;

        let start = Instant::now();
        let mut building = configured(3002 + 2 * round)?.build_async()?;
        let sent = start.elapsed();
        let (subscription, longest, total) = poll_until_done(|| building.poll())?;
        subscribe.add_async(sent, longest, total);
        subscription.close()?;
    }
    subscribe.report("persistent subscription");
    Ok(())
}
