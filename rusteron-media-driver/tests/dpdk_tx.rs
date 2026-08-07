//! Aeron transmit path over DPDK (Ticket 4, plan §7.4/§7.5).
//!
//! Drives the real Aeron binding callbacks (init/send/close/bind_addr_and_port)
//! against the DPDK-free fakes and asserts on the captured frames: golden
//! IPv4/UDP vectors, checksum-offload metadata, DF, routing (direct vs gateway),
//! the ARP state machine (rate limiting, learning, expiry, anti-poisoning), MTU
//! rejection, mbuf exhaustion, partial bursts, and mbuf ownership.
//!
//! The native transport links the same `rusteron_dpdk` core archive plus the
//! fakes declared in `common/mod.rs`. Linux x86_64 only.
#![cfg(all(feature = "dpdk", target_os = "linux", target_arch = "x86_64"))]

mod common;
use common::{close, create, last_error, rusteron_dpdk_config_t, TestEnv};

use serial_test::serial;

use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

// Real Aeron types/functions from the generated bindings so we exercise the
// genuine callback contract.
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// ---------------------------------------------------------------------------
// Native test hooks (fakes + runtime).
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy)]
struct FakeCapture {
    data: [u8; 2048],
    len: u32,
    ol_flags: u32,
    l2_len: u16,
    l3_len: u16,
    l4_len: u16,
    udp_pseudo_csum: u16,
    port_id: u16,
}

extern "C" {
    fn rusteron_dpdk_transport_bindings() -> *mut aeron_udp_channel_transport_bindings_stct;
    fn rusteron_dpdk_transport_test_arp_seed(transport: *mut c_void, ip: *const c_char, mac: *const u8) -> c_int;
    fn rusteron_dpdk_transport_test_arp_rx(transport: *mut c_void, role: c_int, frame: *const u8, len: usize) -> c_int;
    fn rusteron_dpdk_test_set_clock_ms(ms: u64);
    fn rusteron_dpdk_fake_set_tx_burst_cap(n: u16);
    fn rusteron_dpdk_fake_set_pool_avail(n: c_int);
    fn rusteron_dpdk_fake_capture_count() -> c_int;
    fn rusteron_dpdk_fake_capture_at(index: c_int, out: *mut FakeCapture) -> c_int;
    fn rusteron_dpdk_fake_allocated() -> c_int;
    fn rusteron_dpdk_fake_released() -> c_int;
}

// ---------------------------------------------------------------------------
// IPv4 sockaddr mirror (bindings.rs has no sockaddr_in on Linux either).
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy)]
struct SockAddrIn {
    sin_family: u16,
    sin_port: u16, // network order
    sin_addr: u32, // network-order bytes in memory
    sin_zero: [u8; 8],
}

impl SockAddrIn {
    fn new(ip: [u8; 4], port: u16) -> Self {
        Self {
            sin_family: 2, // AF_INET
            sin_port: port.to_be(),
            // s_addr is a plain u32 read natively by the C side (no ntohl), so
            // the octets must sit in memory in order: ip[0] at the low byte on
            // little-endian x86_64. from_be_bytes would store them reversed and
            // every ARP lookup / multicast check in transport.c would miss.
            sin_addr: u32::from_le_bytes(ip),
            sin_zero: [0; 8],
        }
    }
    fn storage_mut(&mut self) -> *mut sockaddr_storage {
        self as *mut Self as *mut sockaddr_storage
    }
}

// ---------------------------------------------------------------------------
// Harness: native runtime + a genuine Aeron transport struct + binding table.
// ---------------------------------------------------------------------------

struct Harness {
    _env: TestEnv,
    native: *mut c_void,
    bindings: *mut aeron_udp_channel_transport_bindings_stct,
    transport: Box<aeron_udp_channel_transport_stct>,
    data_paths: Box<aeron_udp_channel_data_paths_stct>,
    initialized: bool,
}

impl Harness {
    fn new() -> Self {
        Harness::new_with_burst(rusteron_dpdk_config_t::valid().burst_size)
    }

