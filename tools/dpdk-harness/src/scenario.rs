//! Scenario implementations for the interoperability harness (plan §11.2).
//!
//! Each scenario exercises a §11.2 virtual-device case through the full stack:
//! embedded media driver → transport (DPDK vdev or default kernel UDP) →
//! rusteron-client pub/sub. Cross-driver unicast uses manual MDC
//! (`control-mode=manual` + `add_destination`), the pattern a real
//! publisher/subscriber on separate nodes uses.
//!
//! Scenarios are symmetric (both roles send and receive) or asymmetric
//! (primary transmits, secondary receives); the `role` branch decides.

use std::error::Error;
use std::thread::sleep;
use std::time::{Duration, Instant};

use log::{info, warn};
use rusteron_client::{
    Aeron, AeronExclusivePublication, AeronSubscription, Handlers, IntoCString,
};

/// A single `host:port` endpoint (IPv4 — this plan targets Nitro, no brackets).
#[derive(Clone, Debug)]
pub struct Endpoint {
    pub host: String,
    pub port: u16,
}

impl Endpoint {
    /// Parse `host:port`.
    pub fn parse(s: &str) -> Self {
        let (host, port) = s.rsplit_once(':').expect("endpoint must be host:port");
        Endpoint {
            host: host.to_string(),
            port: port.parse().expect("endpoint port must be numeric"),
        }
    }

    /// `aeron:udp?endpoint=host:port` — a subscription or MDC destination URI.
    pub fn udp_uri(&self) -> String {
        format!("aeron:udp?endpoint={}:{}", self.host, self.port)
    }
}

/// Per-process configuration, read from the environment by `main`.
#[derive(Clone, Debug)]
pub struct ScenarioCfg {
    pub role: String,
    pub name: String,
    /// Local MDC control endpoint (publication URI `control-mode=manual`).
    pub pub_ctrl: Endpoint,
    /// Second control endpoint — the reconnect scenario re-opens here.
    pub pub_ctrl2: Endpoint,
    /// Local subscription endpoint(s); `[0]` is the primary.
    pub sub_endpoints: Vec<Endpoint>,
    /// Peer subscription endpoint(s) the publication sends to.
    pub destinations: Vec<Endpoint>,
    pub msgs: u64,
    pub payload: usize,
    pub stream: i32,
    pub timeout: Duration,
    /// Byte this process fills its payload body with (direction is visible).
    pub fill_byte: u8,
    /// Byte this process expects the peer's payload body to contain.
    pub expect_byte: u8,
    /// Media driver MTU; the driver fragments above this and the DPDK transport
    /// bounds every emitted datagram by it (§8).
    pub mtu: usize,
}

impl ScenarioCfg {
    pub fn sub(&self) -> &Endpoint {
        &self.sub_endpoints[0]
    }
    pub fn dest(&self) -> &Endpoint {
        &self.destinations[0]
    }
}

/// Outcome of a scenario run, mapped into the JSON report by `main`.
#[derive(Default, Debug)]
pub struct ScenarioResult {
    pub ok: bool,
    pub sent: u64,
    pub received: u64,
    pub bytes: u64,
    pub bad_payload: u64,
    pub duration_ms: u64,
    pub detail: Vec<String>,
    pub error: Option<String>,
}

impl ScenarioResult {
    fn fail(msg: impl Into<String>) -> Self {
        ScenarioResult {
            ok: false,
            error: Some(msg.into()),
            ..Default::default()
        }
    }
    fn push(&mut self, msg: impl Into<String>) {
        self.detail.push(msg.into());
    }
}

