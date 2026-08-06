//! Ticket 8 integration tests: standalone binary transport selection (plan §8).
//!
//! Each test spawns the real `media_driver` binary as a subprocess with a
//! chosen environment and asserts startup behaviour: which backend is logged,
//! whether a selected-DPDK failure exits nonzero, and that a failure never
//! falls back to the default socket driver.
//!
//! Every subprocess gets its own `AERON_DIR` so its driver state cannot collide
//! with another media driver in the workspace, and tests are `#[serial]`
//! (file-locked) like the other media-driver integration tests. Selector
//! parsing runs on every platform; the install path (incomplete/native-failure
//! cases) is `#[cfg(feature = "dpdk")]` because that is where the native code
//! exists.

use serial_test::serial;

use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Unique Aeron dir per test, in the platform temp dir (works on Linux and
/// macOS; `/dev/shm` does not exist on macOS).
fn aeron_dir(tag: &str) -> String {
    format!("{}/aeron-t8-{tag}", std::env::temp_dir().display())
}

fn spawn(envs: &[(&str, &str)]) -> Child {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_media_driver"));
    cmd.env("AERON_DIR_DELETE_ON_START", "true")
        .env("AERON_DIR_DELETE_ON_SHUTDOWN", "true");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn the media_driver binary")
}

/// Wait (bounded) for a process to exit; panic on timeout.
fn wait_exit(child: &mut Child) -> std::process::ExitStatus {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(status) = child.try_wait().expect("try_wait failed") {
            return status;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("media_driver subprocess did not exit within 30s");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Run the binary to completion, draining stdout and stderr, and return
/// (exit status, stdout, stderr).
fn spawn_and_capture(envs: &[(&str, &str)]) -> (std::process::ExitStatus, String, String) {
    let mut child = spawn(envs);
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let (out_tx, out_rx) = mpsc::channel();
    let (err_tx, err_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut s = String::new();
        let _ = BufReader::new(stdout).read_to_string(&mut s);
        let _ = out_tx.send(s);
    });
    std::thread::spawn(move || {
        let mut s = String::new();
        let _ = BufReader::new(stderr).read_to_string(&mut s);
        let _ = err_tx.send(s);
    });

    let status = wait_exit(&mut child);
    let stdout = out_rx.recv().unwrap_or_default();
    let stderr = err_rx.recv().unwrap_or_default();
    (status, stdout, stderr)
}

/// Spawn the binary and wait for `needle` to appear on stdout (the backend log
/// line is printed before the driver starts). Returns the running child so the
/// caller can stop it.
fn spawn_and_wait_for_line(envs: &[(&str, &str)], needle: &str) -> Child {
    let mut child = spawn(envs);
    let stdout = child.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let line = line.unwrap_or_default();
            if tx.send(line).is_err() {
                break;
            }
        }
    });

    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match rx.recv_timeout(Duration::from_millis(250)) {
            Ok(line) if line.contains(needle) => return child,
            Ok(_) => {}
            Err(_) if Instant::now() > deadline => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("timed out waiting for {needle:?} in media_driver output");
            }
            Err(_) => {}
        }
    }
}

fn send_sigint(child: &Child) {
    let pid = child.id().to_string();
    let status = Command::new("kill")
        .args(["-INT", &pid])
        .status()
        .expect("failed to run kill -INT");
    assert!(status.success(), "kill -INT {pid} failed");
}

// --- Default path: absent / `default` selector --------------------------------

#[serial]
#[test]
fn absent_selector_starts_socket_backend() {
    let dir = aeron_dir("absent");
    let envs = [("AERON_DIR", dir.as_str())];
    let mut child = spawn_and_wait_for_line(&envs, "transport backend: socket");
    send_sigint(&child);
    let status = wait_exit(&mut child);
    assert_eq!(status.code(), Some(0), "graceful SIGINT shutdown must exit 0");
}

#[serial]
#[test]
fn default_selector_starts_socket_backend() {
    let dir = aeron_dir("default");
    let envs = [
        ("AERON_DIR", dir.as_str()),
        ("RUSTERON_MEDIA_DRIVER_TRANSPORT", "default"),
    ];
    let mut child = spawn_and_wait_for_line(&envs, "transport backend: socket");
    send_sigint(&child);
    let status = wait_exit(&mut child);
    assert_eq!(status.code(), Some(0), "graceful SIGINT shutdown must exit 0");
}

// --- Unknown selector ---------------------------------------------------------

#[serial]
#[test]
fn unknown_selector_exits_nonzero_without_launching_a_driver() {
    let dir = aeron_dir("unknown");
    let envs = [
        ("AERON_DIR", dir.as_str()),
        ("RUSTERON_MEDIA_DRIVER_TRANSPORT", "vanilla"),
    ];
    let (status, _out, stderr) = spawn_and_capture(&envs);
    assert_ne!(status.code(), Some(0), "unknown selector must not launch a driver");
    assert!(
        stderr.contains("RUSTERON_MEDIA_DRIVER_TRANSPORT"),
        "stderr must name the selector variable: {stderr}"
    );
}