    fn new_with_burst(burst_size: u16) -> Self {
        let env = TestEnv::new();
        env.eal_skip();
        let mut config = rusteron_dpdk_config_t::valid();
        config.burst_size = burst_size;
        let native = create(&config).unwrap_or_else(|e| panic!("create failed: {e}"));
        let bindings = unsafe { rusteron_dpdk_transport_bindings() };
        assert!(!bindings.is_null());
        let transport = Box::new(unsafe { std::mem::zeroed::<aeron_udp_channel_transport_stct>() });
        let data_paths = Box::new(unsafe { std::mem::zeroed::<aeron_udp_channel_data_paths_stct>() });
        Harness {
            _env: env,
            native,
            bindings,
            transport,
            data_paths,
            initialized: false,
        }
    }

    /// Run the binding init_func with an IPv4 bind address, optional connect
    /// address, and the given channel MTU.
    fn init(
        &mut self,
        affinity: aeron_udp_channel_transport_affinity_t,
        bind_ip: [u8; 4],
        bind_port: u16,
        connect: Option<([u8; 4], u16)>,
        mtu: usize,
    ) {
        assert!(!self.initialized, "harness already initialized");
        let t = &mut *self.transport;
        unsafe { ptr::write_bytes(t as *mut _, 0, 1) };

        let mut bind = SockAddrIn::new(bind_ip, bind_port);
        let mut dummy = SockAddrIn::new([0, 0, 0, 0], 0);
        let mut conn = connect.map(|(ip, p)| SockAddrIn::new(ip, p));
        let mut params: aeron_udp_channel_transport_params_stct = unsafe { std::mem::zeroed() };
        params.mtu_length = mtu;

        let rc = unsafe {
            let b = &*self.bindings;
            b.init_func.unwrap()(
                t,
                bind.storage_mut(),
                dummy.storage_mut(),
                conn.as_mut().map_or(ptr::null_mut(), SockAddrIn::storage_mut),
                &mut params,
                ptr::null_mut(),
                affinity,
            )
        };
        assert_eq!(rc, 0, "init failed: {}", last_error());
        self.initialized = true;
    }

    /// Send to an explicit destination address (unconnected path).
    fn send_to(&mut self, dst: ([u8; 4], u16), payloads: &[&[u8]]) -> (c_int, i64) {
        let mut d = SockAddrIn::new(dst.0, dst.1);
        self.send_common(d.storage_mut(), payloads)
    }

    /// Send with a NULL address (connected path: uses connected_address).
    fn send_connected(&mut self, payloads: &[&[u8]]) -> (c_int, i64) {
        self.send_common(ptr::null_mut(), payloads)
    }

    fn send_common(&mut self, addr: *mut sockaddr_storage, payloads: &[&[u8]]) -> (c_int, i64) {
        let iovs: Vec<iovec> = payloads
            .iter()
            .map(|p| iovec {
                iov_base: p.as_ptr() as *mut c_void,
                iov_len: p.len(),
            })
            .collect();
        let mut bytes_sent: i64 = 0;
        let rc = unsafe {
            let b = &*self.bindings;
            b.send_func.unwrap()(
                &mut *self.data_paths,
                &mut *self.transport,
                addr,
                iovs.as_ptr() as *mut iovec,
                iovs.len(),
                &mut bytes_sent,
            )
        };
        (rc, bytes_sent)
    }

    fn bind_addr_and_port(&mut self) -> String {
        let mut buf = [0 as c_char; 128];
        let rc = unsafe {
            let b = &*self.bindings;
            b.bind_addr_and_port_func.unwrap()(&mut *self.transport, buf.as_mut_ptr(), buf.len())
        };
        assert_eq!(rc, 0, "bind_addr_and_port failed: {}", last_error());
        let s = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        s.to_string_lossy().into_owned()
    }

    fn arp_seed(&self, ip: [u8; 4], mac: [u8; 6]) {
        let ip_str = CString::new(format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])).unwrap();
        let rc = unsafe { rusteron_dpdk_transport_test_arp_seed(self.native, ip_str.as_ptr(), mac.as_ptr()) };
        assert_eq!(rc, 0, "arp_seed failed: {}", last_error());
    }

    fn arp_rx(&self, role: c_int, frame: &[u8]) -> c_int {
        unsafe { rusteron_dpdk_transport_test_arp_rx(self.native, role, frame.as_ptr(), frame.len()) }
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                let b = &*self.bindings;
                b.close_func.unwrap()(&mut *self.transport);
            }
        }
        close(self.native);
    }
}

