//! Aeron receive polling and endpoint dispatch over DPDK (Ticket 5, plan
//! §7.6/§7.7).
//!
//! Drives the real Aeron poller/recvmmsg bindings against the DPDK-free fakes:
//! every accepted and rejected frame class, endpoint-map dispatch (including a
//! shared ENA polled once), ARP routing from the receive loop, callback-before-
//! recycle ordering, the vlen/burst caps, foreign-destination and unknown-port
//! counting, and zero allocations across repeated empty polls.
//!
//! Linux x86_64 only; links the same archives as dpdk_tx.rs.
#![cfg(all(feature = "dpdk", target_os = "linux", target_arch = "x86_64"))]

mod common;
use common::{close, create, last_error, rusteron_dpdk_config_t, TestEnv};

use serial_test::serial;

use std::os::raw::{c_int, c_void};
use std::ptr;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// ---------------------------------------------------------------------------
// Native test hooks (fakes + runtime).
// ---------------------------------------------------------------------------

/// Mirror of `rusteron_dpdk_rx_stats_t` (14 u64 buckets, plan §7.6).
#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
struct RxStats {
    accepted: u64,
    arp: u64,
    ipv6: u64,
    multicast: u64,
    ethertype: u64,
    vlan: u64,
    ip_options: u64,
    fragment: u64,
    truncated: u64,
    protocol: u64,
    checksum: u64,
    multi_segment: u64,
    foreign_dst: u64,
    unknown_port: u64,
}

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
    fn rusteron_dpdk_transport_test_rx_stats(transport: *const c_void, out: *mut RxStats);
    fn rusteron_dpdk_fake_rx_inject(
        port_id: u16,
        frame: *const u8,
        len: usize,
        rx_ol_flags: u32,
        nb_segs: u32,
    ) -> c_int;
    fn rusteron_dpdk_fake_rx_queued(port_id: u16) -> c_int;
    fn rusteron_dpdk_fake_capture_count() -> c_int;
    fn rusteron_dpdk_fake_capture_at(index: c_int, out: *mut FakeCapture) -> c_int;
    fn rusteron_dpdk_fake_allocated() -> c_int;
    fn rusteron_dpdk_fake_released() -> c_int;
    fn rusteron_dpdk_test_set_clock_ms(ms: u64);
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
            sin_addr: u32::from_le_bytes(ip),
            sin_zero: [0; 8],
        }
    }
    fn storage_mut(&mut self) -> *mut sockaddr_storage {
        self as *mut Self as *mut sockaddr_storage
    }
}

/// The C poller writes a sockaddr_in into the sockaddr_storage the callback
/// receives; the octets sit at fixed offsets (family, port, addr, zero).
fn sin_addr_of(addr: *mut sockaddr_storage) -> ([u8; 4], u16) {
    let mut b = [0u8; 128];
    if !addr.is_null() {
        unsafe { ptr::copy_nonoverlapping(addr as *const u8, b.as_mut_ptr(), 128) };
    }
    let port = u16::from_be_bytes([b[2], b[3]]);
    let ip = [b[4], b[5], b[6], b[7]];
    (ip, port)
}

// ---------------------------------------------------------------------------
// Receive callback (clientd carries the Vec; no statics).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct RxDatagram {
    transport: usize,
    data: Vec<u8>,
    src_ip: [u8; 4],
    src_port: u16,
    alloc_at_cb: c_int,
    released_at_cb: c_int,
}

unsafe extern "C" fn recv_cb(
    _data_paths: *mut aeron_udp_channel_data_paths_t,
    transport: *mut aeron_udp_channel_transport_t,
    receiver_clientd: *mut c_void,
    _endpoint_clientd: *mut c_void,
    _destination_clientd: *mut c_void,
    buffer: *mut u8,
    length: usize,
    addr: *mut sockaddr_storage,
    _media_timestamp: *mut timespec,
) {
    let rx = &mut *(receiver_clientd as *mut Vec<RxDatagram>);
    let data = std::slice::from_raw_parts(buffer, length).to_vec();
    let (src_ip, src_port) = sin_addr_of(addr);
    // Snapshot the fake's mbuf accounting: this proves whether the frame's mbuf
    // is still live (callback before recycle) or already released.
    rx.push(RxDatagram {
        transport: transport as usize,
        data,
        src_ip,
        src_port,
        alloc_at_cb: rusteron_dpdk_fake_allocated(),
        released_at_cb: rusteron_dpdk_fake_released(),
    });
}

// ---------------------------------------------------------------------------
// Harness: native runtime + one genuine Aeron transport + the receive Vec.
// ---------------------------------------------------------------------------

