#[cfg(test)]
mod tests {
    use super::super::*;
    use log::{error, info};
    use crate::testing::EmbeddedArchiveMediaDriverProcess;
    use serial_test::serial;
    use std::cell::Cell;
    use std::error::Error;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread::{self, sleep};
    use std::time::{Duration, Instant};
    use std::os::raw::c_int;

    #[derive(Default, Debug)]
    struct ErrorCount {
        error_count: usize,
    }

    impl AeronErrorHandlerCallback for ErrorCount {
        fn handle_aeron_error_handler(&mut self, error_code: c_int, msg: &str) {
            error!("Aeron error {}: {}", error_code, msg);
            self.error_count += 1;
        }
    }

    pub const ARCHIVE_CONTROL_REQUEST: &str = "aeron:udp?endpoint=localhost:8010";
    pub const ARCHIVE_CONTROL_RESPONSE: &str = "aeron:udp?endpoint=localhost:8011";
    pub const ARCHIVE_RECORDING_EVENTS: &str = "aeron:udp?control-mode=dynamic|control=localhost:8012";

    fn start_aeron_archive() -> Result<(Aeron, AeronArchiveContext, EmbeddedArchiveMediaDriverProcess), Box<dyn Error>> {
        let id = Aeron::nano_clock();
        let aeron_dir = format!("target/aeron/{}/shm", id);
        let archive_dir = format!("target/aeron/{}/archive", id);

        let request_port = find_unused_udp_port(8000).expect("Could not find port");
        let response_port = find_unused_udp_port(request_port + 1).expect("Could not find port");
        let recording_event_port = find_unused_udp_port(response_port + 1).expect("Could not find port");
        
        let request_control_channel = &format!("aeron:udp?endpoint=localhost:{}", request_port);
        let response_control_channel = &format!("aeron:udp?endpoint=localhost:{}", response_port);
        let recording_events_channel = &format!("aeron:udp?endpoint=localhost:{}", recording_event_port);

        let archive_media_driver = EmbeddedArchiveMediaDriverProcess::build_and_start(
            &aeron_dir,
            &archive_dir,
            request_control_channel,
            response_control_channel,
            recording_events_channel,
        ).expect("Failed to start Java process");

        let aeron_context = AeronContext::new()?;
        aeron_context.set_dir(&aeron_dir.into_c_string())?;
        aeron_context.set_client_name(&"test".into_c_string())?;
        
        let error_handler = Handler::leak(ErrorCount::default());
        aeron_context.set_error_handler(Some(&error_handler))?;
        
        let aeron = Aeron::new(&aeron_context)?;
        aeron.start()?;

        let archive_context = AeronArchiveContext::new_with_no_credentials_supplier(
            &aeron,
            request_control_channel,
            response_control_channel,
            recording_events_channel,
        )?;
        archive_context.set_error_handler(Some(&error_handler))?;
        
        Ok((aeron, archive_context, archive_media_driver))
    }

    fn find_unused_udp_port(start_port: u16) -> Option<u16> {
        for port in start_port..65535 {
            if std::net::UdpSocket::bind(("127.0.0.1", port)).is_ok() {
                return Some(port);
            }
        }
        None
    }