// ---------------------------------------------------------------------------
// Frame helpers.
// ---------------------------------------------------------------------------

const SENDER: i32 = 0;
const SENDER_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x01];
const RECEIVER_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x02];
const SENDER_IP: [u8; 4] = [10, 0, 0, 1];
const SENDER_GATEWAY: [u8; 4] = [10, 0, 0, 254];
const RECEIVER_IP: [u8; 4] = [10, 0, 1, 1];

fn captures() -> Vec<FakeCapture> {
    let n = unsafe { rusteron_dpdk_fake_capture_count() };
    (0..n)
        .map(|i| {
            let mut c = unsafe { std::mem::zeroed::<FakeCapture>() };
            let rc = unsafe { rusteron_dpdk_fake_capture_at(i, &mut c) };
            assert_eq!(rc, 0);
            c
        })
        .collect()
}

fn cap_bytes(c: &FakeCapture) -> &[u8] {
    &c.data[..c.len as usize]
}

fn assert_no_leak() {
    let allocated = unsafe { rusteron_dpdk_fake_allocated() };
    let released = unsafe { rusteron_dpdk_fake_released() };
    assert_eq!(
        allocated, released,
        "mbuf leak: allocated={allocated} released={released}"
    );
}

fn eth_dst(c: &FakeCapture) -> [u8; 6] {
    c.data[0..6].try_into().unwrap()
}
fn eth_src(c: &FakeCapture) -> [u8; 6] {
    c.data[6..12].try_into().unwrap()
}
fn eth_type(c: &FakeCapture) -> u16 {
    u16::from_be_bytes([c.data[12], c.data[13]])
}
fn u16_at(c: &FakeCapture, off: usize) -> u16 {
    u16::from_be_bytes([c.data[off], c.data[off + 1]])
}
fn u32_ip_at(c: &FakeCapture, off: usize) -> [u8; 4] {
    c.data[off..off + 4].try_into().unwrap()
}

const IP: usize = 14;
const IP_SRC: usize = IP + 12;
const IP_DST: usize = IP + 16;
const UDP: usize = IP + 20;
const UDP_DPORT: usize = UDP + 2;
const PAYLOAD: usize = UDP + 8;

fn ip_ver_ihl(c: &FakeCapture) -> u8 {
    c.data[IP]
}
fn ip_flags_frag(c: &FakeCapture) -> u16 {
    u16_at(c, IP + 6)
}
fn ip_ttl(c: &FakeCapture) -> u8 {
    c.data[IP + 8]
}
fn ip_proto(c: &FakeCapture) -> u8 {
    c.data[IP + 9]
}
fn ip_csum(c: &FakeCapture) -> u16 {
    u16_at(c, IP + 10)
}
fn udp_payload(c: &FakeCapture) -> &[u8] {
    &c.data[PAYLOAD..c.len as usize]
}

/// The same pseudo-header seed DPDK's TX offload expects (packet.c).
fn pseudo_csum(src: [u8; 4], dst: [u8; 4], udp_len: u16) -> u16 {
    let mut sum: u32 = 0;
    for (s, d) in src.chunks(2).zip(dst.chunks(2)) {
        sum += u16::from_be_bytes([s[0], s[1]]) as u32;
        sum += u16::from_be_bytes([d[0], d[1]]) as u32;
    }
    sum += 17; // IPPROTO_UDP
    sum += udp_len as u32;
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    sum as u16
}

/// Build a full UDP golden frame (mirrors packet.c byte-for-byte).
fn build_udp_frame(
    dst_mac: [u8; 6],
    src_mac: [u8; 6],
    src_ip: [u8; 4],
    dst_ip: [u8; 4],
    sport: u16,
    dport: u16,
    payload: &[u8],
) -> Vec<u8> {
    let mut f = Vec::new();
    f.extend_from_slice(&dst_mac);
    f.extend_from_slice(&src_mac);
    f.extend_from_slice(&[0x08, 0x00]);
    f.push(0x45);
    f.push(0x00);
    let total = (20 + 8 + payload.len()) as u16;
    f.extend_from_slice(&total.to_be_bytes());
    f.extend_from_slice(&[0x00, 0x00]); // id
    f.extend_from_slice(&[0x40, 0x00]); // DF
    f.push(64); // ttl
    f.push(17); // proto
    f.extend_from_slice(&[0x00, 0x00]); // csum (hw offload)
    f.extend_from_slice(&src_ip);
    f.extend_from_slice(&dst_ip);
    f.extend_from_slice(&sport.to_be_bytes());
    f.extend_from_slice(&dport.to_be_bytes());
    let ulen = (8 + payload.len()) as u16;
    f.extend_from_slice(&ulen.to_be_bytes());
    f.extend_from_slice(&pseudo_csum(src_ip, dst_ip, ulen).to_be_bytes());
    f.extend_from_slice(payload);
    f
}

