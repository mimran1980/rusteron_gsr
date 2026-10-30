//! What one client poll pass and each media driver duty cycle cost as the number of UDP
//! streams grows, and where a round trip's time goes at many streams.
//!
//! The driver runs in INVOKER mode and this bench drives its conductor, sender and receiver
//! from three pinned threads, which is what dedicated threading does, so it can time every
//! duty cycle with the TSC. Each stream is its own channel, publication and subscription.
//!
//! Arms (`MANY_STREAMS_ARMS`, default all):
//! - `idle`: N connected streams, no traffic. Times client passes and driver cycles.
//! - `plain`: N ping streams and N pong streams, `MANY_STREAMS_RATE` msg/s in total, round
//!   robin; ping polls every pong subscription each pass, pong every ping subscription.
//! - `mux`: N logical streams keyed in the message over one Aeron stream each way.
//! - `driver`: one active stream each way plus N-1 quiet UDP streams each way that nobody
//!   polls: the driver's share of the cost of N streams.
//! - `client`: one active stream each way plus N-1 quiet IPC subscriptions each side polls:
//!   the client's share.
//! - `ready`: as `plain`, with the driver's readiness interceptor installed; each side polls
//!   only the subscriptions whose readiness bit is set ([`AeronReadiness`]).
//! - `mark`: the interceptor's cost per data datagram, without a driver.
//!
//! One driver serves both ping and pong, so in the traffic arms it holds 2N publications and
//! 2N images. Other settings: `MANY_STREAMS_IDLE_N`, `MANY_STREAMS_TRAFFIC_N`,
//! `MANY_STREAMS_SECS`, `MANY_STREAMS_CPUS` (housekeeping and conductor, sender, receiver,
//! ping, pong) and `MANY_STREAMS_OUT`.
//!
//! Every pass, cycle and round trip goes into a histogram; each one over 1 µs is also kept
//! with its timestamp so maxima can be matched across threads, and each over 100 µs records
//! whether the thread was preempted. Output: a table on stdout, `summary.tsv`, and per
//! thread `slow-<arm>-<n>-<role>.tsv` and `stalls-<arm>-<n>-<role>.tsv` under
//! `MANY_STREAMS_OUT`. Run it on Linux x86_64; elsewhere it runs unpinned on `Instant`.

use hdrhistogram::Histogram;
use rusteron_client::*;
use rusteron_media_driver::bindings::{
    aeron_driver_conductor_do_work, aeron_driver_native_resource_agent_do_work, aeron_driver_receiver_do_work,
    aeron_driver_sender_do_work, aeron_driver_t, aeron_threading_mode_enum,
};
use rusteron_media_driver::readiness::mark_ready;
use rusteron_media_driver::{AeronDriver, AeronDriverContext};
use std::error::Error;
use std::ffi::{CString, c_void};
use std::fmt::Write as _;
use std::hint::black_box;
use std::io::Write as _;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::thread::{self, JoinHandle, sleep};
use std::time::{Duration, Instant};

type Res<T> = Result<T, Box<dyn Error + Send + Sync>>;

const MESSAGE_LENGTH: usize = 32;
const FRAGMENT_LIMIT: usize = 16;
const SLOW_NS: u64 = 1_000;
const SLOW_KEEP: usize = 500_000;
const CYCLE_THRESHOLD_NS: u64 = 10_000;
const STALL_NS: u64 = 100_000;
const PING_PORT: u16 = 20_000;
const PONG_PORT: u16 = 22_000;
const PING_STREAM: i32 = 1_000;
/// Ping streams map to bits 232..=255 and 0..=103 of 256, pong streams to 104..=231, so in the
/// `ready` arm the two sides of the one driver never share a readiness bit.
const PONG_STREAM: i32 = PING_STREAM + 128 + 256 * 7;
const READINESS_WORDS: usize = 4;
const IPC_PING_STREAM: i32 = 5_000;
const IPC_PONG_STREAM: i32 = 7_000;

fn main() -> Res<()> {
    let cfg = Config::from_env()?;
    // everything not pinned elsewhere, the clients' conductor threads included, inherits this
    pin(cfg.cpus[0]);
    std::fs::create_dir_all(&cfg.out)?;
    let clock = Clock::calibrate();
    println!("tsc: {:.4} ns/tick; out {}", clock.ns_per_tick, cfg.out);
    let mut summary = String::from(
        "arm\tn\trole\tcount\tp50_ns\tp99_ns\tp999_ns\tmax_ns\tover_1us\tvcsw\tnvcsw\tminflt\tmajflt\tmax_at_ms\tmax_work\n",
    );
    if cfg.mark {
        for row in mark_cost(&cfg, &clock) {
            println!("{}", row.line());
            summary.push_str(&row.tsv());
        }
    }
    for arm in &cfg.arms {
        let ns = if matches!(arm, Arm::Idle(_)) {
            &cfg.idle_n
        } else {
            &cfg.traffic_n
        };
        for &n in ns {
            let rows = run_arm(&cfg, &clock, *arm, n)?;
            for row in rows {
                println!("{}", row.line());
                summary.push_str(&row.tsv());
            }
        }
    }
    std::fs::write(format!("{}/summary.tsv", cfg.out), summary)?;
    let memory = MEMORY.lock().map_err(|_| "memory rows")?;
    std::fs::write(
        format!("{}/memory.tsv", cfg.out),
        format!(
            "arm\tn\tlog_files\tlog_apparent_bytes\tlog_allocated_bytes\trss\trss_anon\trss_file\trss_shmem\n{memory}"
        ),
    )?;
    Ok(())
}

/// How N Aeron streams map to channels: a port each and a stream id each (`PerPort`), one
/// port and a stream id each (`SamePort`), or a port each and one stream id (`SameStream`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Layout {
    PerPort,
    SamePort,
    SameStream,
}

impl Layout {
    fn port(self, base: u16, i: usize) -> u16 {
        if self == Layout::SamePort {
            base
        } else {
            base + i as u16
        }
    }

    fn stream(self, base: i32, i: usize) -> i32 {
        if self == Layout::SameStream {
            base
        } else {
            base + i as i32
        }
    }

    fn suffix(self) -> &'static str {
        match self {
            Layout::PerPort => "",
            Layout::SamePort => "sameport",
            Layout::SameStream => "samestream",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Idle(Layout),
    Plain(Layout),
    /// N logical streams keyed in the message over this many Aeron streams on one port.
    Mux(usize),
    Driver,
    Client,
    Ready,
    /// N per-port streams split evenly over this many drivers, ping and pong on one thread;
    /// `shared` runs each driver's sender and receiver on one thread.
    Shard {
        drivers: usize,
        shared: bool,
    },
    /// Not a stream arm: labels the `mark` timings.
    Mark,
}

impl Arm {
    fn name(self) -> String {
        match self {
            Arm::Idle(Layout::PerPort) => "idle".into(),
            Arm::Idle(l) => format!("idle-{}", l.suffix()),
            Arm::Plain(Layout::PerPort) => "plain".into(),
            Arm::Plain(l) => l.suffix().into(),
            Arm::Mux(1) => "mux".into(),
            Arm::Mux(k) => format!("mux{k}"),
            Arm::Driver => "driver".into(),
            Arm::Client => "client".into(),
            Arm::Ready => "ready".into(),
            Arm::Shard { drivers, shared: false } => format!("shard{drivers}"),
            Arm::Shard { drivers, shared: true } => format!("snshard{drivers}"),
            Arm::Mark => "mark".into(),
        }
    }

