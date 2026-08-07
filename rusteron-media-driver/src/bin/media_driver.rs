use log::info;
use rusteron_media_driver::dpdk::configure_media_transport_from_env;
use rusteron_media_driver::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Flag to indicate when the application should stop (set on Ctrl+C)
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    // Register signal handler for SIGINT (Ctrl+C)
    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::SeqCst);
    })?;

    // Create Aeron context
    let aeron_context = AeronDriverContext::new()?;

    // Pin a reproducible driver dir for standalone acceptance runs (plan §11.3).
    // The caller removes the dir first — Aeron refuses to start over a live cnc.
    if let Ok(dir) = std::env::var("RUSTERON_MEDIA_DRIVER_DIR") {
        let cdir = std::ffi::CString::new(dir)?;
        aeron_context.set_dir(&cdir)?;
    }

    // Select the transport from `RUSTERON_MEDIA_DRIVER_TRANSPORT` after the
    // context exists and before the driver is created (plan §8). A selected
    // DPDK failure propagates as a nonzero exit; it never falls back to the
    // default socket driver.
    let _dpdk = configure_media_transport_from_env(&aeron_context)?;
    let backend = if _dpdk.is_some() { "dpdk-ena" } else { "socket" };
    println!("transport backend: {backend}");
    println!("aeron dir: {}", aeron_context.get_dir());
    aeron_context.print_configuration();

    // Create Aeron driver
    let aeron_driver = AeronDriver::new(&aeron_context)?;
    aeron_driver.start(true)?;
    // Start the Aeron driver
    println!("media driver started");
    info!("Aeron media driver started successfully. Press Ctrl+C to stop.");

    // Poll for work until Ctrl+C is pressed
    while running.load(Ordering::Acquire) {
        aeron_driver.main_idle_strategy(aeron_driver.main_do_work()?);
    }
    info!("Received signal to stop the media driver.");
    info!("Aeron media driver stopped successfully.");
    Ok(())
}