/// Run the named scenario. `name` comes from `--scenario`.
pub fn run(cfg: &ScenarioCfg, aeron: &Aeron) -> ScenarioResult {
    let started = Instant::now();
    let mut res = match cfg.name.as_str() {
        "bidirectional_unicast" | "restart" => symmetric_unicast(cfg, aeron),
        "reconnect" => reconnect(cfg, aeron),
        "multi_endpoint" | "loss_recovery" => multi_endpoint(cfg, aeron),
        other => return ScenarioResult::fail(format!("unknown scenario {other:?}")),
    };
    res.duration_ms = started.elapsed().as_millis() as u64;
    res
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn open_publication(
    aeron: &Aeron,
    ctrl: &Endpoint,
    stream: i32,
    timeout: Duration,
) -> Result<AeronExclusivePublication, Box<dyn Error>> {
    let uri = format!("aeron:udp?control-mode=manual|control={}:{}", ctrl.host, ctrl.port);
    let pub_ = aeron.add_exclusive_publication(&uri.clone().into_c_string(), stream, timeout)?;
    info!("[pub] channel={uri}");
    Ok(pub_)
}

fn open_subscription(
    aeron: &Aeron,
    ep: &Endpoint,
    stream: i32,
    timeout: Duration,
) -> Result<AeronSubscription, Box<dyn Error>> {
    let uri = ep.udp_uri();
    let sub = aeron.add_subscription(&uri.clone().into_c_string(), stream, Handlers::NONE, Handlers::NONE, timeout)?;
    info!("[sub] channel={uri}");
    Ok(sub)
}

/// Add an MDC destination and wait until the publication is connected to it.
fn add_and_wait(
    pub_: &AeronExclusivePublication,
    dest: &Endpoint,
    timeout: Duration,
) -> Result<(), Box<dyn Error>> {
    let uri = dest.udp_uri();
    pub_.add_destination(&uri.clone().into_c_string(), timeout)?;
    let started = Instant::now();
    while !pub_.is_connected() && started.elapsed() < timeout {
        sleep(Duration::from_millis(10));
    }
    if !pub_.is_connected() {
        return Err(format!("publication never connected to destination {uri}").into());
    }
    info!("[pub] connected to {uri}");
    Ok(())
}

/// Offer one message, retrying transient errors until it is accepted.
fn offer_retry(
    pub_: &AeronExclusivePublication,
    msg: &[u8],
    timeout: Duration,
) -> Result<(), Box<dyn Error>> {
    let started = Instant::now();
    loop {
        match pub_.offer(msg) {
            Ok(pos) if pos >= 0 => return Ok(()),
            Ok(_) => { /* negative position — unreachable via the typed wrapper */ }
            Err(e) if e.is_retryable() && started.elapsed() < timeout => {}
            Err(e) => return Err(format!("offer failed: {e}").into()),
        }
        if started.elapsed() >= timeout {
            return Err("offer retry timed out".into());
        }
        sleep(Duration::from_millis(1));
    }
}

#[derive(Default)]
struct Recv {
    received: u64,
    bytes: u64,
    bad_payload: u64,
}

/// Drain `sub` once, counting messages and validating the payload pattern.
fn poll_once(sub: &AeronSubscription, expect_byte: u8, recv: &mut Recv) -> Result<(), Box<dyn Error>> {
    sub.poll_fn(
        |data, _header| {
            if data.len() < 9 {
                return;
            }
            recv.received += 1;
            recv.bytes += data.len() as u64;
            if data[8..].iter().any(|&b| b != expect_byte) {
                recv.bad_payload += 1;
            }
        },
        1024,
    )?;
    Ok(())
}

fn build_msg(cfg: &ScenarioCfg, seq: u64) -> Vec<u8> {
    let mut m = vec![0u8; 8 + cfg.payload];
    m[8..].fill(cfg.fill_byte);
    m[0..8].copy_from_slice(&seq.to_le_bytes());
    m
}

fn send_batch(
    pub_: &AeronExclusivePublication,
    cfg: &ScenarioCfg,
    range: std::ops::Range<u64>,
) -> Result<(), Box<dyn Error>> {
    for seq in range {
        let msg = build_msg(cfg, seq);
        offer_retry(pub_, &msg, cfg.timeout)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Scenarios
// ---------------------------------------------------------------------------

/// §11.2 DPDK-to-DPDK / DPDK-to-default bidirectional unicast. Both roles
/// publish and subscribe; every message must round-trip byte-identically.
fn symmetric_unicast(cfg: &ScenarioCfg, aeron: &Aeron) -> ScenarioResult {
    let mut res = ScenarioResult::default();
    let result = (|| -> Result<(), Box<dyn Error>> {
        let pub_ = open_publication(aeron, &cfg.pub_ctrl, cfg.stream, cfg.timeout)?;
        let sub = open_subscription(aeron, cfg.sub(), cfg.stream, cfg.timeout)?;
        add_and_wait(&pub_, cfg.dest(), cfg.timeout)?;

        let mut sent: u64 = 0;
        let mut recv = Recv::default();
        let started = Instant::now();
        while (sent < cfg.msgs || recv.received < cfg.msgs) && started.elapsed() < cfg.timeout {
            if sent < cfg.msgs {
                let msg = build_msg(cfg, sent);
                offer_retry(&pub_, &msg, cfg.timeout)?;
                sent += 1;
            }
            poll_once(&sub, cfg.expect_byte, &mut recv)?;
            sleep(Duration::from_millis(1));
        }
        res.sent = sent;
        res.received = recv.received;
        res.bytes = recv.bytes;
        res.bad_payload = recv.bad_payload;
        if sent != cfg.msgs {
            return Err(format!("sent {sent} of {} messages", cfg.msgs).into());
        }
        if recv.received != cfg.msgs {
            return Err(format!("received {} of {} messages", recv.received, cfg.msgs).into());
        }
        if recv.bad_payload != 0 {
            return Err(format!("{} payload byte mismatches", recv.bad_payload).into());
        }
        res.push(format!("bidirectional round-trip ok ({sent} messages, {} bytes)", res.bytes));
        Ok(())
    })();

    match result {
        Ok(()) => res.ok = true,
        Err(e) => {
            warn!("scenario failed: {e}");
            res.error = Some(e.to_string());
        }
    }
    res
}

/// §11.2 transport reconnect: the primary closes its publication to the first
/// destination and opens a fresh one to a second destination; the secondary
/// receives both batches across its two subscriptions. The first batch must
/// not be corrupted by the reconnect, and the second must land on the new
/// transport — no caller-pointer retention in the copied remote address.
fn reconnect(cfg: &ScenarioCfg, aeron: &Aeron) -> ScenarioResult {
    let mut res = ScenarioResult::default();
    let split = cfg.msgs / 3;
    let result = if cfg.role == "primary" {
        (|| -> Result<(), Box<dyn Error>> {
            let pub1 = open_publication(aeron, &cfg.pub_ctrl, cfg.stream, cfg.timeout)?;
            add_and_wait(&pub1, cfg.dest(), cfg.timeout)?;
            send_batch(&pub1, cfg, 0..split)?;
            drop(pub1); // close the first transport; the next is a fresh create

            let pub2 = open_publication(aeron, &cfg.pub_ctrl2, cfg.stream, cfg.timeout)?;
            add_and_wait(&pub2, &cfg.destinations[1], cfg.timeout)?;
            send_batch(&pub2, cfg, split..cfg.msgs)?;
            res.sent = cfg.msgs;
            res.push(format!("reconnected: batch1 {split} msgs, batch2 {} msgs", cfg.msgs - split));
            Ok(())
        })()
    } else {
        (|| -> Result<(), Box<dyn Error>> {
            let sub0 = open_subscription(aeron, &cfg.sub_endpoints[0], cfg.stream, cfg.timeout)?;
            let sub1 = open_subscription(aeron, &cfg.sub_endpoints[1], cfg.stream, cfg.timeout)?;
            let mut recv = Recv::default();
            let started = Instant::now();
            while recv.received < cfg.msgs && started.elapsed() < cfg.timeout {
                poll_once(&sub0, cfg.expect_byte, &mut recv)?;
                poll_once(&sub1, cfg.expect_byte, &mut recv)?;
                sleep(Duration::from_millis(1));
            }
            res.received = recv.received;
            res.bytes = recv.bytes;
            res.bad_payload = recv.bad_payload;
            if recv.received != cfg.msgs {
                return Err(format!("received {} of {} messages across both transports", recv.received, cfg.msgs).into());
            }
            if recv.bad_payload != 0 {
                return Err(format!("{} payload byte mismatches", recv.bad_payload).into());
            }
            res.push(format!("reconnected: received {} messages on both transports", recv.received));
            Ok(())
        })()
    };

    match result {
        Ok(()) => res.ok = true,
        Err(e) => {
            warn!("scenario failed: {e}");
            res.error = Some(e.to_string());
        }
    }
    res
}

/// §11.2 multiple registered endpoints (and loss recovery — same code path):
/// the primary adds every destination; the secondary registers one
/// subscription per endpoint and must receive `msgs × endpoints`. Under
/// `tc netem` loss on the bridge the driver's NAK/retransmit path recovers
/// every gap, so the secondary still sees all messages.
fn multi_endpoint(cfg: &ScenarioCfg, aeron: &Aeron) -> ScenarioResult {
    let mut res = ScenarioResult::default();
    let result = if cfg.role == "primary" {
        (|| -> Result<(), Box<dyn Error>> {
            let pub_ = open_publication(aeron, &cfg.pub_ctrl, cfg.stream, cfg.timeout)?;
            for dest in &cfg.destinations {
                add_and_wait(&pub_, dest, cfg.timeout)?;
            }
            send_batch(&pub_, cfg, 0..cfg.msgs)?;
            res.sent = cfg.msgs;
            res.push(format!(
                "published {msgs} messages to {n} destinations",
                msgs = cfg.msgs,
                n = cfg.destinations.len()
            ));
            Ok(())
        })()
    } else {
        (|| -> Result<(), Box<dyn Error>> {
            let mut subs = Vec::new();
            for ep in &cfg.sub_endpoints {
                subs.push(open_subscription(aeron, ep, cfg.stream, cfg.timeout)?);
            }
            let expect = cfg.msgs * cfg.sub_endpoints.len() as u64;
            let mut recv = Recv::default();
            let started = Instant::now();
            while recv.received < expect && started.elapsed() < cfg.timeout {
                for sub in &subs {
                    poll_once(sub, cfg.expect_byte, &mut recv)?;
                }
                sleep(Duration::from_millis(1));
            }
            res.received = recv.received;
            res.bytes = recv.bytes;
            res.bad_payload = recv.bad_payload;
            if recv.received != expect {
                return Err(format!("received {} of {expect} messages across {} endpoints", recv.received, subs.len()).into());
            }
            if recv.bad_payload != 0 {
                return Err(format!("{} payload byte mismatches", recv.bad_payload).into());
            }
            res.push(format!("received {} messages across {} endpoints", recv.received, subs.len()));
            Ok(())
        })()
    };

    match result {
        Ok(()) => res.ok = true,
        Err(e) => {
            warn!("scenario failed: {e}");
            res.error = Some(e.to_string());
        }
    }
    res
}