    #[test]
    #[serial]
    fn test_slow_consumer_replay() -> Result<(), Box<dyn Error>> {
        let _ = env_logger::Builder::new()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .try_init();

        EmbeddedArchiveMediaDriverProcess::kill_all_java_processes().ok();

        let (aeron, archive_context, _media_driver) = start_aeron_archive()?;
        
        let archive_connector = AeronArchiveAsyncConnect::new_with_aeron(&archive_context, &aeron)?;
        let archive = archive_connector.poll_blocking(Duration::from_secs(30))
            .expect("failed to connect to archive");

        let channel = "aeron:udp?endpoint=localhost:20121";
        let stream_id = 1001;

        // Start recording
        let subscription_id = archive.start_recording(
            &channel.into_c_string(),
            stream_id,
            SOURCE_LOCATION_LOCAL,
            true
        )?;
        info!("Started recording subscription_id={}", subscription_id);

        // Create publication
        let publication = aeron.async_add_exclusive_publication(
            &channel.into_c_string(),
            stream_id
        )?.poll_blocking(Duration::from_secs(5))?;

        // Wait for publication to be connected (recording started)
        let start = Instant::now();
        while !publication.is_connected() && start.elapsed() < Duration::from_secs(5) {
            sleep(Duration::from_millis(10));
        }
        assert!(publication.is_connected());

        // Find recording id
        let session_id = publication.get_constants()?.session_id;
        let counters_reader = aeron.counters_reader();
        let mut counter_id = -1;
        let start = Instant::now();
        while counter_id == -1 && start.elapsed() < Duration::from_secs(5) {
            counter_id = RecordingPos::find_counter_id_by_session(&counters_reader, session_id);
            sleep(Duration::from_millis(10));
        }
        assert!(counter_id >= 0, "Could not find recording counter");
        
        let recording_id = RecordingPos::get_recording_id(&counters_reader, counter_id)?;
        info!("Recording ID: {}", recording_id);

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let publication_clone = publication.clone();

        // Publisher thread
        let publisher_thread = thread::spawn(move || {
            let mut seq = 0u64;
            while running_clone.load(Ordering::Acquire) {
                let message = seq.to_le_bytes();
                while publication_clone.offer(
                    &message,
                    Handlers::no_reserved_value_supplier_handler(),
                ) <= 0 {
                    if !running_clone.load(Ordering::Acquire) { break; }
                    thread::yield_now();
                }
                seq += 1;
                // Publish at a reasonable rate
                sleep(Duration::from_millis(1)); 
            }
        });

        // Wait a bit for some data to be recorded
        sleep(Duration::from_secs(2));

        // Start Replay with length = -1 (follow mode)
        let replay_port = find_unused_udp_port(20000).unwrap();
        let replay_channel = format!("aeron:udp?endpoint=localhost:{}", replay_port);
        let replay_stream_id = 1002;
        let replay_params = AeronArchiveReplayParams::new(
            0, 
            i32::MAX, 
            0, 
            -1, // Follow mode
            0, 
            0
        )?;

        let replay_session_id = archive.start_replay(
            recording_id,
            &replay_channel.clone().into_c_string(),
            replay_stream_id,
            &replay_params
        )?;
        info!("Started replay session_id={}", replay_session_id);

        let replay_subscription_channel = format!("{}|session-id={}", replay_channel, replay_session_id as i32);
        let replay_subscription = aeron.async_add_subscription(
            &replay_subscription_channel.into_c_string(),
            replay_stream_id,
            Some(&Handler::leak(AeronAvailableImageLogger)),
            Some(&Handler::leak(AeronUnavailableImageLogger))
        )?.poll_blocking(Duration::from_secs(5))?;

        // Consumer loop
        let mut expected_seq = 0u64;
        let start_check = Instant::now();
        // Run for 1 minute as requested (or shorter for quick check, but user asked for 1 min)
        // I'll use 10 seconds for now to verify it works, then maybe bump it up or make it configurable.
        // User said "After a minute if there is no seq gap then test passes".
        let test_duration = Duration::from_secs(60); 
        
        struct FragmentHandler {
            expected_seq: u64,
            gaps: u64,
        }
        
        impl AeronFragmentHandlerCallback for FragmentHandler {
            fn handle_aeron_fragment_handler(&mut self, buffer: &[u8], _header: AeronHeader) {
                let received_seq = u64::from_le_bytes(buffer.try_into().unwrap());
                if received_seq != self.expected_seq {
                    error!("Gap detected! Expected {} but got {}", self.expected_seq, received_seq);
                    self.gaps += 1;
                    self.expected_seq = received_seq + 1; // Reset expectation to next
                } else {
                    self.expected_seq += 1;
                }
            }
        }

        let mut handler = FragmentHandler { expected_seq: 0, gaps: 0 };
        let mut handler_box = Handler::leak(handler); // Leak to pass to C callback safely

        info!("Starting slow consumer loop");
        while start_check.elapsed() < test_duration {
            let fragments = replay_subscription.poll(Some(&handler_box), 1)?;
            if fragments > 0 {
                // Simulate slow consumer
                sleep(Duration::from_secs(1));
            }
            
            if handler_box.gaps > 0 {
                break;
            }
        }

        running.store(false, Ordering::Release);
        publisher_thread.join().unwrap();

        if handler_box.gaps > 0 {
            panic!("Test failed: Sequence gaps detected!");
        }
        
        info!("Test passed: No sequence gaps detected.");
        Ok(())
    }
}