    fn parse(name: &str) -> Res<Self> {
        Ok(match name {
            "idle" => Arm::Idle(Layout::PerPort),
            "idle-sameport" => Arm::Idle(Layout::SamePort),
            "idle-samestream" => Arm::Idle(Layout::SameStream),
            "plain" => Arm::Plain(Layout::PerPort),
            "sameport" => Arm::Plain(Layout::SamePort),
            "samestream" => Arm::Plain(Layout::SameStream),
            "mux" => Arm::Mux(1),
            "driver" => Arm::Driver,
            "client" => Arm::Client,
            "ready" => Arm::Ready,
            other => {
                let number = |prefix: &str| {
                    other
                        .strip_prefix(prefix)
                        .and_then(|k| k.parse::<usize>().ok())
                        .filter(|&k| k > 0)
                };
                if let Some(k) = number("mux") {
                    Arm::Mux(k)
                } else if let Some(drivers) = number("snshard") {
                    Arm::Shard { drivers, shared: true }
                } else if let Some(drivers) = number("shard") {
                    Arm::Shard { drivers, shared: false }
                } else {
                    return Err(format!("unknown arm {other}").into());
                }
            }
        })
    }
}

struct Config {
    /// Time the readiness interceptor's mark per datagram (arm `mark`).
    mark: bool,
    arms: Vec<Arm>,
    idle_n: Vec<usize>,
    traffic_n: Vec<usize>,
    secs: u64,
    rate: u64,
    cpus: Vec<usize>,
    out: String,
}

impl Config {
    fn from_env() -> Res<Self> {
        let var = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());
        let list = |s: String| -> Res<Vec<usize>> { s.split(',').map(|v| Ok(v.trim().parse()?)).collect() };
        let arm_list = var("MANY_STREAMS_ARMS", "mark,idle,plain,mux,driver,client,ready");
        let mark = arm_list.split(',').any(|a| a.trim() == "mark");
        let arms = arm_list
            .split(',')
            .filter(|a| a.trim() != "mark")
            .map(|a| Arm::parse(a.trim()))
            .collect::<Res<Vec<_>>>()?;
        let cpus = list(var("MANY_STREAMS_CPUS", "0,1,2,3,4"))?;
        if cpus.len() != 5 {
            return Err("MANY_STREAMS_CPUS needs 5 cpus: housekeeping+conductor,sender,receiver,ping,pong".into());
        }
        Ok(Self {
            mark,
            arms,
            idle_n: list(var("MANY_STREAMS_IDLE_N", "1,4,16,64,128,256"))?,
            traffic_n: list(var("MANY_STREAMS_TRAFFIC_N", "1,32,128"))?,
            secs: var("MANY_STREAMS_SECS", "10").parse()?,
            rate: var("MANY_STREAMS_RATE", "10000").parse()?,
            cpus,
            out: var("MANY_STREAMS_OUT", "target/many-streams"),
        })
        .inspect(|_| CONCURRENT.store(var("MANY_STREAMS_CONCURRENT", "0") == "1", Ordering::Relaxed))
    }
}

/// TSC ticks, converted to ns with a ratio measured against `Instant`.
struct Clock {
    ns_per_tick: f64,
    epoch: u64,
}

impl Clock {
    fn calibrate() -> Self {
        let (t0, i0) = (ticks(), Instant::now());
        sleep(Duration::from_millis(300));
        let (t1, i1) = (ticks(), Instant::now());
        Self {
            ns_per_tick: (i1 - i0).as_nanos() as f64 / (t1 - t0) as f64,
            epoch: t0,
        }
    }

    #[inline]
    fn ns(&self, ticks: u64) -> u64 {
        (ticks as f64 * self.ns_per_tick) as u64
    }

