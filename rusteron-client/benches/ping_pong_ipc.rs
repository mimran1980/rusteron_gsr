//! IPC ping/pong RTT across two processes sharing one embedded driver: this binary pings,
//! and a copy of it started with `--pong <dir>` echoes until it is killed.

mod ping_pong_common;

use criterion::Criterion;
use ping_pong_common::{Channels, run_ping, run_pong};
use rusteron_client::AERON_IPC_STREAM;
use rusteron_media_driver::testing::EmbeddedDriver;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const CHANNELS: Channels = Channels {
    ping: AERON_IPC_STREAM,
    pong: AERON_IPC_STREAM,
};

fn criterion_benchmark(c: &mut Criterion) {
    // declared first so it drops last: it stops, joins and deletes its dir
    let driver = EmbeddedDriver::launch_with(|ctx| {
        ctx.set_print_configuration(true)?;
        Ok(())
    })
    .expect("launch embedded driver");
    let mut pong_child = spawn_pong_process(driver.dir()).expect("spawn pong process");

    // the pong process has to start and connect first
    let connect_timeout = Duration::from_secs(10);
    let ping = run_ping(
        c,
        driver.dir(),
        &CHANNELS,
        "ping_pong_ipc_process_benchmark",
        connect_timeout,
    );
    if let Err(e) = pong_child.kill() {
        eprintln!("Failed to kill pong child: {e}");
    }
    let _ = pong_child.wait();
    ping.expect("ping failed");
}

fn spawn_pong_process(dir: &str) -> std::io::Result<Child> {
    let exe = std::env::current_exe()?;
    Command::new(exe)
        .arg("--pong")
        .arg(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
}

fn main() {
    let mut args = std::env::args().skip(1);
    if let Some(dir) = args.position(|arg| arg == "--pong").and_then(|_| args.next()) {
        // runs until the parent kills it
        if let Err(e) = run_pong(&dir, &CHANNELS, || true) {
            eprintln!("Pong process error: {e}");
            std::process::exit(1);
        }
        return;
    }

    let mut c = Criterion::default().configure_from_args();
    criterion_benchmark(&mut c);
    c.final_summary();
}