/// Build an ARP request frame (42 bytes).
fn arp_request(sha: [u8; 6], spa: [u8; 4], tpa: [u8; 4]) -> Vec<u8> {
    let mut f = Vec::new();
    f.extend_from_slice(&[0xff; 6]);
    f.extend_from_slice(&sha);
    f.extend_from_slice(&[0x08, 0x06]);
    f.extend_from_slice(&0x0001u16.to_be_bytes()); // htype
    f.extend_from_slice(&0x0800u16.to_be_bytes()); // ptype
    f.push(6);
    f.push(4);
    f.extend_from_slice(&1u16.to_be_bytes()); // oper request
    f.extend_from_slice(&sha);
    f.extend_from_slice(&spa);
    f.extend_from_slice(&[0; 6]);
    f.extend_from_slice(&tpa);
    f
}

/// Build an ARP reply frame (42 bytes).
fn arp_reply(dst_mac: [u8; 6], sha: [u8; 6], spa: [u8; 4], tpa: [u8; 4]) -> Vec<u8> {
    let mut f = Vec::new();
    f.extend_from_slice(&dst_mac);
    f.extend_from_slice(&sha);
    f.extend_from_slice(&[0x08, 0x06]);
    f.extend_from_slice(&0x0001u16.to_be_bytes());
    f.extend_from_slice(&0x0800u16.to_be_bytes());
    f.push(6);
    f.push(4);
    f.extend_from_slice(&2u16.to_be_bytes()); // oper reply
    f.extend_from_slice(&sha);
    f.extend_from_slice(&spa);
    f.extend_from_slice(&dst_mac);
    f.extend_from_slice(&tpa);
    f
}

fn arp_fields(c: &FakeCapture) -> (u16, u16, [u8; 6], [u8; 4], [u8; 6], [u8; 4]) {
    let a = 14;
    (
        u16_at(c, a),
        u16_at(c, a + 6),
        c.data[a + 8..a + 14].try_into().unwrap(),
        u32_ip_at(c, a + 14),
        c.data[a + 18..a + 24].try_into().unwrap(),
        u32_ip_at(c, a + 24),
    )
}

// ---------------------------------------------------------------------------
// Golden vectors / frame correctness
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn udp_frame_golden_vector() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        1472,
    );
    const TARGET_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x50];
    h.arp_seed([10, 0, 0, 5], TARGET_MAC);

    let payload = b"hello";
    let (rc, bytes_sent) = h.send_to(([10, 0, 0, 5], 40123), &[payload]);
    assert_eq!(rc, 1);
    assert_eq!(bytes_sent, 5);

    let caps = captures();
    assert_eq!(caps.len(), 1);
    let c = &caps[0];
    let expected = build_udp_frame(TARGET_MAC, SENDER_MAC, SENDER_IP, [10, 0, 0, 5], 40000, 40123, payload);
    assert_eq!(cap_bytes(c), expected.as_slice());

    // Checksum-offload metadata (plan §7.4).
    assert_eq!(c.ol_flags, 0b111, "IPV4|IP_CKSUM|UDP_CKSUM");
    assert_eq!((c.l2_len, c.l3_len, c.l4_len), (14, 20, 8));
    assert_eq!(c.udp_pseudo_csum, pseudo_csum(SENDER_IP, [10, 0, 0, 5], 13));
    assert_eq!(c.port_id, 0, "sends on the sender ENA");
    assert_no_leak();
}