struct Harness {
    _env: TestEnv,
    native: *mut c_void,
    bindings: *mut aeron_udp_channel_transport_bindings_stct,
    transport: Box<aeron_udp_channel_transport_stct>,
    rx: Vec<RxDatagram>,
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
        Harness {
            _env: env,
            native,
            bindings,
            transport: Box::new(unsafe { std::mem::zeroed::<aeron_udp_channel_transport_stct>() }),
            rx: Vec::new(),
            initialized: false,
        }
    }

    fn init(&mut self, affinity: aeron_udp_channel_transport_affinity_t, bind_ip: [u8; 4], bind_port: u16, mtu: usize) {
        assert!(!self.initialized, "harness already initialized");
        init_transport(self.bindings, &mut self.transport, affinity, bind_ip, bind_port, mtu);
        self.initialized = true;
    }

    fn stats(&self) -> RxStats {
        let mut s = RxStats::default();
        unsafe { rusteron_dpdk_transport_test_rx_stats(self.native, &mut s) };
        s
    }

    fn poller(&self) -> Poller {
        Poller::new(self.bindings)
    }

    fn recvmmsg(&mut self, vlen: usize, bytes_rcved: &mut i64) -> c_int {
        let mut msgvec = vec![unsafe { std::mem::zeroed::<RealMmsghdr>() }; vlen];
        let clientd = &mut self.rx as *mut Vec<RxDatagram> as *mut c_void;
        unsafe {
            let b = &*self.bindings;
            b.recvmmsg_func.unwrap()(
                &mut *self.transport,
                msgvec.as_mut_ptr() as *mut mmsghdr,
                vlen,
                bytes_rcved,
                Some(recv_cb),
                clientd,
            )
        }
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

fn init_transport(
    bindings: *mut aeron_udp_channel_transport_bindings_stct,
    transport: &mut aeron_udp_channel_transport_stct,
    affinity: aeron_udp_channel_transport_affinity_t,
    bind_ip: [u8; 4],
    bind_port: u16,
    mtu: usize,
) {
    unsafe { ptr::write_bytes(transport as *mut _, 0, 1) };
    let mut bind = SockAddrIn::new(bind_ip, bind_port);
    let mut dummy = SockAddrIn::new([0, 0, 0, 0], 0);
    let mut params: aeron_udp_channel_transport_params_stct = unsafe { std::mem::zeroed() };
    params.mtu_length = mtu;
    let rc = unsafe {
        let b = &*bindings;
        b.init_func.unwrap()(
            transport,
            bind.storage_mut(),
            dummy.storage_mut(),
            ptr::null_mut(),
            &mut params,
            ptr::null_mut(),
            affinity,
        )
    };
    assert_eq!(rc, 0, "init failed: {}", last_error());
}

/// A second Aeron transport for multi-endpoint tests (closed by the caller).
fn init_extra_transport(
    bindings: *mut aeron_udp_channel_transport_bindings_stct,
    bind_ip: [u8; 4],
    bind_port: u16,
) -> Box<aeron_udp_channel_transport_stct> {
    let mut t = Box::new(unsafe { std::mem::zeroed::<aeron_udp_channel_transport_stct>() });
    init_transport(
        bindings,
        &mut t,
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
        bind_ip,
        bind_port,
        1472,
    );
    t
}

// ---------------------------------------------------------------------------
// Poller wrapper over the genuine bindings' poller callbacks.
// ---------------------------------------------------------------------------

struct Poller {
    inner: Box<aeron_udp_transport_poller_stct>,
    bindings: *mut aeron_udp_channel_transport_bindings_stct,
    inited: bool,
    added: Vec<*mut aeron_udp_channel_transport_t>,
}

impl Poller {
    fn new(bindings: *mut aeron_udp_channel_transport_bindings_stct) -> Self {
        Self {
            inner: Box::new(unsafe { std::mem::zeroed::<aeron_udp_transport_poller_stct>() }),
            bindings,
            inited: false,
            added: Vec::new(),
        }
    }

    fn init(&mut self) {
        if self.inited {
            return;
        }
        let rc = unsafe {
            let b = &*self.bindings;
            b.poller_init_func.unwrap()(
                &mut *self.inner,
                ptr::null_mut(),
                aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
            )
        };
        assert_eq!(rc, 0, "poller_init failed: {}", last_error());
        self.inited = true;
    }

    fn add(&mut self, transport: *mut aeron_udp_channel_transport_t) {
        self.try_add(transport)
            .unwrap_or_else(|e| panic!("poller_add failed: {e}"));
    }

    fn try_add(&mut self, transport: *mut aeron_udp_channel_transport_t) -> Result<(), String> {
        self.init();
        let rc = unsafe {
            let b = &*self.bindings;
            b.poller_add_func.unwrap()(&mut *self.inner, transport)
        };
        if rc == 0 {
            self.added.push(transport);
            Ok(())
        } else {
            Err(last_error())
        }
    }

    fn remove(&mut self, transport: *mut aeron_udp_channel_transport_t) {
        let rc = unsafe {
            let b = &*self.bindings;
            b.poller_remove_func.unwrap()(&mut *self.inner, transport)
        };
        assert_eq!(rc, 0, "poller_remove failed: {}", last_error());
    }

    fn poll(&mut self, vlen: usize, clientd: *mut c_void, bytes_rcved: &mut i64) -> c_int {
        let mut msgvec = vec![unsafe { std::mem::zeroed::<RealMmsghdr>() }; vlen];
        self.poll_with(&mut msgvec, clientd, bytes_rcved)
    }

    /// Poll with a caller-owned msgvec, so a steady-state loop never reallocates
    /// (the msgvec is Aeron's own reusable buffer in production).
    fn poll_with(&mut self, msgvec: &mut [RealMmsghdr], clientd: *mut c_void, bytes_rcved: &mut i64) -> c_int {
        unsafe {
            let b = &*self.bindings;
            b.poller_poll_func.unwrap()(
                &mut *self.inner,
                msgvec.as_mut_ptr() as *mut mmsghdr,
                msgvec.len(),
                bytes_rcved,
                Some(recv_cb),
                None,
                clientd,
            )
        }
    }
}

/// Close an extra transport via the genuine close callback (frees the native
/// client state it allocated in init).
fn close_transport(
    bindings: *mut aeron_udp_channel_transport_bindings_stct,
    transport: &mut Box<aeron_udp_channel_transport_stct>,
) {
    unsafe {
        let b = &*bindings;
        b.close_func.unwrap()(&mut **transport);
    }
}

impl Drop for Poller {
    fn drop(&mut self) {
        if !self.inited {
            return;
        }
        unsafe {
            let b = &*self.bindings;
            for t in self.added.drain(..) {
                b.poller_remove_func.unwrap()(&mut *self.inner, t);
            }
            b.poller_close_func.unwrap()(&mut *self.inner);
        }
    }
}

/// Linux x86_64 `struct mmsghdr` (bindings.rs only declares an opaque stub).
/// The C poller touches msg_name (offset 0) and msg_len (offset 48), both within
/// this 56-byte layout; it is zeroed so msg_name is NULL (the C side falls back
/// to its own source-address buffer).
#[repr(C)]
#[derive(Clone, Copy)]
struct RealMmsghdr {
    msg_hdr: msghdr,
    msg_len: u32,
    _pad: u32,
}

// ---------------------------------------------------------------------------
// Frame helpers (real checksums so rx_ol_flags = 0 exercises software verify).
// ---------------------------------------------------------------------------

const RECEIVER_PORT: u16 = 1;
const RECEIVER_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x02];
const RECEIVER_IP: [u8; 4] = [10, 0, 1, 1];
const PEER_IP: [u8; 4] = [10, 0, 1, 5];
const PEER_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x50];