    fn ticks_for(&self, ns: u64) -> u64 {
        (ns as f64 / self.ns_per_tick) as u64
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn ticks() -> u64 {
    // SAFETY: lfence and rdtsc have no memory effects; the fence stops rdtsc running early
    unsafe {
        core::arch::x86_64::_mm_lfence();
        core::arch::x86_64::_rdtsc()
    }
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
fn ticks() -> u64 {
    static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_nanos() as u64
}

/// One thread's timings: a histogram of every sample and the slow ones with timestamps.
struct Recorder {
    role: &'static str,
    hist: Histogram<u64>,
    /// (start, ns, work, preempted): preempted is the involuntary context switches seen since
    /// the previous sample over `STALL_NS`, read only for those, else -1.
    slow: Vec<(u64, u64, i64, i64)>,
    /// Every sample over `STALL_NS`, kept even once `slow` is full.
    stalls: Vec<(u64, u64, i64, i64)>,
    last_stats: ThreadStats,
    end_stats: Option<ThreadStats>,
    slow_dropped: u64,
    slow_ticks: u64,
    max: (u64, u64, i64),
    stats_start: ThreadStats,
}

impl Recorder {
    fn new(role: &'static str, clock: &Clock) -> Self {
        let mut hist = Histogram::new_with_bounds(1, 60_000_000_000, 3).expect("histogram bounds");
        let mut slow = Vec::with_capacity(SLOW_KEEP);
        // touch every page now, so no first-touch fault lands inside a timed sample
        let mut v = 1.0f64;
        while v < 6e10 {
            hist.saturating_record(v as u64);
            v *= 1.0 + 1.0 / 4096.0;
        }
        hist.reset();
        slow.resize(SLOW_KEEP, (0, 0, 0, 0));
        slow.clear();
        Self {
            role,
            hist,
            slow,
            stalls: Vec::with_capacity(10_000),
            slow_dropped: 0,
            slow_ticks: clock.ticks_for(SLOW_NS),
            max: (0, 0, 0),
            stats_start: ThreadStats::now(),
            last_stats: ThreadStats::now(),
            end_stats: None,
        }
    }

    /// Records a sample from `start` to `end` ticks, `work` being what the thread did in it.
    #[inline]
    fn record(&mut self, clock: &Clock, start: u64, end: u64, work: i64) {
        let d = end.saturating_sub(start);
        let ns = clock.ns(d).max(1);
        self.hist.saturating_record(ns);
        if d > self.slow_ticks {
            let mut preempted = -1;
            if ns > STALL_NS {
                // costs ~10 µs, outside the sample, and only after a stall
                let now = ThreadStats::now();
                preempted = (now.nvcsw - self.last_stats.nvcsw) as i64;
                self.last_stats = now;
                self.stalls.push((start, ns, work, preempted));
            }
            if self.slow.len() < SLOW_KEEP {
                self.slow.push((start, ns, work, preempted));
            } else {
                self.slow_dropped += 1;
            }
        }
        if ns > self.max.1 {
            self.max = (start, ns, work);
        }
    }

    /// Takes the thread's end statistics; call on the thread that recorded.
    fn close(mut self) -> Self {
        self.end_stats = Some(ThreadStats::now());
        self
    }

    fn finish(self, clock: &Clock, cfg: &Config, arm: Arm, n: usize) -> Res<Row> {
        let mut file = String::from("at_ms\tns\twork\tpreempted\n");
        for (t, ns, w, p) in &self.slow {
            let _ = writeln!(file, "{:.3}\t{ns}\t{w}\t{p}", ms_since(clock, *t));
        }
        if self.slow_dropped > 0 {
            let _ = writeln!(file, "# {} more not kept", self.slow_dropped);
        }
        std::fs::write(format!("{}/slow-{}-{n}-{}.tsv", cfg.out, arm.name(), self.role), file)?;
        let mut file = String::from("at_ms\tns\twork\tpreempted\n");
        for (t, ns, w, p) in &self.stalls {
            let _ = writeln!(file, "{:.3}\t{ns}\t{w}\t{p}", ms_since(clock, *t));
        }
        std::fs::write(format!("{}/stalls-{}-{n}-{}.tsv", cfg.out, arm.name(), self.role), file)?;
        let q = |p: f64| self.hist.value_at_quantile(p);
        Ok(Row {
            arm,
            n,
            role: self.role,
            count: self.hist.len(),
            p50: q(0.5),
            p99: q(0.99),
            p999: q(0.999),
            max: self.hist.max(),
            over_1us: self.slow.len() as u64 + self.slow_dropped,
            stats: self.end_stats.ok_or("recorder not closed")?.since(self.stats_start),
            max_at_ms: ms_since(clock, self.max.0),
            max_work: self.max.2,
        })
    }
}

fn ms_since(clock: &Clock, t: u64) -> f64 {
    clock.ns(t.saturating_sub(clock.epoch)) as f64 / 1e6
}

struct Row {
    arm: Arm,
    n: usize,
    role: &'static str,
    count: u64,
    p50: u64,
    p99: u64,
    p999: u64,
    max: u64,
    over_1us: u64,
    stats: ThreadStats,
    max_at_ms: f64,
    max_work: i64,
}

impl Row {
    fn line(&self) -> String {
        format!(
            "{:5} n={:<3} {:15} count={:<10} p50={:>7} p99={:>7} p99.9={:>7} max={:>9} >1us={:<9} cs={}/{} flt={}/{} max@{:.1}ms work={}",
            self.arm.name(),
            self.n,
            self.role,
            self.count,
            self.p50,
            self.p99,
            self.p999,
            self.max,
            self.over_1us,
            self.stats.vcsw,
            self.stats.nvcsw,
            self.stats.minflt,
            self.stats.majflt,
            self.max_at_ms,
            self.max_work
        )
    }

    fn tsv(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.3}\t{}\n",
            self.arm.name(),
            self.n,
            self.role,
            self.count,
            self.p50,
            self.p99,
            self.p999,
            self.max,
            self.over_1us,
            self.stats.vcsw,
            self.stats.nvcsw,
            self.stats.minflt,
            self.stats.majflt,
            self.max_at_ms,
            self.max_work
        )
    }
}

/// The calling thread's (voluntary, involuntary) context switches and (minor, major) page
/// faults so far (Linux; zeros elsewhere).
#[derive(Clone, Copy, Default)]
struct ThreadStats {
    vcsw: u64,
    nvcsw: u64,
    minflt: u64,
    majflt: u64,
}

impl ThreadStats {
    fn now() -> Self {
        let status = std::fs::read_to_string("/proc/thread-self/status").unwrap_or_default();
        let get = |key: &str| {
            status
                .lines()
                .find_map(|l| l.strip_prefix(key))
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(0)
        };
        // fields after the parenthesised comm: state is field 3, minflt 10, majflt 12
        let stat = std::fs::read_to_string("/proc/thread-self/stat").unwrap_or_default();
        let after = stat.rsplit_once(')').map_or("", |(_, rest)| rest);
        let fields: Vec<u64> = after.split_whitespace().map(|f| f.parse().unwrap_or(0)).collect();
        Self {
            vcsw: get("voluntary_ctxt_switches:"),
            nvcsw: get("nonvoluntary_ctxt_switches:"),
            minflt: fields.get(7).copied().unwrap_or(0),
            majflt: fields.get(9).copied().unwrap_or(0),
        }
    }

    fn since(self, start: Self) -> Self {
        Self {
            vcsw: self.vcsw - start.vcsw,
            nvcsw: self.nvcsw - start.nvcsw,
            minflt: self.minflt - start.minflt,
            majflt: self.majflt - start.majflt,
        }
    }
}

#[cfg(target_os = "linux")]
fn pin(cpu: usize) {
    // SAFETY: cpu_set_t is plain data, zeroed is empty; pid 0 is the calling thread
    unsafe {
        let mut set: libc::cpu_set_t = std::mem::zeroed();
        libc::CPU_SET(cpu, &mut set);
        assert_eq!(
            0,
            libc::sched_setaffinity(0, size_of::<libc::cpu_set_t>(), &set),
            "pin to cpu {cpu}"
        );
    }
}

#[cfg(not(target_os = "linux"))]
fn pin(_cpu: usize) {}

/// The driver pointer, handed to the agent threads; the driver outlives them (see `Driver`).
#[derive(Clone, Copy)]
struct DriverPtr(*mut aeron_driver_t);
// SAFETY: each agent touches only its own part of the driver, as under dedicated threading
unsafe impl Send for DriverPtr {}

/// Where a driver's agents run: the conductor (yielding, on the housekeeping cpu) and the
/// sender and receiver on a cpu each or together; `name` suffixes the agents' roles.
#[derive(Clone, Copy)]
struct Placement {
    conductor: usize,
    net: Net,
    name: &'static str,
}

#[derive(Clone, Copy)]
enum Net {
    Dedicated(usize, usize),
    Shared(usize),
}

/// A media driver whose conductor, sender and receiver each run on a pinned thread of ours.
struct Driver {
    dir: String,
    running: Arc<AtomicBool>,
    /// Bumped to make the agents start new recorders, so set-up work is not counted.
    generation: Arc<AtomicU64>,
    agents: Vec<JoinHandle<Res<Recorder>>>,
    driver: Option<AeronDriver>,
    _context: AeronDriverContext,
}

impl Driver {
    fn launch(cfg: &Config, clock: &Arc<Clock>, tag: &str, readiness: bool) -> Res<Self> {
        let placement = Placement {
            conductor: cfg.cpus[0],
            net: Net::Dedicated(cfg.cpus[1], cfg.cpus[2]),
            name: "",
        };
        Self::launch_placed(clock, tag, readiness, placement)
    }