#[test]
#[serial]
fn frame_fields_are_ordinary_valid_ipv4_udp() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        1472,
    );
    h.arp_seed([10, 0, 0, 9], [0x02, 0, 0, 0, 0, 0x51]);

    let (rc, _) = h.send_to(([10, 0, 0, 9], 40123), &[b"abcd", b"efgh"]);
    assert_eq!(rc, 2);
    let caps = captures();
    assert_eq!(caps.len(), 2, "each iovec is one datagram");
    for c in &caps {
        assert_eq!(eth_type(c), 0x0800);
        assert_eq!(ip_ver_ihl(c), 0x45);
        assert_eq!(ip_proto(c), 17);
        assert_eq!(ip_ttl(c), 64);
        assert_eq!(ip_flags_frag(c), 0x4000, "DF must be set, never fragment");
        assert_eq!(ip_csum(c), 0, "IPv4 checksum delegated to hw offload");
        assert_eq!(c.l2_len, 14);
        assert_eq!(c.l3_len, 20);
        assert_eq!(c.l4_len, 8);
    }
    assert_eq!(udp_payload(&caps[0]), b"abcd");
    assert_eq!(udp_payload(&caps[1]), b"efgh");
    assert_eq!(caps[0].len, 42 + 4);
    assert_eq!(caps[1].len, 42 + 4);
    assert_no_leak();
}

// ---------------------------------------------------------------------------
// Routing
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn routing_direct_for_in_subnet_gateway_otherwise() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        1472,
    );
    const PEER: [u8; 6] = [0x02, 0, 0, 0, 0, 0x60];
    const GW: [u8; 6] = [0x02, 0, 0, 0, 0, 0xfe];
    h.arp_seed([10, 0, 0, 5], PEER);
    h.arp_seed(SENDER_GATEWAY, GW);

    // In-subnet: ARP the destination itself.
    let (rc, _) = h.send_to(([10, 0, 0, 5], 1000), &[b"direct"]);
    assert_eq!(rc, 1);
    let c0 = &captures()[0];
    assert_eq!(eth_dst(c0), PEER);
    assert_eq!(u32_ip_at(c0, IP_DST), [10, 0, 0, 5]);

    // Outside the /24: ARP the gateway, keep the destination IP in the header.
    let (rc, _) = h.send_to(([10, 0, 2, 5], 1000), &[b"viagw"]);
    assert_eq!(rc, 1);
    let c1 = &captures()[1];
    assert_eq!(eth_dst(c1), GW);
    assert_eq!(u32_ip_at(c1, IP_DST), [10, 0, 2, 5]);
    assert_eq!(u32_ip_at(c1, IP_SRC), SENDER_IP);
    assert_no_leak();
}

#[test]
#[serial]
fn connected_transport_uses_connected_address() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        Some(([10, 0, 0, 77], 5000)),
        1472,
    );
    h.arp_seed([10, 0, 0, 77], [0x02, 0, 0, 0, 0, 0x77]);

    let (rc, bytes_sent) = h.send_connected(&[b"connected"]);
    assert_eq!(rc, 1);
    assert_eq!(bytes_sent, 9);
    let c = &captures()[0];
    assert_eq!(u32_ip_at(c, IP_DST), [10, 0, 0, 77]);
    assert_eq!(u16_at(c, UDP_DPORT), 5000);
    assert_no_leak();
}

#[test]
#[serial]
fn multicast_destination_rejected() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        1472,
    );
    let (rc, bytes_sent) = h.send_to(([224, 0, 0, 5], 1234), &[b"mc"]);
    assert_eq!(rc, -1);
    assert_eq!(bytes_sent, 0);
    assert!(last_error().contains("multicast"), "got: {}", last_error());
    assert_eq!(captures().len(), 0);
}

#[test]
#[serial]
fn receiver_affinity_uses_receiver_port() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
        RECEIVER_IP,
        60000,
        None,
        1472,
    );
    h.arp_seed([10, 0, 1, 9], [0x02, 0, 0, 0, 0, 0x59]);

    let (rc, _) = h.send_to(([10, 0, 1, 9], 7000), &[b"rx"]);
    assert_eq!(rc, 1);
    let c = &captures()[0];
    assert_eq!(c.port_id, 1);
    assert_eq!(eth_src(c), RECEIVER_MAC);
    assert_eq!(u32_ip_at(c, IP_SRC), RECEIVER_IP);
    assert_no_leak();
}

