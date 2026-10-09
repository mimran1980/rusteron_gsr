use std::ffi::CString;
use std::time::{Duration, Instant};

/// Pins the calling thread to `cpu`; `None` leaves it where it is.
#[cfg(target_os = "linux")]
pub fn pin(cpu: Option<usize>) {
    let Some(cpu) = cpu else { return };
    // SAFETY: a zeroed cpu_set_t is the empty set, and the call only reads it.
    unsafe {
        let mut set: libc::cpu_set_t = std::mem::zeroed();
        libc::CPU_SET(cpu, &mut set);
        let rc = libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set);
        assert_eq!(rc, 0, "could not pin to cpu {cpu}");
    }
}

/// Thread pinning is Linux only; the Mac runs unpinned for smoke tests.
#[cfg(not(target_os = "linux"))]
pub fn pin(_cpu: Option<usize>) {}

/// `-` or a CPU number.
pub fn cpu_arg(arg: Option<String>) -> Option<usize> {
    arg.filter(|a| a != "-").map(|a| a.parse().expect("cpu number or -"))
}

/// The external driver's dir, from `AERON_DIR`.
pub fn aeron_dir() -> CString {
    CString::new(std::env::var("AERON_DIR").expect("AERON_DIR: the external driver's dir")).unwrap()
}

/// `LABEL`, which names the run in the result line.
pub fn label() -> String {
    std::env::var("LABEL").unwrap_or_else(|_| "-".into())
}

/// Sleeps on `ready` until it holds, failing after `secs`.
pub fn wait_until(what: &str, secs: u64, mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(secs);
    while !ready() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(1));
    }
}
