use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use rusteron_media_driver::media_driver::{AeronContext, AeronDriver};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Flag to indicate when the application should stop (set on Ctrl+C)
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    // Register signal handler for SIGINT (Ctrl+C)
    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::SeqCst);
    })?;

    // Create Aeron context
    let aeron_context = AeronContext::new()?;
    aeron_context.print_config()?;

    // Create Aeron driver
    let aeron_driver = AeronDriver::new(&aeron_context)?;

    // Start the Aeron driver
    aeron_driver.start()?;
    println!("Aeron media driver started successfully. Press Ctrl+C to stop.");

    // Poll for work until Ctrl+C is pressed
    while running.load(Ordering::Acquire) {
        aeron_driver.do_work();
    }

    println!("Received signal to stop the media driver.");
    println!("Aeron media driver stopped successfully.");
    Ok(())
}
