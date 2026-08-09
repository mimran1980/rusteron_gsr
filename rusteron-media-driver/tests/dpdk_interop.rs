//! Bidirectional Aeron semantics and interoperability over DPDK (Ticket 6,
//! plan §8).
//!
//! Drives the real Aeron binding callbacks on both roles of a single native
//! runtime — a sender-affinity transport and a receiver-affinity transport on
//! the same two ENA ports — and asserts the §8 interoperability contract:
//!   * DPDK frames preserve Aeron's UDP payload bytes exactly (a data frame
//!     emitted by the TX path arrives byte-identical at the RX path).
//!   * Connected and unconnected transports coexist and follow their own
//!     addressing.
//!   * bind_addr_and_port reports the effective local IPv4/port (the wildcard
//!     port manager remains the authority for ephemeral ports).
//!   * reconnect replaces the copied remote address without retaining a
//!     caller-owned pointer.
//!   * interceptors are reachable: the poller forwards `target->data_paths` to
//!     the recv callback exactly as the kernel socket path does.
//!   * NAK/status/setup/RTT/data/retransmit role frames are opaque UDP payloads
//!     dispatched to the correct endpoint (manual and dynamic MDC); removing an
//!     endpoint stops its dispatch.
//!   * every emitted datagram stays within `max_aeron_mtu` and never fragments
//!     (DF set), with oversized iovecs rejected.
//!
//! Linux x86_64 only; links the same archives as dpdk_rx.rs/dpdk_tx.rs.
#![cfg(all(feature = "dpdk", target_os = "linux", target_arch = "x86_64"))]

mod common;
use common::*;

use serial_test::serial;

use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// ---------------------------------------------------------------------------
// Native test hooks (fakes + runtime).
// ---------------------------------------------------------------------------

extern "C" {
    fn rusteron_dpdk_transport_bindings() -> *mut aeron_udp_channel_transport_bindings_stct;
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
            // s_addr is read natively (no ntohl), so the octets must sit in
            // memory in order (see dpdk_tx.rs).
            sin_addr: u32::from_le_bytes(ip),
            sin_zero: [0; 8],
        }
    }
    fn storage_mut(&mut self) -> *mut sockaddr_storage {
        self as *mut Self as *mut sockaddr_storage
    }
}

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
// Receive callback (clientd carries the Vec; records data_paths + target so the
// interop contract is observable from the callback).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct RxDatagram {
    data_paths: usize,
    transport: usize,
    data: Vec<u8>,
    src_ip: [u8; 4],
    src_port: u16,
}

unsafe extern "C" fn recv_cb(
    data_paths: *mut aeron_udp_channel_data_paths_t,
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
    rx.push(RxDatagram {
        data_paths: data_paths as usize,
        transport: transport as usize,
        data,
        src_ip,
        src_port,
    });
}

// ---------------------------------------------------------------------------
// Harness: one native runtime (both ENA ports) + a sender- and receiver-
// affinity Aeron transport, exactly as one media-driver process would own it.
// ---------------------------------------------------------------------------

struct Harness {
    _env: TestEnv,
    native: *mut c_void,
    bindings: *mut aeron_udp_channel_transport_bindings_stct,
    tx: Box<aeron_udp_channel_transport_stct>,
    rx: Box<aeron_udp_channel_transport_stct>,
    data_paths: Box<aeron_udp_channel_data_paths_stct>,
    rx_dgrams: Vec<RxDatagram>,
    tx_inited: bool,
    rx_inited: bool,
}

impl Harness {
    fn new() -> Self {
        let env = TestEnv::new();
        env.eal_skip();
        let native = create(&rusteron_dpdk_config_t::valid()).unwrap_or_else(|e| panic!("create failed: {e}"));
        let bindings = unsafe { rusteron_dpdk_transport_bindings() };
        assert!(!bindings.is_null());
        Harness {
            _env: env,
            native,
            bindings,
            tx: Box::new(unsafe { std::mem::zeroed::<aeron_udp_channel_transport_stct>() }),
            rx: Box::new(unsafe { std::mem::zeroed::<aeron_udp_channel_transport_stct>() }),
            data_paths: Box::new(unsafe { std::mem::zeroed::<aeron_udp_channel_data_paths_stct>() }),
            rx_dgrams: Vec::new(),
            tx_inited: false,
            rx_inited: false,
        }
    }