const RX_GOOD: u32 = 0b11; // IPV4_CKSUM_GOOD | UDP_CKSUM_GOOD
const RX_IPV4_BAD: u32 = 0b100; // IPV4_CKSUM_BAD

fn ones_sum(words: &[u16]) -> u32 {
    let mut sum: u32 = 0;
    for w in words {
        sum += *w as u32;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    sum
}

/// IPv4 header checksum over the 20-byte header (checksum field read as zero).
fn ip_csum(hdr: &[u8]) -> u16 {
    let mut words = Vec::with_capacity(10);
    for i in 0..10 {
        words.push(((hdr[i * 2] as u16) << 8) | hdr[i * 2 + 1] as u16);
    }
    !(ones_sum(&words) as u16)
}

/// UDP checksum including the pseudo-header (UDP checksum field zeroed).
fn udp_csum(src_ip: [u8; 4], dst_ip: [u8; 4], sport: u16, dport: u16, payload: &[u8]) -> u16 {
    let udp_len = 8 + payload.len();
    let mut words = Vec::new();
    for i in 0..2 {
        words.push(((src_ip[i * 2] as u16) << 8) | src_ip[i * 2 + 1] as u16);
        words.push(((dst_ip[i * 2] as u16) << 8) | dst_ip[i * 2 + 1] as u16);
    }
    words.push(17); // IPPROTO_UDP
    words.push(udp_len as u16); // pseudo-header length
    words.push(sport);
    words.push(dport);
    words.push(udp_len as u16); // UDP length field
    words.push(0); // checksum field (zeroed during computation)
    for c in payload.chunks(2) {
        if c.len() == 2 {
            words.push(((c[0] as u16) << 8) | c[1] as u16);
        } else {
            words.push((c[0] as u16) << 8);
        }
    }
    let c = !(ones_sum(&words) as u16);
    if c == 0 {
        0xFFFF
    } else {
        c
    }
}

/// A valid IPv4/UDP frame: real IPv4 + UDP checksums, DF set, unicast MACs.
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
    f.extend_from_slice(&((20 + 8 + payload.len()) as u16).to_be_bytes());
    f.extend_from_slice(&[0x00, 0x00]); // id
    f.extend_from_slice(&[0x40, 0x00]); // DF
    f.push(64);
    f.push(17);
    f.extend_from_slice(&[0x00, 0x00]); // csum placeholder
    f.extend_from_slice(&src_ip);
    f.extend_from_slice(&dst_ip);
    let csum = ip_csum(&f[14..34]);
    f[24] = (csum >> 8) as u8;
    f[25] = (csum & 0xFF) as u8;
    f.extend_from_slice(&sport.to_be_bytes());
    f.extend_from_slice(&dport.to_be_bytes());
    f.extend_from_slice(&((8 + payload.len()) as u16).to_be_bytes());
    f.extend_from_slice(&[0x00, 0x00]); // csum placeholder
    let uc = udp_csum(src_ip, dst_ip, sport, dport, payload);
    let n = f.len();
    f[n - 2] = (uc >> 8) as u8;
    f[n - 1] = (uc & 0xFF) as u8;
    f.extend_from_slice(payload);
    f
}