// ---------------------------------------------------------------------------
// MTU / oversized
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn oversized_datagram_rejected() {
    let mut h = Harness::new();
    // Channel MTU of 32 bytes: 32 fits, 33 is rejected.
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        32,
    );
    h.arp_seed([10, 0, 0, 5], [0x02, 0, 0, 0, 0, 0x50]);

    let ok_payload = [7u8; 32];
    let (rc, bytes_sent) = h.send_to(([10, 0, 0, 5], 40123), &[&ok_payload]);
    assert_eq!(rc, 1);
    assert_eq!(bytes_sent, 32);

    let (rc, bytes_sent) = h.send_to(([10, 0, 0, 5], 40123), &[&[1u8; 33]]);
    assert_eq!(rc, -1);
    assert_eq!(bytes_sent, 0);
    assert!(last_error().contains("oversized"), "got: {}", last_error());
    assert_no_leak();
}

#[test]
#[serial]
fn oversized_after_valid_prefix_flushes_prefix_then_errors() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        32,
    );
    h.arp_seed([10, 0, 0, 5], [0x02, 0, 0, 0, 0, 0x50]);

    let small = [7u8; 8];
    let (rc, _) = h.send_to(([10, 0, 0, 5], 40123), &[&small, &[1u8; 33]]);
    assert_eq!(rc, -1, "permanent validation error after sending the prefix");
    let caps = captures();
    assert_eq!(caps.len(), 1, "the valid prefix was transmitted");
    assert_eq!(udp_payload(&caps[0]), small.as_slice());
    assert_no_leak();
}

// ---------------------------------------------------------------------------
// Batching / mbuf ownership
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn each_iovec_is_one_datagram() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        1472,
    );
    h.arp_seed([10, 0, 0, 5], [0x02, 0, 0, 0, 0, 0x50]);

    let (rc, bytes_sent) = h.send_to(([10, 0, 0, 5], 40123), &[b"a", b"bb", b"ccc"]);
    assert_eq!(rc, 3, "returns the accepted datagram count");
    assert_eq!(bytes_sent, 6, "bytes_sent accrues the accepted payload bytes");
    let caps = captures();
    assert_eq!(caps.len(), 3);
    assert_eq!(udp_payload(&caps[0]), b"a");
    assert_eq!(udp_payload(&caps[1]), b"bb");
    assert_eq!(udp_payload(&caps[2]), b"ccc");
    assert_eq!(caps[1].len, 42 + 2);
    assert_no_leak();
}

#[test]
#[serial]
fn burst_flushes_at_burst_size_and_accumulates() {
    let mut h = Harness::new_with_burst(4);
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        1472,
    );
    h.arp_seed([10, 0, 0, 5], [0x02, 0, 0, 0, 0, 0x50]);

    let payloads: Vec<Vec<u8>> = (0..8).map(|i| vec![i; 4]).collect();
    let refs: Vec<&[u8]> = payloads.iter().map(|p| p.as_slice()).collect();
    let (rc, bytes_sent) = h.send_to(([10, 0, 0, 5], 40123), &refs);
    assert_eq!(rc, 8);
    assert_eq!(bytes_sent, 8 * 4);
    assert_eq!(captures().len(), 8, "two bursts of 4");
    assert_no_leak();
}

#[test]
#[serial]
fn partial_burst_reports_accepted_prefix() {
    let mut h = Harness::new_with_burst(4);
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        1472,
    );
    h.arp_seed([10, 0, 0, 5], [0x02, 0, 0, 0, 0, 0x50]);

    // The NIC accepts only the first 2 frames of a 4-frame burst.
    unsafe { rusteron_dpdk_fake_set_tx_burst_cap(2) };

    let payloads: Vec<Vec<u8>> = (0..8).map(|i| vec![i; 4]).collect();
    let refs: Vec<&[u8]> = payloads.iter().map(|p| p.as_slice()).collect();
    let (rc, bytes_sent) = h.send_to(([10, 0, 0, 5], 40123), &refs);
    assert_eq!(rc, 2, "stops after the partial burst, reporting the accepted prefix");
    assert_eq!(bytes_sent, 2 * 4);
    let caps = captures();
    assert_eq!(caps.len(), 2);
    assert_eq!(udp_payload(&caps[0]), vec![0; 4].as_slice());
    assert_eq!(udp_payload(&caps[1]), vec![1; 4].as_slice());
    assert_no_leak();
}