    /// The sender-affinity transport: emits frames on the sender ENA.
    fn init_tx(&mut self, bind_ip: [u8; 4], bind_port: u16, connect: Option<([u8; 4], u16)>, mtu: usize) {
        assert!(!self.tx_inited, "tx already initialized");
        self.tx_inited = true;
        let t = &mut *self.tx;
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
                aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
            )
        };
        assert_eq!(rc, 0, "tx init failed: {}", last_error());
    }

    /// The receiver-affinity transport: receives frames on the receiver ENA.
    fn init_rx(&mut self, bind_ip: [u8; 4], bind_port: u16, mtu: usize) {
        assert!(!self.rx_inited, "rx already initialized");
        self.rx_inited = true;
        let t = &mut *self.rx;
        unsafe { ptr::write_bytes(t as *mut _, 0, 1) };
        let mut bind = SockAddrIn::new(bind_ip, bind_port);
        let mut dummy = SockAddrIn::new([0, 0, 0, 0], 0);
        let mut params: aeron_udp_channel_transport_params_stct = unsafe { std::mem::zeroed() };
        params.mtu_length = mtu;
        let rc = unsafe {
            let b = &*self.bindings;
            b.init_func.unwrap()(
                t,
                bind.storage_mut(),
                dummy.storage_mut(),
                ptr::null_mut(),
                &mut params,
                ptr::null_mut(),
                aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
            )
        };
        assert_eq!(rc, 0, "rx init failed: {}", last_error());
    }

    fn send_to(&mut self, dst: ([u8; 4], u16), payloads: &[&[u8]]) -> (c_int, i64) {
        let mut d = SockAddrIn::new(dst.0, dst.1);
        self.send_common(d.storage_mut(), payloads)
    }

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
                &mut *self.tx,
                addr,
                iovs.as_ptr() as *mut iovec,
                iovs.len(),
                &mut bytes_sent,
            )
        };
        (rc, bytes_sent)
    }

    /// Call reconnect_func with a caller-owned address that is dropped right
    /// after the call — any subsequent connected send that reaches the new
    /// address proves the native side copied it (no pointer retention).
    fn reconnect(&mut self, new_addr: ([u8; 4], u16)) {
        let mut a = SockAddrIn::new(new_addr.0, new_addr.1);
        let rc = unsafe {
            let b = &*self.bindings;
            b.reconnect_func.unwrap()(&mut *self.tx, a.storage_mut())
        };
        assert_eq!(rc, 0, "reconnect failed: {}", last_error());
    }

    fn bind_addr_and_port(
        bindings: *mut aeron_udp_channel_transport_bindings_stct,
        transport: &mut Box<aeron_udp_channel_transport_stct>,
    ) -> String {
        let mut buf = [0 as c_char; 128];
        let rc = unsafe {
            let b = &*bindings;
            b.bind_addr_and_port_func.unwrap()(&mut **transport, buf.as_mut_ptr(), buf.len())
        };
        assert_eq!(rc, 0, "bind_addr_and_port failed: {}", last_error());
        unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) }
            .to_string_lossy()
            .into_owned()
    }

    fn arp_seed(&self, ip: [u8; 4], mac: [u8; 6]) {
        let ip_str = std::ffi::CString::new(format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])).unwrap();
        let rc = unsafe { rusteron_dpdk_transport_test_arp_seed(self.native, ip_str.as_ptr(), mac.as_ptr()) };
        assert_eq!(rc, 0, "arp_seed failed: {}", last_error());
    }

    fn stats(&self) -> RxStats {
        let mut s = RxStats::default();
        unsafe { rusteron_dpdk_transport_test_rx_stats(self.native, &mut s) };
        s
    }

    fn poller(&self) -> Poller {
        Poller::new(self.bindings)
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        unsafe {
            let b = &*self.bindings;
            if self.rx_inited {
                b.close_func.unwrap()(&mut *self.rx);
            }
            if self.tx_inited {
                b.close_func.unwrap()(&mut *self.tx);
            }
        }
        close(self.native);
    }
}