/// A valid frame addressed to the receiver transport's own endpoint.
fn frame_to_receiver(dport: u16, payload: &[u8]) -> Vec<u8> {
    build_udp_frame(RECEIVER_MAC, PEER_MAC, PEER_IP, RECEIVER_IP, 7000, dport, payload)
}

/// ARP request for the receiver's IPv4 (broadcast dst, like a real probe).
fn arp_request_for_receiver(sha: [u8; 6], spa: [u8; 4]) -> Vec<u8> {
    let mut f = Vec::new();
    f.extend_from_slice(&[0xff; 6]);
    f.extend_from_slice(&sha);
    f.extend_from_slice(&[0x08, 0x06]);
    f.extend_from_slice(&1u16.to_be_bytes()); // htype
    f.extend_from_slice(&0x0800u16.to_be_bytes()); // ptype
    f.push(6);
    f.push(4);
    f.extend_from_slice(&1u16.to_be_bytes()); // oper request
    f.extend_from_slice(&sha);
    f.extend_from_slice(&spa);
    f.extend_from_slice(&[0; 6]);
    f.extend_from_slice(&RECEIVER_IP);
    f
}

fn assert_no_leak() {
    let allocated = unsafe { rusteron_dpdk_fake_allocated() };
    let released = unsafe { rusteron_dpdk_fake_released() };
    assert_eq!(
        allocated, released,
        "mbuf leak: allocated={allocated} released={released}"
    );
}

// ---------------------------------------------------------------------------
// Accepted frames
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn valid_frame_software_checksums_dispatched() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
        RECEIVER_IP,
        60000,
        1472,
    );
    let mut p = h.poller();
    p.add(&mut *h.transport);

    // rx_ol_flags = 0: the NIC left the verdict unknown, so the transport must
    // software-verify the real IPv4 + UDP checksums (plan §7.6).
    let frame = frame_to_receiver(60000, b"hello rx");
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, frame.as_ptr(), frame.len(), 0, 1) },
        0
    );
    let mut bytes = 0i64;
    let clientd = &mut h.rx as *mut Vec<RxDatagram> as *mut c_void;
    let work = p.poll(16, clientd, &mut bytes);
    assert_eq!(work, 1, "one datagram dispatched");
    assert_eq!(bytes, 8, "bytes_rcved accrues the payload length only");
    assert_eq!(h.rx.len(), 1);
    let d = &h.rx[0];
    assert_eq!(d.transport, &mut *h.transport as *mut _ as usize, "dispatch target");
    assert_eq!(d.data, b"hello rx");
    assert_eq!(d.src_ip, PEER_IP);
    assert_eq!(d.src_port, 7000);
    let s = h.stats();
    assert_eq!(s.accepted, 1);
    assert_eq!(s.foreign_dst, 0);
    assert_eq!(s.unknown_port, 0);
    assert_no_leak();
}

#[test]
#[serial]
fn nic_good_verdict_trusted_skips_software_verify() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
        RECEIVER_IP,
        60000,
        1472,
    );
    let mut p = h.poller();
    p.add(&mut *h.transport);

    // A frame whose UDP checksum field is NOT a valid final checksum (it carries
    // the TX-offload pseudo-header seed): with NIC GOOD verdicts the transport
    // must trust the NIC and accept it anyway.
    let mut frame = frame_to_receiver(60000, b"trust nic");
    frame[41] ^= 0xFF; // corrupt the UDP checksum field so software verify would reject
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, frame.as_ptr(), frame.len(), RX_GOOD, 1) },
        0
    );
    let mut bytes = 0i64;
    let clientd = &mut h.rx as *mut Vec<RxDatagram> as *mut c_void;
    let work = p.poll(16, clientd, &mut bytes);
    assert_eq!(work, 1, "NIC GOOD verdict is trusted");
    assert_eq!(h.stats().accepted, 1);
    assert_no_leak();
}