#[test]
#[serial]
fn mbuf_exhaustion_flushes_prefix() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        1472,
    );
    h.arp_seed([10, 0, 0, 5], [0x02, 0, 0, 0, 0, 0x50]);

    // Only 2 mbufs may be live; the third allocation fails.
    unsafe { rusteron_dpdk_fake_set_pool_avail(2) };

    let payloads: Vec<Vec<u8>> = (0..16).map(|i| vec![i; 4]).collect();
    let refs: Vec<&[u8]> = payloads.iter().map(|p| p.as_slice()).collect();
    let (rc, bytes_sent) = h.send_to(([10, 0, 0, 5], 40123), &refs);
    assert_eq!(rc, 2, "mbuf exhaustion flushes and reports the built prefix");
    assert_eq!(bytes_sent, 2 * 4);
    assert_eq!(captures().len(), 2);
    assert_no_leak();
}

#[test]
#[serial]
fn zero_length_send_is_retryable_zero() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        1472,
    );
    let iovs: Vec<iovec> = vec![];
    let mut bytes_sent: i64 = 0;
    let mut d = SockAddrIn::new([10, 0, 0, 5], 40123);
    let rc = unsafe {
        let b = &*h.bindings;
        b.send_func.unwrap()(
            &mut *h.data_paths,
            &mut *h.transport,
            d.storage_mut(),
            iovs.as_ptr() as *mut iovec,
            0,
            &mut bytes_sent,
        )
    };
    assert_eq!(rc, 0);
    assert_eq!(bytes_sent, 0);
}

// ---------------------------------------------------------------------------
// ARP state machine (plan §7.5)
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn arp_request_on_unresolved_rate_limited_100ms() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        1472,
    );
    // A non-zero base so the first send (last_request_ms starts at 0) clears
    // the 100 ms retry gate; the pinned clock must sit past the window.
    unsafe { rusteron_dpdk_test_set_clock_ms(1000) };

    // First send: unresolved -> ARP request, retryable zero result.
    let (rc, bytes_sent) = h.send_to(([10, 0, 0, 5], 40123), &[b"x"]);
    assert_eq!(rc, 0, "unresolved ARP returns zero so Aeron retries");
    assert_eq!(bytes_sent, 0);
    assert_eq!(captures().len(), 1);

    // Within the 100 ms retry window: still unresolved, no new request.
    unsafe { rusteron_dpdk_test_set_clock_ms(1050) };
    let (rc, _) = h.send_to(([10, 0, 0, 5], 40123), &[b"x"]);
    assert_eq!(rc, 0);
    assert_eq!(captures().len(), 1, "rate-limited: no request within 100 ms");

    // Past the window: request sent again.
    unsafe { rusteron_dpdk_test_set_clock_ms(1101) };
    let (rc, _) = h.send_to(([10, 0, 0, 5], 40123), &[b"x"]);
    assert_eq!(rc, 0);
    assert_eq!(captures().len(), 2);

    // The request is a broadcast ARP request for the next hop.
    let req = &captures()[0];
    assert_eq!(eth_type(req), 0x0806);
    assert_eq!(eth_dst(req), [0xff; 6]);
    let (htype, oper, sha, spa, tha, tpa) = arp_fields(req);
    assert_eq!(htype, 1);
    assert_eq!(oper, 1);
    assert_eq!(sha, SENDER_MAC);
    assert_eq!(spa, SENDER_IP);
    assert_eq!(tha, [0; 6]);
    assert_eq!(tpa, [10, 0, 0, 5]);
    assert_no_leak();
}

#[test]
#[serial]
fn arp_learns_reply_and_sends() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        1472,
    );
    // Non-zero base so the unresolved send clears the 100 ms ARP retry gate.
    unsafe { rusteron_dpdk_test_set_clock_ms(1000) };
    const PEER_MAC: [u8; 6] = [0x02, 0, 0, 0, 0, 0x50];

    let (rc, _) = h.send_to(([10, 0, 0, 5], 40123), &[b"x"]);
    assert_eq!(rc, 0);
    assert_eq!(captures().len(), 1, "ARP request");

    // A reply addressed to our MAC for the outstanding request is learned.
    let reply = arp_reply(SENDER_MAC, PEER_MAC, [10, 0, 0, 5], SENDER_IP);
    let consumed = h.arp_rx(SENDER, &reply);
    assert_eq!(consumed, 1, "reply for an outstanding request is consumed");

    let (rc, bytes_sent) = h.send_to(([10, 0, 0, 5], 40123), &[b"nowresolved"]);
    assert_eq!(rc, 1);
    assert_eq!(bytes_sent, 11);
    let caps = captures();
    assert_eq!(caps.len(), 2, "request + the data frame");
    assert_eq!(eth_dst(&caps[1]), PEER_MAC, "data frame goes to the learned MAC");
    assert_no_leak();
}