// ---------------------------------------------------------------------------
// Poller wrapper over the genuine bindings' poller callbacks (as dpdk_rx.rs).
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

    fn add(&mut self, transport: *mut aeron_udp_channel_transport_t) {
        if !self.inited {
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
        let rc = unsafe {
            let b = &*self.bindings;
            b.poller_add_func.unwrap()(&mut *self.inner, transport)
        };
        assert_eq!(rc, 0, "poller_add failed: {}", last_error());
        self.added.push(transport);
    }

    fn remove(&mut self, transport: *mut aeron_udp_channel_transport_t) {
        let rc = unsafe {
            let b = &*self.bindings;
            b.poller_remove_func.unwrap()(&mut *self.inner, transport)
        };
        assert_eq!(rc, 0, "poller_remove failed: {}", last_error());
        self.added.retain(|&t| t != transport);
    }

    fn poll(&mut self, vlen: usize, clientd: *mut c_void, bytes_rcved: &mut i64) -> c_int {
        let mut msgvec = vec![unsafe { std::mem::zeroed::<RealMmsghdr>() }; vlen];
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

#[repr(C)]
#[derive(Clone, Copy)]
struct RealMmsghdr {
    msg_hdr: msghdr,
    msg_len: u32,
    _pad: u32,
}

/// A second receiver-affinity transport for multi-endpoint tests (closed by
/// the caller). The local IPv4 must be the receiver role's own IP so the
/// endpoint map keys it alongside the primary rx transport.
fn init_extra_transport(h: &mut Harness, bind_port: u16) -> Box<aeron_udp_channel_transport_stct> {
    let mut t = Box::new(unsafe { std::mem::zeroed::<aeron_udp_channel_transport_stct>() });
    let t_ptr = &mut *t;
    unsafe { ptr::write_bytes(t_ptr as *mut _, 0, 1) };
    let mut bind = SockAddrIn::new(RECEIVER_IP, bind_port);
    let mut dummy = SockAddrIn::new([0, 0, 0, 0], 0);
    let mut params: aeron_udp_channel_transport_params_stct = unsafe { std::mem::zeroed() };
    params.mtu_length = 1472;
    let rc = unsafe {
        let b = &*h.bindings;
        b.init_func.unwrap()(
            t_ptr,
            bind.storage_mut(),
            dummy.storage_mut(),
            ptr::null_mut(),
            &mut params,
            ptr::null_mut(),
            aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
        )
    };
    assert_eq!(rc, 0, "extra transport init failed: {}", last_error());
    t
}

fn close_transport(
    bindings: *mut aeron_udp_channel_transport_bindings_stct,
    transport: &mut Box<aeron_udp_channel_transport_stct>,
) {
    unsafe {
        let b = &*bindings;
        b.close_func.unwrap()(&mut **transport);
    }
}

// ---------------------------------------------------------------------------
// Frame helpers.
// ---------------------------------------------------------------------------

const RECEIVER_PORT: u16 = 1; // fake port id of the receiver ENA
const SENDER_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x01];
const RECEIVER_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x02];
const SENDER_IP: [u8; 4] = [10, 0, 0, 1];
const RECEIVER_IP: [u8; 4] = [10, 0, 1, 1];

const IP: usize = 14;
const UDP: usize = IP + 20;
const PAYLOAD: usize = UDP + 8;

const RX_GOOD: u32 = 0b11; // IPV4_CKSUM_GOOD | UDP_CKSUM_GOOD

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

fn udp_payload(c: &FakeCapture) -> &[u8] {
    &c.data[PAYLOAD..c.len as usize]
}

fn u32_ip_at(c: &FakeCapture, off: usize) -> [u8; 4] {
    c.data[off..off + 4].try_into().unwrap()
}

fn u16_at(c: &FakeCapture, off: usize) -> u16 {
    u16::from_be_bytes([c.data[off], c.data[off + 1]])
}

fn ip_flags_frag(c: &FakeCapture) -> u16 {
    u16_at(c, IP + 6)
}

/// A plausible Aeron data frame (little-endian header, type DATA). The
/// transport treats it as opaque bytes; the layout matches aeron's wire format
/// so an unmodified peer could decode it.
fn aeron_data_frame(payload: &[u8]) -> Vec<u8> {
    let mut f = Vec::new();
    let frame_len = (24 + payload.len()) as u16;
    f.extend_from_slice(&frame_len.to_le_bytes()); // frame_length
    f.push(0); // version
    f.push(0x18); // flags: BEGIN | END
    f.extend_from_slice(&0x0001u16.to_le_bytes()); // type = DATA
    f.extend_from_slice(&0x0000_0123i32.to_le_bytes()); // session_id
    f.extend_from_slice(&0x0000_0065i32.to_le_bytes()); // stream_id
    f.extend_from_slice(&0x0000_1000i32.to_le_bytes()); // term_id
    f.extend_from_slice(&0i32.to_le_bytes()); // reserved_value
    f.extend_from_slice(&24u32.to_le_bytes()); // data_offset
    f.extend_from_slice(payload);
    f
}