#[test]
#[serial]
fn recvmmsg_dispatches_single_transport() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
        RECEIVER_IP,
        60000,
        1472,
    );

    let frame = frame_to_receiver(60000, b"direct recvmmsg");
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, frame.as_ptr(), frame.len(), 0, 1) },
        0
    );
    let mut bytes = 0i64;
    let work = h.recvmmsg(16, &mut bytes);
    assert_eq!(work, 1);
    assert_eq!(bytes, 15, "payload length only");
    assert_eq!(h.rx.len(), 1);
    assert_eq!(h.rx[0].data, b"direct recvmmsg");
    assert_eq!(h.stats().accepted, 1);
    assert_no_leak();
}

#[test]
#[serial]
fn two_endpoints_same_ena_dispatched_by_map() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
        RECEIVER_IP,
        60000,
        1472,
    );
    let mut t2 = init_extra_transport(h.bindings, RECEIVER_IP, 60001);
    let mut p = h.poller();
    p.add(&mut *h.transport);
    p.add(&mut *t2);

    // Two frames on the same ENA (receiver port), one per endpoint; the poller
    // must drain the shared port once and dispatch each to its own transport.
    let f1 = frame_to_receiver(60000, b"to-endpoint-a");
    let f2 = frame_to_receiver(60001, b"to-endpoint-b");
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, f1.as_ptr(), f1.len(), 0, 1) },
        0
    );
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, f2.as_ptr(), f2.len(), 0, 1) },
        0
    );
    let mut bytes = 0i64;
    let clientd = &mut h.rx as *mut Vec<RxDatagram> as *mut c_void;
    let work = p.poll(16, clientd, &mut bytes);
    assert_eq!(work, 2, "both endpoints dispatched in one poll");
    assert_eq!(bytes, 26, "13-byte + 13-byte payloads");
    assert_eq!(h.rx.len(), 2, "exactly one callback per datagram");
    let a = h.rx.iter().find(|d| d.data == b"to-endpoint-a").unwrap();
    let b = h.rx.iter().find(|d| d.data == b"to-endpoint-b").unwrap();
    assert_eq!(a.transport, &mut *h.transport as *mut _ as usize);
    assert_eq!(b.transport, &mut *t2 as *mut _ as usize);
    assert_eq!(h.stats().accepted, 2);
    assert_no_leak();
    close_transport(h.bindings, &mut t2);
}

// ---------------------------------------------------------------------------
// ARP routing from the receive loop
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn arp_request_handled_and_reply_sent() {
    let mut h = Harness::new();
    unsafe { rusteron_dpdk_test_set_clock_ms(1000) };
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
        RECEIVER_IP,
        60000,
        1472,
    );
    let mut p = h.poller();
    p.add(&mut *h.transport);

    let req = arp_request_for_receiver(PEER_MAC, PEER_IP);
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, req.as_ptr(), req.len(), 0, 1) },
        0
    );
    let mut bytes = 0i64;
    let clientd = &mut h.rx as *mut Vec<RxDatagram> as *mut c_void;
    let work = p.poll(16, clientd, &mut bytes);
    assert_eq!(work, 0, "ARP is not a UDP datagram");
    assert_eq!(h.rx.len(), 0, "no Aeron callback for ARP");
    assert_eq!(h.stats().arp, 1, "ARP frames are counted");

    // The handler answered with a unicast ARP reply on the same ENA.
    let n = unsafe { rusteron_dpdk_fake_capture_count() };
    assert_eq!(n, 1, "one ARP reply transmitted");
    let mut c = unsafe { std::mem::zeroed::<FakeCapture>() };
    assert_eq!(unsafe { rusteron_dpdk_fake_capture_at(0, &mut c) }, 0);
    assert_eq!(&c.data[0..6], &PEER_MAC, "reply dst = requester's MAC, not broadcast");
    assert_eq!(&c.data[6..12], &RECEIVER_MAC, "reply src = this role's MAC");
    assert_eq!(c.data[12], 0x08);
    assert_eq!(c.data[13], 0x06, "ARP ethertype");
    // ARP body: oper at +20, sender hw at +22, sender proto at +28, target proto at +38.
    assert_eq!(u16::from_be_bytes([c.data[14 + 6], c.data[14 + 7]]), 2, "oper = reply");
    assert_eq!(&c.data[14 + 8..14 + 14], &RECEIVER_MAC, "reply sender hw = us");
    assert_eq!(&c.data[14 + 14..14 + 18], &RECEIVER_IP, "reply sender proto = us");
    assert_eq!(&c.data[14 + 24..14 + 28], &PEER_IP, "reply target proto = requester");
    assert_no_leak();
}

// ---------------------------------------------------------------------------
// Rejected frame classes (plan §7.6)
// ---------------------------------------------------------------------------