// --- Selected-DPDK failures (feature-gated: native code exists only with it) --

#[cfg(feature = "dpdk")]
#[serial]
#[test]
fn selected_dpdk_with_incomplete_env_exits_nonzero() {
    let dir = aeron_dir("incomplete");
    // Every required §6.2/§6.3 variable except the sender PCI, which is
    // deliberately absent so environment parsing fails deterministically.
    let mut envs = vec![
        ("AERON_DIR", dir.as_str()),
        ("RUSTERON_MEDIA_DRIVER_TRANSPORT", "dpdk-ena"),
        ("RUSTERON_DPDK_FILE_PREFIX", "rusteron-ena"),
        ("RUSTERON_DPDK_SENDER_IPV4_CIDR", "10.0.0.1/24"),
        ("RUSTERON_DPDK_SENDER_GATEWAY", "10.0.0.254"),
        ("RUSTERON_DPDK_RECEIVER_PCI", "0000:00:02.0"),
        ("RUSTERON_DPDK_RECEIVER_IPV4_CIDR", "10.0.1.1/24"),
        ("RUSTERON_DPDK_RECEIVER_GATEWAY", "10.0.1.254"),
    ];
    envs.retain(|(k, _)| *k != "RUSTERON_DPDK_SENDER_PCI");
    let (status, _out, stderr) = spawn_and_capture(&envs);
    assert_ne!(status.code(), Some(0), "missing DPDK env must not launch a driver");
    assert!(
        stderr.contains("RUSTERON_DPDK_SENDER_PCI"),
        "stderr must name the missing variable: {stderr}"
    );
}

#[cfg(feature = "dpdk")]
#[serial]
#[test]
fn selected_dpdk_native_failure_exits_nonzero() {
    let dir = aeron_dir("native");
    // Full §6.2/§6.3 env plus the §6.4 context env (distinct CPUs, spin idle,
    // disjoint wildcard port ranges), so validation passes and the native EAL
    // init runs — and fails in the container, which has no hugepages or ENAs.
    let envs = [
        ("AERON_DIR", dir.as_str()),
        ("RUSTERON_MEDIA_DRIVER_TRANSPORT", "dpdk-ena"),
        ("RUSTERON_DPDK_FILE_PREFIX", "rusteron-ena"),
        ("RUSTERON_DPDK_SENDER_PCI", "0000:00:01.0"),
        ("RUSTERON_DPDK_SENDER_IPV4_CIDR", "10.0.0.1/24"),
        ("RUSTERON_DPDK_SENDER_GATEWAY", "10.0.0.254"),
        ("RUSTERON_DPDK_RECEIVER_PCI", "0000:00:02.0"),
        ("RUSTERON_DPDK_RECEIVER_IPV4_CIDR", "10.0.1.1/24"),
        ("RUSTERON_DPDK_RECEIVER_GATEWAY", "10.0.1.254"),
        ("AERON_SENDER_CPU_AFFINITY", "1"),
        ("AERON_RECEIVER_CPU_AFFINITY", "2"),
        ("AERON_SENDER_IDLE_STRATEGY", "spin"),
        ("AERON_RECEIVER_IDLE_STRATEGY", "spin"),
        ("AERON_SENDER_WILDCARD_PORT_RANGE", "20000-20999"),
        ("AERON_RECEIVER_WILDCARD_PORT_RANGE", "21000-21999"),
        ("AERON_MTU_LENGTH", "1408"),
    ];
    let (status, _out, stderr) = spawn_and_capture(&envs);
    assert_ne!(
        status.code(),
        Some(0),
        "a selected-DPDK native failure must not launch the default driver"
    );
    assert!(!stderr.is_empty(), "stderr must report the native failure");
}

// --- dpdk-ena selected without the feature (runs everywhere without `dpdk`) ---

#[cfg(not(feature = "dpdk"))]
#[serial]
#[test]
fn selected_dpdk_without_feature_exits_nonzero() {
    let dir = aeron_dir("disabled");
    let envs = [
        ("AERON_DIR", dir.as_str()),
        ("RUSTERON_MEDIA_DRIVER_TRANSPORT", "dpdk-ena"),
    ];
    let (status, _out, stderr) = spawn_and_capture(&envs);
    assert_ne!(status.code(), Some(0), "dpdk-ena without the feature must not launch a driver");
    // Rust's `main` Err path prints the Debug form (`Error: {err:?}`).
    assert!(
        stderr.contains("FeatureDisabled"),
        "stderr must name the missing feature: {stderr}"
    );
}