/// A plausible Aeron control frame (NAK/SM/SETUP/RTT/ERRT) with the given type.
fn aeron_control_frame(frame_type: u16, payload: &[u8]) -> Vec<u8> {
    let mut f = Vec::new();
    let frame_len = (24 + payload.len()) as u16;
    f.extend_from_slice(&frame_len.to_le_bytes());
    f.push(0);
    f.push(0);
    f.extend_from_slice(&frame_type.to_le_bytes());
    f.extend_from_slice(&0i32.to_le_bytes()); // session_id
    f.extend_from_slice(&0i32.to_le_bytes()); // stream_id
    f.extend_from_slice(&0i32.to_le_bytes()); // term_id
    f.extend_from_slice(&0i32.to_le_bytes()); // reserved_value
    f.extend_from_slice(&24u32.to_le_bytes()); // data_offset
    f.extend_from_slice(payload);
    f
}

/// A valid IPv4/UDP frame with the given payload (real checksums so the
/// software-verify path in packet.c accepts it).
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
    f.extend_from_slice(&[0x00, 0x00]);
    f.extend_from_slice(&[0x40, 0x00]); // DF
    f.push(64);
    f.push(17);
    f.extend_from_slice(&[0x00, 0x00]);
    f.extend_from_slice(&src_ip);
    f.extend_from_slice(&dst_ip);
    // IPv4 header checksum over bytes 14..34.
    let mut sum: u32 = 0;
    for i in 0..10 {
        let w = ((f[14 + i * 2] as u16) << 8) | f[14 + i * 2 + 1] as u16;
        sum += w as u32;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    let ipc = !(sum as u16);
    f[24] = (ipc >> 8) as u8;
    f[25] = (ipc & 0xFF) as u8;
    f.extend_from_slice(&sport.to_be_bytes());
    f.extend_from_slice(&dport.to_be_bytes());
    let ulen = (8 + payload.len()) as u16;
    f.extend_from_slice(&ulen.to_be_bytes());
    f.extend_from_slice(&[0x00, 0x00]);
    let uc = udp_csum(src_ip, dst_ip, sport, dport, payload);
    let n = f.len();
    f[n - 2] = (uc >> 8) as u8;
    f[n - 1] = (uc & 0xFF) as u8;
    f.extend_from_slice(payload);
    f
}