fn assert_reject(
    h: &mut Harness,
    p: &mut Poller,
    frame: &[u8],
    ol_flags: u32,
    nb_segs: u32,
    stat: impl Fn(&RxStats) -> u64,
) {
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, frame.as_ptr(), frame.len(), ol_flags, nb_segs) },
        0,
        "inject failed (frame too long?)"
    );
    let mut bytes = 0i64;
    let clientd = &mut h.rx as *mut Vec<RxDatagram> as *mut c_void;
    let work = p.poll(16, clientd, &mut bytes);
    assert_eq!(work, 0, "rejected frame must not dispatch");
    assert_eq!(bytes, 0);
    assert_eq!(h.rx.len(), 0, "no callback for a rejected frame");
    let s = h.stats();
    assert_eq!(stat(&s), 1, "the reject bucket must count");
    assert_no_leak();
}

/// Return the poller first so it drops before the harness (its Drop removes the
/// transport from the map while the transport is still alive).
fn reject_setup() -> (Poller, Harness) {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
        RECEIVER_IP,
        60000,
        1472,
    );
    let mut p = h.poller();
    p.add(&mut *h.transport);
    (p, h)
}

#[test]
#[serial]
fn reject_ipv6() {
    let (mut p, mut h) = reject_setup();
    let mut frame = frame_to_receiver(60000, b"x");
    frame[12] = 0x86;
    frame[13] = 0xDD;
    assert_reject(&mut h, &mut p, &frame, 0, 1, |s| s.ipv6);
}

#[test]
#[serial]
fn reject_vlan() {
    let (mut p, mut h) = reject_setup();
    let mut frame = frame_to_receiver(60000, b"x");
    frame[12] = 0x81;
    frame[13] = 0x00;
    assert_reject(&mut h, &mut p, &frame, 0, 1, |s| s.vlan);
}

#[test]
#[serial]
fn reject_unknown_ethertype() {
    let (mut p, mut h) = reject_setup();
    let mut frame = frame_to_receiver(60000, b"x");
    frame[12] = 0x12;
    frame[13] = 0x34;
    assert_reject(&mut h, &mut p, &frame, 0, 1, |s| s.ethertype);
}

#[test]
#[serial]
fn reject_multicast_mac() {
    let (mut p, mut h) = reject_setup();
    let mut frame = frame_to_receiver(60000, b"x");
    frame[0] = 0x01; // multicast dst MAC (LSB of first octet set)
    assert_reject(&mut h, &mut p, &frame, 0, 1, |s| s.multicast);
}

#[test]
#[serial]
fn reject_multicast_ip() {
    let (mut p, mut h) = reject_setup();
    let frame = build_udp_frame(RECEIVER_MAC, PEER_MAC, PEER_IP, [224, 0, 0, 1], 7000, 60000, b"x");
    assert_reject(&mut h, &mut p, &frame, 0, 1, |s| s.multicast);
}

#[test]
#[serial]
fn reject_ip_options() {
    let (mut p, mut h) = reject_setup();
    let mut frame = frame_to_receiver(60000, b"x");
    frame[14] = 0x46; // IHL = 6 (options present)
    assert_reject(&mut h, &mut p, &frame, 0, 1, |s| s.ip_options);
}

#[test]
#[serial]
fn reject_ip_fragment() {
    let (mut p, mut h) = reject_setup();
    let mut frame = frame_to_receiver(60000, b"x");
    frame[20] = 0x20; // MF set
    assert_reject(&mut h, &mut p, &frame, 0, 1, |s| s.fragment);
}

#[test]
#[serial]
fn reject_ip_fragment_offset() {
    let (mut p, mut h) = reject_setup();
    let mut frame = frame_to_receiver(60000, b"x");
    frame[20] = 0x00;
    frame[21] = 0x01; // non-zero fragment offset
    assert_reject(&mut h, &mut p, &frame, 0, 1, |s| s.fragment);
}

#[test]
#[serial]
fn reject_non_udp_protocol() {
    let (mut p, mut h) = reject_setup();
    let mut frame = frame_to_receiver(60000, b"x");
    frame[23] = 6; // TCP
    assert_reject(&mut h, &mut p, &frame, 0, 1, |s| s.protocol);
}

#[test]
#[serial]
fn reject_truncated_short_frame() {
    let (mut p, mut h) = reject_setup();
    let frame = frame_to_receiver(60000, b"x");
    assert_reject(&mut h, &mut p, &frame[..20], 0, 1, |s| s.truncated);
}

#[test]
#[serial]
fn reject_truncated_length_inconsistent() {
    let (mut p, mut h) = reject_setup();
    let mut frame = frame_to_receiver(60000, b"x");
    frame[16] = 0x00;
    frame[17] = 0xC8; // total_len (200) exceeds the real frame length
    assert_reject(&mut h, &mut p, &frame, 0, 1, |s| s.truncated);
}

#[test]
#[serial]
fn reject_bad_ip_checksum() {
    let (mut p, mut h) = reject_setup();
    let mut frame = frame_to_receiver(60000, b"x");
    frame[24] ^= 0xFF; // corrupt the IPv4 header checksum
    assert_reject(&mut h, &mut p, &frame, 0, 1, |s| s.checksum);
}