    fn launch_placed(clock: &Arc<Clock>, tag: &str, readiness: bool, placement: Placement) -> Res<Self> {
        let context = AeronDriverContext::new()?;
        let dir = format!("{}-many-streams-{tag}-{}", context.get_dir(), std::process::id());
        context.set_dir(&CString::new(dir.clone())?)?;
        context.set_dir_delete_on_start(true)?;
        context.set_dir_delete_on_shutdown(true)?;
        context.set_threading_mode(aeron_threading_mode_enum::AERON_THREADING_MODE_INVOKER)?;
        // 2 x 256 streams of 16 MiB terms would map 24 GiB
        context.set_term_buffer_length(64 * 1024)?;
        context.set_ipc_term_buffer_length(64 * 1024)?;
        context.set_conductor_cycle_threshold_ns(CYCLE_THRESHOLD_NS);
        context.set_sender_cycle_threshold_ns(CYCLE_THRESHOLD_NS);
        context.set_receiver_cycle_threshold_ns(CYCLE_THRESHOLD_NS);
        if readiness {
            context.set_readiness_interceptor(READINESS_WORDS)?;
        }
        let driver = AeronDriver::new(&context)?;
        driver.start(true)?;
        let ptr = DriverPtr(driver.get_inner());
        let running = Arc::new(AtomicBool::new(true));
        let generation = Arc::new(AtomicU64::new(0));
        let spawn = |role: &str, cpu: usize, yields: bool, work: fn(DriverPtr) -> i32| {
            // roles name rows and files, which outlive this driver
            let role: &'static str = Box::leak(format!("{role}{}", placement.name).into_boxed_str());
            let (running, generation, clock) = (Arc::clone(&running), Arc::clone(&generation), Arc::clone(clock));
            thread::Builder::new()
                .name(role.into())
                .spawn(move || -> Res<Recorder> {
                    pin(cpu);
                    // lets a syscall trace of the run be split by agent
                    #[cfg(target_os = "linux")]
                    println!("agent {role} tid {}", unsafe { libc::gettid() });
                    let mut seen = generation.load(Ordering::Acquire);
                    let mut rec = Recorder::new(role, &clock);
                    let mut cycles = 0u64;
                    while running.load(Ordering::Relaxed) {
                        cycles += 1;
                        let start = ticks();
                        let w = work(ptr);
                        rec.record(&clock, start, ticks(), w as i64);
                        let g = generation.load(Ordering::Relaxed);
                        if g != seen {
                            seen = g;
                            rec = Recorder::new(role, &clock);
                        }
                        if yields && w == 0 {
                            // the conductor shares the housekeeping cpu with the clients' conductors
                            thread::yield_now();
                        }
                    }
                    #[cfg(target_os = "linux")]
                    println!("agent {role} tid {} cycles {cycles}", unsafe { libc::gettid() });
                    Ok(rec.close())
                })
        };
        // SAFETY (all): the driver is started and stays open until these threads join
        let mut agents = vec![spawn("conductor", placement.conductor, true, |p| unsafe {
            aeron_driver_conductor_do_work((&raw mut (*p.0).conductor).cast::<c_void>())
                + aeron_driver_native_resource_agent_do_work((&raw mut (*p.0).native_resource_agent).cast::<c_void>())
        })?];
        match placement.net {
            Net::Dedicated(sender, receiver) => {
                agents.push(spawn("sender", sender, false, |p| unsafe {
                    aeron_driver_sender_do_work((&raw mut (*p.0).sender).cast::<c_void>())
                })?);
                agents.push(spawn("receiver", receiver, false, |p| unsafe {
                    aeron_driver_receiver_do_work((&raw mut (*p.0).receiver).cast::<c_void>())
                })?);
            }
            // as SHARED_NETWORK threading: one thread runs the sender, then the receiver
            Net::Shared(cpu) => agents.push(spawn("net", cpu, false, |p| unsafe {
                aeron_driver_sender_do_work((&raw mut (*p.0).sender).cast::<c_void>())
                    + aeron_driver_receiver_do_work((&raw mut (*p.0).receiver).cast::<c_void>())
            })?),
        }
        Ok(Self {
            dir,
            running,
            generation,
            agents,
            driver: Some(driver),
            _context: context,
        })
    }

    /// Starts the agents' timings, and Aeron's cycle counters, afresh.
    fn restart_timing(&self, counters: &AeronCountersReader) -> [i64; 3] {
        self.generation.fetch_add(1, Ordering::Release);
        // the agents swap in new recorders, which takes a few ms, before Aeron's maxima reset
        sleep(Duration::from_millis(50));
        cycle_counters(counters).map(|(id, _)| {
            // the agent proposes a new max with a plain store, so at worst one update is lost
            *counters.addr(id) = 0;
            counters.get_counter_value(id + 1).unwrap_or(0)
        })
    }

    /// Stops the agent threads and returns their timings. The driver stays open.
    fn stop(&mut self) -> Res<Vec<Recorder>> {
        self.running.store(false, Ordering::Relaxed);
        self.agents
            .drain(..)
            .map(|h| h.join().map_err(|_| "agent thread panicked")?)
            .collect()
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        let _ = self.stop();
        // closing runs every agent's on_close, now that no thread is in do_work
        self.driver.take();
    }
}

/// The system counter ids of the conductor's, sender's and receiver's cycle time max; the
/// over-threshold count is the next id.
fn cycle_counters(counters: &AeronCountersReader) -> [(i32, &'static str); 3] {
    let ids = [(26, "aeron-conductor"), (28, "aeron-sender"), (30, "aeron-receiver")];
    for (id, _) in ids {
        let label = counters.get_counter_label(id, 128).unwrap_or_default();
        assert!(
            label.contains("ycle time"),
            "counter {id} is {label:?}, not a cycle time max"
        );
    }
    ids
}

fn client(dir: &str) -> Res<Aeron> {
    let context = AeronContext::new()?;
    context.set_dir(&CString::new(dir)?)?;
    let aeron = Aeron::new(&context)?;
    aeron.start()?;
    Ok(aeron)
}

fn channel(port: u16) -> Res<CString> {
    Ok(CString::new(format!("aeron:udp?endpoint=127.0.0.1:{port}"))?)
}

/// A publication's channel: as `channel`, with `term-length` from `MANY_STREAMS_TERM_LENGTH`
/// when set (else the driver default this bench sets, 64 KiB).
fn publication_channel(port: u16) -> Res<CString> {
    let term = std::env::var("MANY_STREAMS_TERM_LENGTH").map_or(String::new(), |t| format!("|term-length={t}"));
    Ok(CString::new(format!("aeron:udp?endpoint=127.0.0.1:{port}{term}"))?)
}

/// Memory rows (`memory.tsv`), one per arm and N, taken once timing ends.
static MEMORY: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

/// Records the log files in `dirs` (apparent and allocated bytes) and this process's RSS,
/// which holds the driver(s) and every client.
fn record_memory(arm: Arm, n: usize, dirs: &[String]) {
    use std::os::unix::fs::MetadataExt;
    fn walk(dir: &std::path::Path, apparent: &mut u64, allocated: &mut u64, files: &mut u64) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let Ok(meta) = entry.metadata() else { continue };
            if meta.is_dir() {
                walk(&entry.path(), apparent, allocated, files);
            } else if entry.path().extension().is_some_and(|e| e == "logbuffer") {
                *apparent += meta.len();
                *allocated += meta.blocks() * 512;
                *files += 1;
            }
        }
    }
    let (mut apparent, mut allocated, mut files) = (0, 0, 0);
    for dir in dirs {
        walk(std::path::Path::new(dir), &mut apparent, &mut allocated, &mut files);
    }
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let kib = |key: &str| -> u64 {
        status
            .lines()
            .find_map(|l| l.strip_prefix(key))
            .and_then(|v| v.split_whitespace().next()?.parse().ok())
            .unwrap_or(0)
    };
    let row = format!(
        "{}\t{n}\t{files}\t{apparent}\t{allocated}\t{}\t{}\t{}\t{}\n",
        arm.name(),
        kib("VmRSS:") * 1024,
        kib("RssAnon:") * 1024,
        kib("RssFile:") * 1024,
        kib("RssShmem:") * 1024
    );
    print!("memory {row}");
    MEMORY.lock().expect("memory rows").push_str(&row);
}