fn udp_csum(src_ip: [u8; 4], dst_ip: [u8; 4], sport: u16, dport: u16, payload: &[u8]) -> u16 {
    let udp_len = 8 + payload.len();
    let mut words = Vec::new();
    for i in 0..2 {
        words.push(((src_ip[i * 2] as u16) << 8) | src_ip[i * 2 + 1] as u16);
        words.push(((dst_ip[i * 2] as u16) << 8) | dst_ip[i * 2 + 1] as u16);
    }
    words.push(17);
    words.push(udp_len as u16);
    words.push(sport);
    words.push(dport);
    words.push(udp_len as u16);
    words.push(0);
    for c in payload.chunks(2) {
        if c.len() == 2 {
            words.push(((c[0] as u16) << 8) | c[1] as u16);
        } else {
            words.push((c[0] as u16) << 8);
        }
    }
    let mut sum: u32 = 0;
    for w in words {
        sum += w as u32;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    let c = !(sum as u16);
    if c == 0 {
        0xFFFF
    } else {
        c
    }
}

// ---------------------------------------------------------------------------
// §8 interop semantics
// ---------------------------------------------------------------------------

/// A data frame emitted by the DPDK TX path arrives byte-identical at the DPDK
/// RX path: the TX capture holds an ordinary IPv4/UDP datagram, and injecting
/// that datagram's UDP payload into the receiver yields the exact same bytes.
#[test]
#[serial]
fn aeron_data_frame_roundtrip_preserves_bytes() {
    let mut h = Harness::new();
    h.init_tx(SENDER_IP, 40000, None, 1472);
    h.init_rx(RECEIVER_IP, 60000, 1472);
    const PEER: [u8; 4] = [10, 0, 0, 5];
    const PEER_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x50];
    h.arp_seed(PEER, PEER_MAC);

    let frame = aeron_data_frame(b"interop payload");
    let (rc, bytes_sent) = h.send_to((PEER, 40123), &[&frame]);
    assert_eq!(rc, 1);
    assert_eq!(bytes_sent, frame.len() as i64);

    // TX side: the emitted datagram carries the Aeron frame bytes unchanged.
    let caps = captures();
    assert_eq!(caps.len(), 1);
    let c = &caps[0];
    assert_eq!(udp_payload(c), frame.as_slice(), "TX path preserves payload bytes");
    assert_eq!(u32_ip_at(c, IP + 12), SENDER_IP, "source is the sender ENA IPv4");
    assert_eq!(u16_at(c, IP + 20 + 2), 40123, "destination UDP port");

    // RX side: wrap that same UDP payload in a frame addressed to the receiver
    // transport (with the sender's real identity as source) and dispatch it.
    let eth = build_udp_frame(RECEIVER_MAC, SENDER_MAC, SENDER_IP, RECEIVER_IP, 40000, 60000, &frame);
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, eth.as_ptr(), eth.len(), RX_GOOD, 1) },
        0
    );
    let mut p = h.poller();
    p.add(&mut *h.rx);
    let mut bytes = 0i64;
    let clientd = &mut h.rx_dgrams as *mut Vec<RxDatagram> as *mut c_void;
    assert_eq!(p.poll(16, clientd, &mut bytes), 1);
    assert_eq!(bytes, frame.len() as i64, "bytes_rcved = the full payload");
    assert_eq!(h.rx_dgrams.len(), 1);
    let d = &h.rx_dgrams[0];
    assert_eq!(d.data, frame, "RX path yields the exact bytes the TX path emitted");
    assert_eq!(d.src_ip, SENDER_IP, "the sender's IPv4 is reconstructed");
    assert_eq!(d.src_port, 40000, "the sender's UDP port is reconstructed");
    assert_eq!(
        d.transport, &mut *h.rx as *mut _ as usize,
        "dispatched to the rx transport"
    );
    assert_eq!(h.stats().accepted, 1);
    assert_no_leak();
}

/// Connected and unconnected transports coexist on one runtime: a connected
/// transport routes via its connected address, an unconnected one routes per
/// call, both through the sender ENA.
#[test]
#[serial]
fn connected_and_unconnected_transports_coexist() {
    let mut h = Harness::new();
    h.init_tx(SENDER_IP, 40000, Some(([10, 0, 0, 77], 5000)), 1472);
    h.init_rx(RECEIVER_IP, 60000, 1472);
    const CONN_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x77];
    const PEER_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x50];
    h.arp_seed([10, 0, 0, 77], CONN_MAC);
    h.arp_seed([10, 0, 0, 5], PEER_MAC);

    let (rc, sent) = h.send_connected(&[b"connected"]);
    assert_eq!((rc, sent), (1, 9));
    let (rc, sent) = h.send_to(([10, 0, 0, 5], 40123), &[b"unconnected"]);
    assert_eq!((rc, sent), (1, 11));

    let caps = captures();
    assert_eq!(caps.len(), 2, "one datagram per transport");
    assert_eq!(
        u32_ip_at(&caps[0], IP + 16),
        [10, 0, 0, 77],
        "connected -> connected address"
    );
    assert_eq!(u16_at(&caps[0], IP + 20 + 2), 5000);
    assert_eq!(
        u32_ip_at(&caps[1], IP + 16),
        [10, 0, 0, 5],
        "unconnected -> explicit destination"
    );
    assert_eq!(u16_at(&caps[1], IP + 20 + 2), 40123);
    assert_eq!(caps[0].port_id, 0, "both emit on the sender ENA");
    assert_no_leak();
}

/// bind_addr_and_port reports the effective local identity, i.e. exactly what
/// the wildcard port manager bound (the DPDK transport assigns no port of its
/// own).
#[test]
#[serial]
fn bind_addr_and_port_reports_effective_local_identity() {
    let mut h = Harness::new();
    h.init_tx(SENDER_IP, 40000, None, 1472);
    h.init_rx(RECEIVER_IP, 60000, 1472);
    assert_eq!(Harness::bind_addr_and_port(h.bindings, &mut h.tx), "10.0.0.1:40000");
    assert_eq!(Harness::bind_addr_and_port(h.bindings, &mut h.rx), "10.0.1.1:60000");
}