#[test]
#[serial]
fn reject_bad_udp_checksum() {
    let (mut p, mut h) = reject_setup();
    let mut frame = frame_to_receiver(60000, b"payload");
    let n = frame.len();
    frame[n - 1] ^= 0xFF; // corrupt a payload byte -> UDP checksum invalid
    assert_reject(&mut h, &mut p, &frame, 0, 1, |s| s.checksum);
}

#[test]
#[serial]
fn reject_nic_bad_checksum() {
    let (mut p, mut h) = reject_setup();
    let frame = frame_to_receiver(60000, b"x");
    assert_reject(&mut h, &mut p, &frame, RX_IPV4_BAD, 1, |s| s.checksum);
}

#[test]
#[serial]
fn reject_multi_segment() {
    let (mut p, mut h) = reject_setup();
    let frame = frame_to_receiver(60000, b"x");
    assert_reject(&mut h, &mut p, &frame, 0, 2, |s| s.multi_segment);
}

// ---------------------------------------------------------------------------
// Dispatch statistics and lifecycle
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn foreign_dst_and_unknown_port_counted_not_dispatched() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
        RECEIVER_IP,
        60000,
        1472,
    );
    let mut p = h.poller();
    p.add(&mut *h.transport);

    // Valid frame for a foreign destination IP on this NIC.
    let foreign = build_udp_frame(RECEIVER_MAC, PEER_MAC, PEER_IP, [10, 0, 1, 99], 7000, 60000, b"f");
    // Valid frame for our IP but an unregistered port.
    let unknown = frame_to_receiver(60099, b"u");

    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, foreign.as_ptr(), foreign.len(), 0, 1) },
        0
    );
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, unknown.as_ptr(), unknown.len(), 0, 1) },
        0
    );
    let mut bytes = 0i64;
    let clientd = &mut h.rx as *mut Vec<RxDatagram> as *mut c_void;
    let work = p.poll(16, clientd, &mut bytes);
    assert_eq!(work, 0);
    assert_eq!(bytes, 0);
    assert_eq!(h.rx.len(), 0);
    let s = h.stats();
    assert_eq!(s.foreign_dst, 1, "wrong-destination IP counted separately");
    assert_eq!(s.unknown_port, 1, "unregistered port on our IP counted separately");
    assert_eq!(s.accepted, 0);
    assert_no_leak();
}

#[test]
#[serial]
fn callback_before_recycle_ordering() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
        RECEIVER_IP,
        60000,
        1472,
    );
    let mut p = h.poller();
    p.add(&mut *h.transport);

    // Two frames in one burst: rx_burst allocates both views up front, so the
    // first callback must observe the second mbuf still live, and neither frame
    // may be released before its own callback runs (plan §7.6).
    let f1 = frame_to_receiver(60000, b"first");
    let f2 = frame_to_receiver(60000, b"second");
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, f1.as_ptr(), f1.len(), 0, 1) },
        0
    );
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, f2.as_ptr(), f2.len(), 0, 1) },
        0
    );
    let mut bytes = 0i64;
    let clientd = &mut h.rx as *mut Vec<RxDatagram> as *mut c_void;
    let work = p.poll(16, clientd, &mut bytes);
    assert_eq!(work, 2);
    assert_eq!(h.rx.len(), 2);

    let (a, b) = (&h.rx[0], &h.rx[1]);
    assert_eq!(a.data, b"first");
    assert_eq!(b.data, b"second");
    // Both views were allocated before any callback ran.
    assert_eq!(a.alloc_at_cb, 2, "frame 1's mbuf is live during frame 0's callback");
    assert_eq!(a.released_at_cb, 0, "frame 0 recycled strictly after its callback");
    assert_eq!(b.released_at_cb, 1, "frame 0 recycled before frame 1's callback");
    assert_eq!(b.alloc_at_cb, 2);
    assert_no_leak();
}

#[test]
#[serial]
fn poll_caps_to_vlen_and_accumulates_bytes() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
        RECEIVER_IP,
        60000,
        1472,
    );
    let mut p = h.poller();
    p.add(&mut *h.transport);

    let f1 = frame_to_receiver(60000, b"one");
    let f2 = frame_to_receiver(60000, b"two");
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, f1.as_ptr(), f1.len(), 0, 1) },
        0
    );
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, f2.as_ptr(), f2.len(), 0, 1) },
        0
    );
    let mut bytes = 0i64;
    let clientd = &mut h.rx as *mut Vec<RxDatagram> as *mut c_void;
    assert_eq!(p.poll(1, clientd, &mut bytes), 1, "vlen=1 caps the poll");
    assert_eq!(bytes, 3);
    assert_eq!(h.rx.len(), 1);
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_queued(RECEIVER_PORT) },
        1,
        "one frame still queued"
    );

    assert_eq!(p.poll(16, clientd, &mut bytes), 1, "the second poll drains the rest");
    assert_eq!(bytes, 6, "bytes_rcved accumulates across polls");
    assert_eq!(h.rx.len(), 2);
    assert_eq!(h.stats().accepted, 2);
    assert_no_leak();
}