/// Publications are concurrent rather than exclusive (`MANY_STREAMS_CONCURRENT=1`).
static CONCURRENT: AtomicBool = AtomicBool::new(false);

/// An exclusive or a concurrent publication.
enum Pub {
    Exclusive(AeronExclusivePublication),
    Concurrent(AeronPublication),
}

impl Pub {
    #[inline]
    fn offer(&self, buffer: &[u8]) -> Result<i64, AeronOfferError> {
        match self {
            Pub::Exclusive(p) => p.offer(buffer),
            Pub::Concurrent(p) => p.offer(buffer),
        }
    }

    fn is_connected(&self) -> bool {
        match self {
            Pub::Exclusive(p) => p.is_connected(),
            Pub::Concurrent(p) => p.is_connected(),
        }
    }
}

/// Subscriptions to `n` streams laid out by `layout` from `port` and `stream`.
fn add_subscriptions(aeron: &Aeron, layout: Layout, port: u16, stream: i32, n: usize) -> Res<Vec<AeronSubscription>> {
    (0..n)
        .map(|i| {
            Ok(aeron
                .async_add_subscription(
                    &channel(layout.port(port, i))?,
                    layout.stream(stream, i),
                    Handlers::NONE,
                    Handlers::NONE,
                )?
                .poll_blocking(Duration::from_secs(10))?)
        })
        .collect()
}

/// Publications to `n` streams laid out by `layout` from `port` and `stream`.
fn add_publications(aeron: &Aeron, layout: Layout, port: u16, stream: i32, n: usize) -> Res<Vec<Pub>> {
    (0..n)
        .map(|i| {
            let (ch, id) = (publication_channel(layout.port(port, i))?, layout.stream(stream, i));
            Ok(if CONCURRENT.load(Ordering::Relaxed) {
                Pub::Concurrent(
                    aeron
                        .async_add_publication(&ch, id)?
                        .poll_blocking(Duration::from_secs(10))?,
                )
            } else {
                Pub::Exclusive(
                    aeron
                        .async_add_exclusive_publication(&ch, id)?
                        .poll_blocking(Duration::from_secs(10))?,
                )
            })
        })
        .collect()
}

fn wait_connected(publications: &[Pub], subscriptions: &[AeronSubscription]) -> Res<()> {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !publications.iter().all(|p| p.is_connected()) || !subscriptions.iter().all(|s| s.is_connected()) {
        if Instant::now() > deadline {
            return Err("streams never connected".into());
        }
        sleep(Duration::from_millis(10));
    }
    Ok(())
}

/// Streams that carry nothing: UDP ones only the driver works on, IPC ones only the
/// client polls. Both ends of the IPC ones are on this client.
struct Quiet {
    udp_subs: Vec<AeronSubscription>,
    udp_pubs: Vec<Pub>,
    ipc_subs: Vec<AeronSubscription>,
    ipc_pubs: Vec<Pub>,
}

impl Quiet {
    /// `udp` subscriptions on ports from `sub_port + skip` and as many publications on ports
    /// from `pub_port + skip` (the other side adds the matching ends), and `ipc` IPC streams.
    fn add(
        aeron: &Aeron,
        sub_port: u16,
        pub_port: u16,
        skip: usize,
        udp: usize,
        ipc_stream: i32,
        ipc: usize,
    ) -> Res<Self> {
        let at = |port: u16| port + skip as u16;
        let stream = |base: i32| base + skip as i32;
        let (sub_stream, pub_stream) = if sub_port == PING_PORT {
            (PING_STREAM, PONG_STREAM)
        } else {
            (PONG_STREAM, PING_STREAM)
        };
        let ipc_subs = (0..ipc)
            .map(|i| {
                Ok(aeron
                    .async_add_subscription(AERON_IPC_STREAM, ipc_stream + i as i32, Handlers::NONE, Handlers::NONE)?
                    .poll_blocking(Duration::from_secs(10))?)
            })
            .collect::<Res<_>>()?;
        let ipc_pubs = (0..ipc)
            .map(|i| {
                Ok(Pub::Exclusive(
                    aeron
                        .async_add_exclusive_publication(AERON_IPC_STREAM, ipc_stream + i as i32)?
                        .poll_blocking(Duration::from_secs(10))?,
                ))
            })
            .collect::<Res<_>>()?;
        Ok(Self {
            udp_subs: add_subscriptions(aeron, Layout::PerPort, at(sub_port), stream(sub_stream), udp)?,
            udp_pubs: add_publications(aeron, Layout::PerPort, at(pub_port), stream(pub_stream), udp)?,
            ipc_subs,
            ipc_pubs,
        })
    }

    /// Polls the IPC subscriptions, as a client with that many quiet streams would.
    #[inline]
    fn poll(&self) -> Res<()> {
        for sub in &self.ipc_subs {
            if black_box(sub).poll_fn(
                |m, _| {
                    black_box(m);
                },
                FRAGMENT_LIMIT,
            )? != 0
            {
                return Err("a quiet stream delivered a message".into());
            }
        }
        Ok(())
    }
}

/// Which subscriptions a pass polls: all of them, or in the `ready` arm only those whose
/// readiness bit the driver set, each until it is empty.
enum Selector {
    All,
    Ready {
        readiness: AeronReadiness,
        /// Per word, the bits of this side's streams.
        mask: Vec<u64>,
        /// Per bit, the subscriptions whose stream maps to it.
        buckets: Vec<Vec<usize>>,
    },
}

impl Selector {
    /// For `n` subscriptions on streams from `stream`.
    fn new(aeron: &Aeron, arm: Arm, stream: i32, n: usize) -> Res<Self> {
        if arm != Arm::Ready {
            return Ok(Self::All);
        }
        let readiness = AeronReadiness::find(aeron).ok_or("driver keeps no readiness words")?;
        let mut mask = vec![0u64; readiness.bits() / 64];
        let mut buckets = vec![Vec::new(); readiness.bits()];
        for i in 0..n {
            let (word, bit) = readiness.bit_of(stream + i as i32);
            mask[word] |= 1 << bit;
            buckets[word * 64 + bit as usize].push(i);
        }
        Ok(Self::Ready {
            readiness,
            mask,
            buckets,
        })
    }

