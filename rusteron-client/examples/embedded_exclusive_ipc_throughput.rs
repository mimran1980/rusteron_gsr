use rusteron_client::*;
use rusteron_media_driver::testing::EmbeddedDriver;
use std::error::Error;
use std::ffi::CStr;
use std::hint::spin_loop;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

const BURST_LENGTH: usize = 1_000_000;
const MESSAGE_LENGTH: usize = 32;
const FRAGMENT_COUNT_LIMIT: usize = 10;
static CHANNEL: &CStr = AERON_IPC_STREAM;
const STREAM_ID: i32 = 1001;

/// Port of Aeron's EmbeddedExclusiveIpcThroughput sample; runs until Ctrl-C.
///
/// Set `AERON_DIR` to use a running media driver; otherwise one is embedded.
fn main() -> Result<(), Box<dyn Error>> {
    // declared first so the driver outlives the client
    let driver = match std::env::var_os("AERON_DIR") {
        Some(_) => None,
        None => Some(EmbeddedDriver::launch()?),
    };
    let dir = match &driver {
        Some(driver) => driver.dir().to_string(),
        None => std::env::var("AERON_DIR")?,
    };

    let running = Arc::new(AtomicBool::new(true));

    println!("message length {MESSAGE_LENGTH}, channel {CHANNEL:?}");

    let running_ctrl_c = Arc::clone(&running);
    ctrlc::set_handler(move || {
        running_ctrl_c.store(false, Ordering::SeqCst);
    })?;

    let running_publisher = Arc::clone(&running);
    let running_subscriber = Arc::clone(&running);

    let ctx = AeronContext::new()?;
    ctx.set_error_handler(Some(|code: i32, msg: &str| eprintln!("aeron error {code}: {msg}")))?;
    ctx.set_dir(&cformat!("{dir}"))?;
    let aeron = Aeron::new(&ctx)?;
    aeron.start()?;

    let publication = aeron
        .async_add_exclusive_publication(CHANNEL, STREAM_ID)?
        .poll_blocking(Duration::from_secs(5))?;

    let subscription = aeron
        .async_add_subscription(CHANNEL, STREAM_ID, Handlers::NONE, Handlers::NONE)?
        .poll_blocking(Duration::from_secs(5))?;

    let subscriber_thread =
        thread::spawn(move || ImageRateSubscriber::new(running_subscriber, subscription, MESSAGE_LENGTH).run());

    let published = Publisher::new(running_publisher, publication).run();
    subscriber_thread.join().expect("subscriber thread panicked")?;
    published?;

    Ok(())
}

struct Publisher {
    running: Arc<AtomicBool>,
    publication: AeronExclusivePublication,
}

impl Publisher {
    fn new(running: Arc<AtomicBool>, publication: AeronExclusivePublication) -> Self {
        Publisher { running, publication }
    }

    fn run(&self) -> Result<(), AeronOfferError> {
        let mut back_pressure_count = 0u64;
        let mut total_message_count = 0u64;
        let buffer = vec![1u8; MESSAGE_LENGTH];

        'publish: while self.running.load(Ordering::Acquire) {
            loop {
                match self.publication.offer(&buffer) {
                    Ok(_) => break,
                    Err(e) if e.is_retryable() => {
                        if matches!(e, AeronOfferError::BackPressured | AeronOfferError::AdminAction) {
                            back_pressure_count += 1;
                        }
                        if !self.running.load(Ordering::Acquire) {
                            break 'publish;
                        }
                        spin_loop();
                    }
                    Err(e) => {
                        // stop the subscriber too, rather than leave it polling a dead stream
                        self.running.store(false, Ordering::Release);
                        return Err(e);
                    }
                }
            }
            total_message_count += 1;
        }

        if total_message_count > 0 {
            let back_pressure_ratio = back_pressure_count as f64 / total_message_count as f64;
            println!("Publisher back pressure ratio: {back_pressure_ratio:.6}");
        }
        Ok(())
    }
}

struct ImageRateSubscriber {
    running: Arc<AtomicBool>,
    subscription: AeronSubscription,
    message_length: usize,
    start_time: Instant,
}

impl ImageRateSubscriber {
    fn new(running: Arc<AtomicBool>, subscription: AeronSubscription, message_length: usize) -> Self {
        ImageRateSubscriber {
            running,
            subscription,
            message_length,
            start_time: Instant::now(),
        }
    }

    fn run(&mut self) -> Result<(), AeronCError> {
        let mut message_count = 0usize;
        while self.running.load(Ordering::Acquire) {
            let fragments = self
                .subscription
                .poll_fn(|_, _| message_count += 1, FRAGMENT_COUNT_LIMIT)?;
            if fragments == 0 {
                spin_loop();
            }

            if message_count >= BURST_LENGTH && self.start_time.elapsed() >= Duration::from_secs(1) {
                let elapsed = self.start_time.elapsed().as_secs_f64();
                let rate = message_count as f64 / elapsed;
                let throughput = rate * self.message_length as f64;

                use num_format::{Locale, ToFormattedString};
                println!(
                    "Throughput: {} msgs/sec, {} bytes/sec",
                    (rate.round() as u64).to_formatted_string(&Locale::en),
                    (throughput.round() as u64).to_formatted_string(&Locale::en)
                );

                self.start_time = Instant::now();
                message_count = 0;
            }
        }
        Ok(())
    }
}