#[test]
#[serial]
fn burst_size_caps_poll_batch() {
    let mut h = Harness::new_with_burst(2);
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
        RECEIVER_IP,
        60000,
        1472,
    );
    let mut p = h.poller();
    p.add(&mut *h.transport);

    let frames: Vec<Vec<u8>> = (0..3).map(|i| frame_to_receiver(60000, &[b'a' + i as u8; 4])).collect();
    for f in &frames {
        assert_eq!(
            unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, f.as_ptr(), f.len(), 0, 1) },
            0
        );
    }
    let mut bytes = 0i64;
    let clientd = &mut h.rx as *mut Vec<RxDatagram> as *mut c_void;
    assert_eq!(p.poll(16, clientd, &mut bytes), 2, "burst_size caps the batch");
    assert_eq!(h.rx.len(), 2);
    assert_eq!(unsafe { rusteron_dpdk_fake_rx_queued(RECEIVER_PORT) }, 1);
    assert_eq!(p.poll(16, clientd, &mut bytes), 1, "remaining frame on the next poll");
    assert_eq!(h.rx.len(), 3);
    assert_eq!(h.stats().accepted, 3);
    assert_no_leak();
}

#[test]
#[serial]
fn zero_allocations_during_empty_steady_state_poll() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
        RECEIVER_IP,
        60000,
        1472,
    );
    let mut p = h.poller();
    p.add(&mut *h.transport);

    let before_alloc = unsafe { rusteron_dpdk_fake_allocated() };
    // A caller-owned msgvec (Aeron's reusable buffer in production) so the loop
    // itself performs no allocation: any growth in the mbuf accounting would
    // prove the C hot path allocates.
    let mut msgvec = vec![unsafe { std::mem::zeroed::<RealMmsghdr>() }; 16];
    let mut bytes = 0i64;
    let clientd = &mut h.rx as *mut Vec<RxDatagram> as *mut c_void;
    for _ in 0..1000 {
        assert_eq!(p.poll_with(&mut msgvec, clientd, &mut bytes), 0);
    }
    assert_eq!(bytes, 0);
    assert_eq!(
        unsafe { rusteron_dpdk_fake_allocated() },
        before_alloc,
        "no mbufs in steady state"
    );
    assert_no_leak();
}

#[test]
#[serial]
fn duplicate_endpoint_add_rejected() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
        RECEIVER_IP,
        60000,
        1472,
    );
    let mut t2 = init_extra_transport(h.bindings, RECEIVER_IP, 60000); // same endpoint!
    let mut p = h.poller();
    p.add(&mut *h.transport);
    let err = p.try_add(&mut *t2).unwrap_err();
    assert!(err.contains("duplicate"), "got: {err}");

    // The first registration still dispatches.
    let frame = frame_to_receiver(60000, b"still works");
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, frame.as_ptr(), frame.len(), 0, 1) },
        0
    );
    let mut bytes = 0i64;
    let clientd = &mut h.rx as *mut Vec<RxDatagram> as *mut c_void;
    assert_eq!(p.poll(16, clientd, &mut bytes), 1);
    assert_eq!(h.rx[0].transport, &mut *h.transport as *mut _ as usize);
    assert_no_leak();
    close_transport(h.bindings, &mut t2);
}

#[test]
#[serial]
fn remove_unregisters_endpoint() {
    let mut h = Harness::new();
    h.init(
        aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
        RECEIVER_IP,
        60000,
        1472,
    );
    // A second endpoint on the same ENA keeps the poller draining the port after
    // the first is removed (an empty poller would return before any rx_burst).
    let mut t2 = init_extra_transport(h.bindings, RECEIVER_IP, 60001);
    let mut p = h.poller();
    p.add(&mut *h.transport);
    p.add(&mut *t2);

    // Before removal: dispatched to the first transport.
    let frame = frame_to_receiver(60000, b"before");
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, frame.as_ptr(), frame.len(), 0, 1) },
        0
    );
    let mut bytes = 0i64;
    let clientd = &mut h.rx as *mut Vec<RxDatagram> as *mut c_void;
    assert_eq!(p.poll(16, clientd, &mut bytes), 1);
    assert_eq!(h.rx.len(), 1);

    // After removal: the same frame is no longer dispatched and is counted as an
    // unknown port (the endpoint map entry is gone).
    p.remove(&mut *h.transport);
    let frame = frame_to_receiver(60000, b"after");
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, frame.as_ptr(), frame.len(), 0, 1) },
        0
    );
    assert_eq!(p.poll(16, clientd, &mut bytes), 0);
    assert_eq!(h.rx.len(), 1, "no dispatch after removal");
    let s = h.stats();
    assert_eq!(s.accepted, 1);
    assert_eq!(s.unknown_port, 1, "the unregistered endpoint is counted");
    assert_no_leak();
    close_transport(h.bindings, &mut t2);
}