    /// Polls `subs`, calling `f` with the subscription's index and each message.
    #[inline]
    fn pass(&self, subs: &[AeronSubscription], mut f: impl FnMut(usize, &[u8])) -> Res<i32> {
        let mut fragments = 0;
        match self {
            Self::All => {
                for (i, sub) in subs.iter().enumerate() {
                    fragments += black_box(sub).poll_fn(|m, _| f(i, m), FRAGMENT_LIMIT)?;
                }
            }
            Self::Ready {
                readiness,
                mask,
                buckets,
            } => {
                for (word, &word_mask) in mask.iter().enumerate() {
                    let mut ready = readiness.take(word, word_mask);
                    while ready != 0 {
                        let bit = ready.trailing_zeros() as usize;
                        ready &= ready - 1;
                        for &i in &buckets[word * 64 + bit] {
                            loop {
                                let polled = black_box(&subs[i]).poll_fn(|m, _| f(i, m), FRAGMENT_LIMIT)?;
                                fragments += polled;
                                if polled < FRAGMENT_LIMIT as i32 {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(fragments)
    }
}

fn offer(publication: &Pub, buffer: &[u8]) -> Res<()> {
    loop {
        match publication.offer(buffer) {
            Ok(_) => return Ok(()),
            Err(e) if e.is_retryable() => std::hint::spin_loop(),
            Err(e) => return Err(format!("offer: {e:?}").into()),
        }
    }
}

fn message(stamp: u64, seq: u64, logical: u32) -> [u8; MESSAGE_LENGTH] {
    let mut m = [0u8; MESSAGE_LENGTH];
    m[0..8].copy_from_slice(&stamp.to_le_bytes());
    m[8..16].copy_from_slice(&seq.to_le_bytes());
    m[16..20].copy_from_slice(&logical.to_le_bytes());
    m
}

fn field(m: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(m[at..at + 8].try_into().expect("message too short"))
}

fn logical_of(m: &[u8]) -> usize {
    u32::from_le_bytes(m[16..20].try_into().expect("message too short")) as usize
}

/// Sends one message on every logical stream and checks each comes back on its own stream.
fn check_all_streams(publications: &[Pub], subscriptions: &[AeronSubscription], n: usize) -> Res<()> {
    let mut seen = vec![0u32; n];
    for i in 0..n {
        offer(&publications[i % publications.len()], &message(0, i as u64, i as u32))?;
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    while seen.contains(&0) {
        for (s, sub) in subscriptions.iter().enumerate() {
            sub.poll_fn(
                |m, _| {
                    let logical = logical_of(m);
                    // one subscription per logical stream unless multiplexed
                    if subscriptions.len() > 1 {
                        let expected = logical % subscriptions.len();
                        assert_eq!(s, expected, "echo for stream {logical} arrived on stream {s}");
                    }
                    seen[logical] += 1;
                },
                FRAGMENT_LIMIT,
            )?;
        }
        if Instant::now() > deadline {
            return Err(format!(
                "{} of {n} streams never echoed",
                seen.iter().filter(|&&c| c == 0).count()
            )
            .into());
        }
    }
    if let Some(i) = seen.iter().position(|&c| c != 1) {
        return Err(format!("stream {i} echoed {} times, expected once", seen[i]).into());
    }
    Ok(())
}

fn run_arm(cfg: &Config, clock: &Clock, arm: Arm, n: usize) -> Res<Vec<Row>> {
    let clock = Arc::new(Clock {
        ns_per_tick: clock.ns_per_tick,
        epoch: clock.epoch,
    });
    let tag = format!("{}-{n}", arm.name());
    if let Arm::Shard { drivers, shared } = arm {
        return sharded(cfg, &clock, arm, n, drivers, shared);
    }
    let mut driver = Driver::launch(cfg, &clock, &tag, arm == Arm::Ready)?;
    let rows = match arm {
        Arm::Idle(layout) => idle(cfg, &clock, &mut driver, n, layout, arm),
        _ => traffic(cfg, &clock, &mut driver, arm, n),
    };
    let rows = rows?;
    drop(driver);
    let _ = std::io::stdout().flush();
    Ok(rows)
}

/// Polls `n` connected but quiet subscriptions in a loop, timing each pass.
fn idle(cfg: &Config, clock: &Arc<Clock>, driver: &mut Driver, n: usize, layout: Layout, arm: Arm) -> Res<Vec<Row>> {
    let dir = driver.dir.clone();
    let (rec, exceeded) = thread::scope(|s| {
        s.spawn(|| -> Res<(Recorder, [i64; 3])> {
            // handles are not Send, so this thread makes its own
            let publisher = client(&dir)?;
            let subscriber = client(&dir)?;
            let subscriptions = add_subscriptions(&subscriber, layout, PING_PORT, PING_STREAM, n)?;
            let publications = add_publications(&publisher, layout, PING_PORT, PING_STREAM, n)?;
            wait_connected(&publications, &subscriptions)?;
            // every subscription has its image and is empty again before timing starts
            check_all_streams(&publications, &subscriptions, n)?;
            sleep(Duration::from_millis(200));
            // pinned only now, so the clients' conductor threads are not pinned with it
            pin(cfg.cpus[3]);
            let exceeded = driver.restart_timing(&subscriber.counters_reader());

            let mut rec = Recorder::new("client", clock);
            let end = Instant::now() + Duration::from_secs(cfg.secs);
            let mut passes = 0u64;
            while passes & 0x3ff != 0 || Instant::now() < end {
                let start = ticks();
                let mut fragments = 0;
                for sub in &subscriptions {
                    fragments += black_box(sub).poll_fn(
                        |m, _| {
                            black_box(m);
                        },
                        FRAGMENT_LIMIT,
                    )?;
                }
                rec.record(clock, start, ticks(), fragments as i64);
                if fragments != 0 {
                    return Err("idle streams delivered a message".into());
                }
                passes += 1;
            }
            record_memory(arm, n, std::slice::from_ref(&dir));
            Ok((rec.close(), exceeded))
        })
        .join()
        .map_err(|_| "client thread panicked")?
    })?;
    let mut rows = vec![rec.finish(clock, cfg, arm, n)?];
    for agent in driver.stop()? {
        rows.push(agent.finish(clock, cfg, arm, n)?);
    }
    rows.extend(aeron_cycle_rows(
        &client(&driver.dir)?.counters_reader(),
        exceeded,
        arm,
        n,
    ));
    Ok(rows)
}

/// Ping sends `rate` msg/s round robin over `n` logical streams, pong echoes each on the same
/// logical stream, and ping records the round trip.
fn traffic(cfg: &Config, clock: &Arc<Clock>, driver: &mut Driver, arm: Arm, n: usize) -> Res<Vec<Row>> {
    // active Aeron streams each way, logical streams, quiet UDP streams each way that only
    // the driver sees, quiet IPC subscriptions each side polls
    let (active, logical_n, extra_udp, extra_ipc, layout) = match arm {
        Arm::Mux(k) => (k.min(n), n, 0, 0, Layout::SamePort),
        Arm::Driver => (1, 1, n - 1, 0, Layout::PerPort),
        Arm::Client => (1, 1, 0, n - 1, Layout::PerPort),
        Arm::Plain(layout) => (n, n, 0, 0, layout),
        _ => (n, n, 0, 0, Layout::PerPort),
    };
    let rows_n = n;
    let n = logical_n;
    let dir = driver.dir.clone();
    let running = AtomicBool::new(true);
    let (pass, pong_rec, rtt, exceeded) = thread::scope(|s| -> Res<(Recorder, Recorder, Recorder, [i64; 3])> {
        let pong_thread = s.spawn(|| -> Res<Recorder> {
            let pong = client(&dir)?;
            let ping_subs = add_subscriptions(&pong, layout, PING_PORT, PING_STREAM, active)?;
            let pong_pubs = add_publications(&pong, layout, PONG_PORT, PONG_STREAM, active)?;
            let quiet = Quiet::add(
                &pong,
                PING_PORT,
                PONG_PORT,
                active,
                extra_udp,
                IPC_PONG_STREAM,
                extra_ipc,
            )?;
            let selector = Selector::new(&pong, arm, PING_STREAM, active)?;
            pin(cfg.cpus[4]);
            // the warm-up echoes are timed too: they come before ping has checked every stream
            let mut rec = Recorder::new("pong", clock);
            let mut failure = None;
            while running.load(Ordering::Relaxed) {
                let start = ticks();
                let fragments = selector.pass(&ping_subs, |i, m| {
                    if let Err(e) = offer(&pong_pubs[i], m) {
                        failure.get_or_insert(e);
                    }
                })?;
                quiet.poll()?;
                rec.record(clock, start, ticks(), fragments as i64);
                if let Some(e) = failure.take() {
                    return Err(e);
                }
            }
            Ok(rec.close())
        });

        let ping_thread = s.spawn(|| -> Res<(Recorder, Recorder, [i64; 3])> {
            let ping = client(&dir)?;
            let pong_subs = add_subscriptions(&ping, layout, PONG_PORT, PONG_STREAM, active)?;
            let ping_pubs = add_publications(&ping, layout, PING_PORT, PING_STREAM, active)?;
            let quiet = Quiet::add(
                &ping,
                PONG_PORT,
                PING_PORT,
                active,
                extra_udp,
                IPC_PING_STREAM,
                extra_ipc,
            )?;
            wait_connected(&ping_pubs, &pong_subs)?;
            wait_connected(&quiet.udp_pubs, &quiet.udp_subs)?;
            wait_connected(&quiet.ipc_pubs, &quiet.ipc_subs)?;
            check_all_streams(&ping_pubs, &pong_subs, n)?;
            let selector = Selector::new(&ping, arm, PONG_STREAM, active)?;
            sleep(Duration::from_millis(200));
            pin(cfg.cpus[3]);
            let exceeded = driver.restart_timing(&ping.counters_reader());

            let mut pass = Recorder::new("ping", clock);
            let mut rtt = Recorder::new("rtt", clock);
            let interval = clock.ticks_for(1_000_000_000 / cfg.rate);
            let total = cfg.rate * cfg.secs;
            let (mut sent, mut received) = (0u64, 0u64);
            let mut next_send = ticks();
            let mut failure: Option<Box<dyn Error + Send + Sync>> = None;
            let mut all_sent_at: Option<Instant> = None;
            loop {
                if sent < total && ticks() >= next_send {
                    let logical = (sent % n as u64) as u32;
                    let p = &ping_pubs[logical as usize % ping_pubs.len()];
                    offer(p, &message(ticks(), sent, logical))?;
                    sent += 1;
                    next_send += interval;
                    if sent == total {
                        all_sent_at = Some(Instant::now());
                    }
                }
                let start = ticks();
                let fragments = selector.pass(&pong_subs, |s, m| {
                    let at = ticks();
                    let logical = logical_of(m);
                    if logical % pong_subs.len() != s {
                        failure.get_or_insert(format!("echo of stream {logical} on stream {s}").into());
                    }
                    if field(m, 8) % n as u64 != logical as u64 {
                        failure.get_or_insert("echo carries the wrong logical stream".into());
                    }
                    rtt.record(clock, field(m, 0), at, logical as i64);
                    received += 1;
                })?;
                quiet.poll()?;
                pass.record(clock, start, ticks(), fragments as i64);
                if let Some(e) = failure.take() {
                    return Err(e);
                }
                if received == total || all_sent_at.is_some_and(|t| t.elapsed() > Duration::from_millis(500)) {
                    break;
                }
            }
            if received != sent {
                return Err(format!("sent {sent}, {received} echoes came back").into());
            }
            println!("{} n={rows_n}: sent {sent}, received {received}", arm.name());
            record_memory(arm, rows_n, std::slice::from_ref(&dir));
            Ok((pass.close(), rtt.close(), exceeded))
        });
        let ping_result = ping_thread.join().map_err(|_| "ping thread panicked")?;
        running.store(false, Ordering::Relaxed);
        let pong_rec = pong_thread.join().map_err(|_| "pong thread panicked")??;
        let (pass, rtt, exceeded) = ping_result?;
        Ok((pass, pong_rec, rtt, exceeded))
    })?;
    let mut rows = vec![
        rtt.finish(clock, cfg, arm, rows_n)?,
        pass.finish(clock, cfg, arm, rows_n)?,
        pong_rec.finish(clock, cfg, arm, rows_n)?,
    ];
    for agent in driver.stop()? {
        rows.push(agent.finish(clock, cfg, arm, rows_n)?);
    }
    rows.extend(aeron_cycle_rows(
        &client(&driver.dir)?.counters_reader(),
        exceeded,
        arm,
        rows_n,
    ));
    Ok(rows)
}

/// `n` per-port streams each way split evenly over `k` drivers. One thread (on the pong cpu)
/// both sends pings and echoes them, so the client side is the same for every `k`.
///
/// CPUs, from `MANY_STREAMS_SHARD_CPUS` (default `1,2,3,5`): conductors yield on the
/// housekeeping cpu; with `shared` driver d's sender and receiver share cpu d; otherwise
/// driver d uses cpus 2d and 2d+1 of the list, so at most two dedicated drivers fit.
fn sharded(cfg: &Config, clock: &Arc<Clock>, arm: Arm, n: usize, k: usize, shared: bool) -> Res<Vec<Row>> {
    let cpus: Vec<usize> = std::env::var("MANY_STREAMS_SHARD_CPUS")
        .unwrap_or_else(|_| "1,2,3,5".into())
        .split(',')
        .map(|c| c.trim().parse())
        .collect::<Result<_, _>>()?;
    if !n.is_multiple_of(k) || (shared && k > cpus.len()) || (!shared && 2 * k > cpus.len()) {
        return Err(format!("{n} streams over {k} drivers do not fit cpus {cpus:?}").into());
    }
    let names = ["-a", "-b", "-c", "-d", "-e", "-f", "-g", "-h"];
    let mut drivers = (0..k)
        .map(|d| {
            let net = if shared {
                Net::Shared(cpus[d])
            } else {
                Net::Dedicated(cpus[2 * d], cpus[2 * d + 1])
            };
            let placement = Placement {
                conductor: cfg.cpus[0],
                net,
                name: names[d],
            };
            Driver::launch_placed(clock, &format!("{}-{n}{}", arm.name(), names[d]), false, placement)
        })
        .collect::<Res<Vec<_>>>()?;
    let dirs: Vec<String> = drivers.iter().map(|d| d.dir.clone()).collect();
    let per = n / k;
    let (pass, rtt) = thread::scope(|s| {
        s.spawn(|| -> Res<(Recorder, Recorder)> {
            let clients = dirs.iter().map(|d| client(d)).collect::<Res<Vec<_>>>()?;
            let (mut ping_pubs, mut ping_subs, mut pong_pubs, mut pong_subs) = (vec![], vec![], vec![], vec![]);
            for (d, c) in clients.iter().enumerate() {
                let (port, stream) = ((d * per) as u16, (d * per) as i32);
                ping_subs.extend(add_subscriptions(
                    c,
                    Layout::PerPort,
                    PING_PORT + port,
                    PING_STREAM + stream,
                    per,
                )?);
                pong_subs.extend(add_subscriptions(
                    c,
                    Layout::PerPort,
                    PONG_PORT + port,
                    PONG_STREAM + stream,
                    per,
                )?);
                ping_pubs.extend(add_publications(
                    c,
                    Layout::PerPort,
                    PING_PORT + port,
                    PING_STREAM + stream,
                    per,
                )?);
                pong_pubs.extend(add_publications(
                    c,
                    Layout::PerPort,
                    PONG_PORT + port,
                    PONG_STREAM + stream,
                    per,
                )?);
            }
            wait_connected(&ping_pubs, &ping_subs)?;
            wait_connected(&pong_pubs, &pong_subs)?;

            // echoes pings and hands each echo, with its stream, to `on_echo`
            let pass = |on_echo: &mut dyn FnMut(usize, &[u8])| -> Res<i32> {
                let mut failure = None;
                let mut fragments = 0;
                for (i, sub) in ping_subs.iter().enumerate() {
                    fragments += black_box(sub).poll_fn(
                        |m, _| {
                            if let Err(e) = offer(&pong_pubs[i], m) {
                                failure.get_or_insert(e);
                            }
                        },
                        FRAGMENT_LIMIT,
                    )?;
                }
                for (i, sub) in pong_subs.iter().enumerate() {
                    fragments += black_box(sub).poll_fn(|m, _| on_echo(i, m), FRAGMENT_LIMIT)?;
                }
                failure.map_or(Ok(fragments), Err)
            };

            // every stream echoes once, on its own stream, before timing starts
            for (i, p) in ping_pubs.iter().enumerate() {
                offer(p, &message(0, i as u64, i as u32))?;
            }
            let mut seen = vec![0u32; n];
            let deadline = Instant::now() + Duration::from_secs(10);
            while seen.contains(&0) {
                pass(&mut |i, m| {
                    assert_eq!(i, logical_of(m), "echo on the wrong stream");
                    seen[i] += 1;
                })?;
                if Instant::now() > deadline {
                    return Err("not every stream echoed".into());
                }
            }
            sleep(Duration::from_millis(200));
            pin(cfg.cpus[4]);
            for (driver, c) in drivers.iter().zip(&clients) {
                driver.restart_timing(&c.counters_reader());
            }

            let mut pass_rec = Recorder::new("client", clock);
            let mut rtt = Recorder::new("rtt", clock);
            let interval = clock.ticks_for(1_000_000_000 / cfg.rate);
            let total = cfg.rate * cfg.secs;
            let (mut sent, mut received) = (0u64, 0u64);
            let mut next_send = ticks();
            let mut all_sent_at: Option<Instant> = None;
            let mut wrong = None;
            loop {
                if sent < total && ticks() >= next_send {
                    let logical = (sent % n as u64) as u32;
                    offer(&ping_pubs[logical as usize], &message(ticks(), sent, logical))?;
                    sent += 1;
                    next_send += interval;
                    if sent == total {
                        all_sent_at = Some(Instant::now());
                    }
                }
                let start = ticks();
                let fragments = pass(&mut |i, m| {
                    let at = ticks();
                    if logical_of(m) != i {
                        wrong = Some(i);
                    }
                    rtt.record(clock, field(m, 0), at, i as i64);
                    received += 1;
                })?;
                pass_rec.record(clock, start, ticks(), fragments as i64);
                if let Some(i) = wrong {
                    return Err(format!("echo on the wrong stream {i}").into());
                }
                if received == total || all_sent_at.is_some_and(|t| t.elapsed() > Duration::from_millis(500)) {
                    break;
                }
            }
            if received != sent {
                return Err(format!("sent {sent}, {received} echoes came back").into());
            }
            println!("{} n={n}: sent {sent}, received {received}", arm.name());
            record_memory(arm, n, &dirs);
            Ok((pass_rec.close(), rtt.close()))
        })
        .join()
        .map_err(|_| "client thread panicked")?
    })?;
    let mut rows = vec![rtt.finish(clock, cfg, arm, n)?, pass.finish(clock, cfg, arm, n)?];
    for driver in &mut drivers {
        for agent in driver.stop()? {
            rows.push(agent.finish(clock, cfg, arm, n)?);
        }
    }
    Ok(rows)
}

/// What the readiness interceptor adds per data datagram: `mark_ready` when the stream's bit
/// is already set (a busy stream) and when it is clear (an atomic OR as well). Rows carry the
/// mean in `p50`; the clear case includes one plain store that clears the bit again.
fn mark_cost(cfg: &Config, clock: &Clock) -> Vec<Row> {
    const CALLS: u64 = 20_000_000;
    let words: Vec<AtomicI64> = (0..READINESS_WORDS).map(|_| AtomicI64::new(0)).collect();
    let refs: Vec<&AtomicI64> = words.iter().collect();
    let mut frame = [0u8; 64];
    frame[0..4].copy_from_slice(&64i32.to_le_bytes());
    frame[6..8].copy_from_slice(&1i16.to_le_bytes());
    frame[16..20].copy_from_slice(&5i32.to_le_bytes());
    mark_ready(&refs, &frame);
    assert_eq!(
        1 << 5,
        words[0].load(Ordering::Relaxed),
        "mark_ready did not set the bit"
    );
    thread::scope(|s| {
        s.spawn(|| {
            pin(cfg.cpus[3]);
            let time = |clear: bool| {
                let start = ticks();
                for _ in 0..CALLS {
                    if clear {
                        words[0].store(0, Ordering::Relaxed);
                    }
                    mark_ready(black_box(&refs), black_box(&frame));
                }
                clock.ns(ticks() - start) as f64 / CALLS as f64
            };
            [("mark-set", time(false)), ("mark-clear", time(true))]
                .into_iter()
                .map(|(role, mean)| Row {
                    arm: Arm::Mark,
                    n: 0,
                    role,
                    count: CALLS,
                    p50: mean.round() as u64,
                    p99: 0,
                    p999: 0,
                    max: 0,
                    over_1us: 0,
                    stats: ThreadStats::default(),
                    max_at_ms: 0.0,
                    max_work: 0,
                })
                .collect()
        })
        .join()
        .expect("mark thread panicked")
    })
}

/// Aeron's own cycle-time-max and over-threshold counters since `exceeded_at_start` was
/// read, as rows with only `max` and `over_1us` (here: over `CYCLE_THRESHOLD_NS`) set.
fn aeron_cycle_rows(counters: &AeronCountersReader, exceeded_at_start: [i64; 3], arm: Arm, n: usize) -> Vec<Row> {
    cycle_counters(counters)
        .into_iter()
        .zip(exceeded_at_start)
        .map(|((id, role), start)| Row {
            arm,
            n,
            role,
            count: 0,
            p50: 0,
            p99: 0,
            p999: 0,
            max: counters.get_counter_value(id).unwrap_or(-1) as u64,
            over_1us: (counters.get_counter_value(id + 1).unwrap_or(0) - start) as u64,
            stats: ThreadStats::default(),
            max_at_ms: 0.0,
            max_work: 0,
        })
        .collect()
}