/// Reconnect replaces the copied remote address without retaining a caller-
/// owned pointer: the new address is passed from a temporary buffer that dies
/// before the next send, yet the connected send still reaches it.
#[test]
#[serial]
fn reconnect_replaces_copied_remote_address() {
    let mut h = Harness::new();
    h.init_tx(SENDER_IP, 40000, Some(([10, 0, 0, 77], 5000)), 1472);
    const A_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x77];
    const B_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x88];
    h.arp_seed([10, 0, 0, 77], A_MAC);
    h.arp_seed([10, 0, 0, 88], B_MAC);

    // Before reconnect: connected sends reach the original address.
    let (rc, _) = h.send_connected(&[b"to-a"]);
    assert_eq!(rc, 1);
    assert_eq!(u32_ip_at(&captures()[0], IP + 16), [10, 0, 0, 77]);

    // Reconnect to B from a temporary that is dropped here — the native side
    // must have copied the address into its own client state.
    h.reconnect(([10, 0, 0, 88], 6000));

    let (rc, _) = h.send_connected(&[b"to-b"]);
    assert_eq!(rc, 1);
    let caps = captures();
    assert_eq!(caps.len(), 2);
    assert_eq!(
        u32_ip_at(&caps[1], IP + 16),
        [10, 0, 0, 88],
        "connected send follows the new address"
    );
    assert_eq!(u16_at(&caps[1], IP + 20 + 2), 6000);
    assert_no_leak();
}

/// Interceptors are usable through `aeron_udp_channel_data_paths_t`: the
/// poller dispatches with the registered transport's data_paths pointer, so the
/// incoming-interceptor chain reachable from the callback gets the channel's
/// data path exactly as the kernel socket path would.
#[test]
#[serial]
fn data_paths_forwarded_to_recv_callback() {
    let mut h = Harness::new();
    h.init_rx(RECEIVER_IP, 60000, 1472);
    let dp = Box::new(unsafe { std::mem::zeroed::<aeron_udp_channel_data_paths_stct>() });
    h.rx.data_paths = &*dp as *const _ as *mut _;

    let frame = build_udp_frame(
        RECEIVER_MAC,
        SENDER_MAC,
        SENDER_IP,
        RECEIVER_IP,
        40000,
        60000,
        b"interceptor",
    );
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, frame.as_ptr(), frame.len(), RX_GOOD, 1) },
        0
    );
    let mut p = h.poller();
    p.add(&mut *h.rx);
    let mut bytes = 0i64;
    let clientd = &mut h.rx_dgrams as *mut Vec<RxDatagram> as *mut c_void;
    assert_eq!(p.poll(16, clientd, &mut bytes), 1);
    assert_eq!(h.rx_dgrams.len(), 1);
    assert_eq!(
        h.rx_dgrams[0].data_paths, &*dp as *const _ as usize,
        "recv_cb received the transport's data_paths"
    );
    assert_no_leak();
}