#[test]
#[serial]
fn arp_reachable_expires_after_30s() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        1472,
    );
    unsafe { rusteron_dpdk_test_set_clock_ms(1000) };
    const PEER_MAC: [u8; 6] = [0x02, 0, 0, 0, 0, 0x50];
    h.arp_seed([10, 0, 0, 5], PEER_MAC);

    let (rc, _) = h.send_to(([10, 0, 0, 5], 40123), &[b"fresh"]);
    assert_eq!(rc, 1);
    assert_eq!(captures().len(), 1);

    // 30 s later the entry is stale and re-resolution begins.
    unsafe { rusteron_dpdk_test_set_clock_ms(1000 + 30_000) };
    let (rc, _) = h.send_to(([10, 0, 0, 5], 40123), &[b"stale"]);
    assert_eq!(rc, 0, "expired entry re-resolves");
    assert_eq!(captures().len(), 2, "a fresh ARP request was sent");
    let req = &captures()[1];
    assert_eq!(eth_type(req), 0x0806);
    assert_no_leak();
}

#[test]
#[serial]
fn arp_reachable_entry_not_poisoned_by_unrelated_reply() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        1472,
    );
    unsafe { rusteron_dpdk_test_set_clock_ms(0) };
    const GOOD: [u8; 6] = [0x02, 0, 0, 0, 0, 0x50];
    const EVIL: [u8; 6] = [0x02, 0, 0, 0, 0, 0x99];
    h.arp_seed([10, 0, 0, 5], GOOD);

    // An attacker claims the peer's IP with a different MAC while the entry is
    // reachable: it must be ignored (plan §7.5 anti-poisoning).
    let reply = arp_reply(SENDER_MAC, EVIL, [10, 0, 0, 5], SENDER_IP);
    let consumed = h.arp_rx(SENDER, &reply);
    assert_eq!(consumed, 0, "reply not addressed to an outstanding request");

    let (rc, _) = h.send_to(([10, 0, 0, 5], 40123), &[b"safe"]);
    assert_eq!(rc, 1);
    assert_eq!(eth_dst(&captures()[0]), GOOD, "reachable entry is unchanged");
    assert_no_leak();
}

#[test]
#[serial]
fn arp_responds_to_requests_for_local_ip() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        1472,
    );

    // A request for our local IPv4 gets a unicast reply.
    let req = arp_request([0x02, 0, 0, 0, 0, 0x42], [10, 0, 0, 42], SENDER_IP);
    let consumed = h.arp_rx(SENDER, &req);
    assert_eq!(consumed, 1);
    let caps = captures();
    assert_eq!(caps.len(), 1);
    let rep = &caps[0];
    assert_eq!(eth_type(rep), 0x0806);
    assert_eq!(eth_dst(rep), [0x02, 0, 0, 0, 0, 0x42]);
    let (_, oper, sha, spa, tha, tpa) = arp_fields(rep);
    assert_eq!(oper, 2, "reply");
    assert_eq!(sha, SENDER_MAC);
    assert_eq!(spa, SENDER_IP);
    assert_eq!(tha, [0x02, 0, 0, 0, 0, 0x42]);
    assert_eq!(tpa, [10, 0, 0, 42]);
    assert_no_leak();

    // A request for a foreign IP is ignored.
    let foreign = arp_request([0x02, 0, 0, 0, 0, 0x43], [10, 0, 0, 43], [10, 0, 0, 99]);
    let consumed = h.arp_rx(SENDER, &foreign);
    assert_eq!(consumed, 0, "requests for other IPs are not answered");
    assert_eq!(captures().len(), 1, "no extra frame");
}

// ---------------------------------------------------------------------------
// bind_addr_and_port
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn bind_addr_and_port_formats_local_identity() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
        SENDER_IP,
        40000,
        None,
        1472,
    );
    assert_eq!(h.bind_addr_and_port(), "10.0.0.1:40000");
}