/// NAK/status/setup/RTT/data/retransmit role frames are opaque UDP payloads
/// that dispatch to the correct endpoint: two receiver transports on the shared
/// receiver ENA behave like a manual MDC control endpoint plus a data endpoint,
/// and removing one stops its dispatch (dynamic MDC close) without disturbing
/// the other.
#[test]
#[serial]
fn role_frames_dispatch_by_endpoint_and_removal_stops() {
    let mut h = Harness::new();
    h.init_rx(RECEIVER_IP, 60000, 1472);
    let mut control = init_extra_transport(&mut h, 60001);
    let mut p = h.poller();
    p.add(&mut *h.rx);
    p.add(&mut *control);

    // (dport, frame type, label) — every Aeron control-plane and data-plane
    // role Aeron produces, delivered as UDP payloads.
    let roles: &[(u16, u16, &[u8])] = &[
        (60000, 0x03, b"nak"),        // NAK (retransmission request)
        (60001, 0x04, b"sm"),         // status message
        (60000, 0x06, b"setup"),      // stream setup
        (60001, 0x07, b"rtt"),        // RTT measurement
        (60000, 0x01, b"data"),       // data frame
        (60001, 0x05, b"retransmit"), // ERRT retransmit
    ];
    for (dport, ty, label) in roles {
        let payload = aeron_control_frame(*ty, label);
        let eth = build_udp_frame(
            RECEIVER_MAC,
            SENDER_MAC,
            SENDER_IP,
            RECEIVER_IP,
            40000,
            *dport,
            &payload,
        );
        assert_eq!(
            unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, eth.as_ptr(), eth.len(), RX_GOOD, 1) },
            0
        );
    }

    let mut bytes = 0i64;
    let clientd = &mut h.rx_dgrams as *mut Vec<RxDatagram> as *mut c_void;
    assert_eq!(p.poll(16, clientd, &mut bytes), 6, "all six role frames dispatched");
    assert_eq!(h.rx_dgrams.len(), 6);
    let primary = &mut *h.rx as *mut _ as usize;
    let control_ptr = &mut *control as *mut _ as usize;
    for (dport, ty, label) in roles {
        let want_frame = aeron_control_frame(*ty, label);
        let d = h.rx_dgrams.iter().find(|d| d.data == want_frame).unwrap();
        let want = if *dport == 60000 { primary } else { control_ptr };
        assert_eq!(
            d.transport, want,
            "role frame on port {dport} dispatched to the right endpoint"
        );
    }
    assert_eq!(h.stats().accepted, 6);

    // Dynamic MDC close: removing the control endpoint stops its dispatch; the
    // data endpoint is unaffected.
    p.remove(&mut *control);
    let payload = aeron_control_frame(0x07, b"rtt");
    let eth = build_udp_frame(RECEIVER_MAC, SENDER_MAC, SENDER_IP, RECEIVER_IP, 40000, 60001, &payload);
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, eth.as_ptr(), eth.len(), RX_GOOD, 1) },
        0
    );
    assert_eq!(p.poll(16, clientd, &mut bytes), 0, "no dispatch after removal");
    assert_eq!(h.rx_dgrams.len(), 6);
    let s = h.stats();
    assert_eq!(s.accepted, 6);
    assert_eq!(s.unknown_port, 1, "the removed endpoint is an unknown port");

    let payload = aeron_data_frame(b"still alive");
    let eth = build_udp_frame(RECEIVER_MAC, SENDER_MAC, SENDER_IP, RECEIVER_IP, 40000, 60000, &payload);
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, eth.as_ptr(), eth.len(), RX_GOOD, 1) },
        0
    );
    assert_eq!(
        p.poll(16, clientd, &mut bytes),
        1,
        "the remaining endpoint still dispatches"
    );
    assert_no_leak();
    close_transport(h.bindings, &mut control);
}

/// Messages exceeding one Aeron frame use Aeron fragmentation at the transport
/// boundary: every emitted datagram stays within `max_aeron_mtu`, DF is always
/// set (so a router never fragments), and an oversized iovec is rejected.
#[test]
#[serial]
fn mtu_bound_never_fragments() {
    let mut h = Harness::new();
    // A 32-byte channel MTU: 32 fits, 33 is oversized.
    h.init_tx(SENDER_IP, 40000, None, 32);
    const PEER: [u8; 4] = [10, 0, 0, 5];
    const PEER_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x50];
    h.arp_seed(PEER, PEER_MAC);

    // Aeron fragments a large message into per-frame iovecs; each iovec is one
    // datagram within the MTU.
    let (rc, sent) = h.send_to((PEER, 40123), &[&[1u8; 32], &[2u8; 16], &[3u8; 24]]);
    assert_eq!(rc, 3);
    assert_eq!(sent, 32 + 16 + 24);
    let caps = captures();
    assert_eq!(caps.len(), 3);
    for c in &caps {
        assert_eq!(
            udp_payload(c).len(),
            c.len as usize - PAYLOAD,
            "each datagram is its own iovec"
        );
        assert!(udp_payload(c).len() <= 32, "no datagram exceeds max_aeron_mtu");
        assert_eq!(ip_flags_frag(c), 0x4000, "DF set: never emit IP fragments");
    }

    // A single iovec beyond the MTU is rejected outright.
    let (rc, sent) = h.send_to((PEER, 40123), &[&[4u8; 33]]);
    assert_eq!(rc, -1);
    assert_eq!(sent, 0);
    assert!(last_error().contains("oversized"), "got: {}", last_error());
    assert_no_leak();
}
