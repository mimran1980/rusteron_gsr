//! Aeron DPDK counters and ENA statistics (Ticket 7, plan §9).
//!
//! Registers the per-role-port counter sets into a real
//! `aeron_counters_manager_t` (aligned metadata + values buffers, real clock
//! cache) reached through a fake driver context, then drives the genuine
//! binding callbacks (send on the sender ENA, poll on the receiver ENA) and
//! asserts the counters read back from the manager buffers:
//!   * every fixed counter exists with its plan §9 type ID and a label carrying
//!     role / BDF / DPDK port / queue / direction (port-info also driver + MAC),
//!   * TX packet/byte totals are monotonic and attribute exactly the accepted
//!     prefix (plan §7.4: rejected datagrams are errors, not TX bytes),
//!   * every approved TX error path bumps its counter (nobufs, tx-eagain, ARP
//!     miss, validation error incl. oversized), and every RX reject class bumps
//!     its own counter (checksum, fragmented, unsupported ethertype/protocol,
//!     discard, queue-drop),
//!   * ENA SRD/Express extended statistics appear as type-92 counters with the
//!     PMD's names, are re-read at most once per second (no metrics thread),
//!     and a *missed* xstat mirrors the plan §9 type-84 counter.
//!
//! Linux x86_64 only; links the same archives as the other DPDK integration
//! tests plus the counters manager/clock compiled into the core archive.
#![cfg(all(feature = "dpdk", target_os = "linux", target_arch = "x86_64"))]

mod common;
use common::*;

use serial_test::serial;

use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// ---------------------------------------------------------------------------
// Counter layout (AERON_COUNTERS_MANAGER_* descriptor sizes).
// ---------------------------------------------------------------------------

const VALUE_LENGTH: usize = 128;
const METADATA_LENGTH: usize = 512;
const COUNTER_SLOTS: usize = 128;

const STATE_OFFSET: usize = 0;
const TYPE_ID_OFFSET: usize = 4;
const LABEL_LENGTH_OFFSET: usize = 128;
const LABEL_OFFSET: usize = 132;

const RECORD_ALLOCATED: i32 = 1;

// Type IDs (must match native/dpdk/rusteron_dpdk_counters.h).
const TYPE_PORT_INFO: i32 = 75;
const TYPE_TRANSPORT: i32 = 76;
const TYPE_NOBUFS: i32 = 77;
const TYPE_TX_EAGAIN: i32 = 78;
const TYPE_ERROR: i32 = 79;
const TYPE_PKTS: i32 = 82;
const TYPE_BYTES: i32 = 83;
const TYPE_MISSED_PACKETS: i32 = 84;
const TYPE_ARP_MISS: i32 = 85;
const TYPE_POLLER: i32 = 87;
const TYPE_QUEUE_DROP: i32 = 88;
const TYPE_CHECKSUM_FAILURE: i32 = 89;
const TYPE_FRAGMENTED: i32 = 90;
const TYPE_MEMPOOL_AVAILABLE: i32 = 91;
const TYPE_EXTENDED_STATS: i32 = 92;
const TYPE_UNSUPPORTED_ETHERTYPE: i32 = 93;
const TYPE_UNSUPPORTED_PROTOCOL: i32 = 94;
const TYPE_RX_RECEIVER_DISCARD: i32 = 95;

// ---------------------------------------------------------------------------
// Native test hooks (fakes + runtime).
// ---------------------------------------------------------------------------

extern "C" {
    fn rusteron_dpdk_transport_bindings() -> *mut aeron_udp_channel_transport_bindings_stct;
    fn rusteron_dpdk_fake_set_xstats(names: *const *const c_char, values: *const u64, count: u32);
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

// ---------------------------------------------------------------------------
// Receive callback (clientd carries the Vec; the poller forwards data_paths).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct RxDatagram {
    data: Vec<u8>,
}

unsafe extern "C" fn recv_cb(
    _data_paths: *mut aeron_udp_channel_data_paths_t,
    _transport: *mut aeron_udp_channel_transport_t,
    receiver_clientd: *mut c_void,
    _endpoint_clientd: *mut c_void,
    _destination_clientd: *mut c_void,
    buffer: *mut u8,
    length: usize,
    _addr: *mut sockaddr_storage,
    _media_timestamp: *mut timespec,
) {
    let rx = &mut *(receiver_clientd as *mut Vec<RxDatagram>);
    let data = std::slice::from_raw_parts(buffer, length).to_vec();
    rx.push(RxDatagram { data });
}

// ---------------------------------------------------------------------------
// Counters manager: real buffers so the native allocate/free paths run against
// real memory (and the free path's cached-clock deref has a real target).
// ---------------------------------------------------------------------------

struct CountersManager {
    manager: Box<aeron_counters_manager_stct>,
    clock: Box<aeron_clock_cache_stct>,
    metadata: Box<[u64]>,
    values: Box<[u64]>,
}

impl CountersManager {
    fn new() -> Self {
        let metadata: Box<[u64]> = vec![0u64; COUNTER_SLOTS * METADATA_LENGTH / 8].into_boxed_slice();
        let values: Box<[u64]> = vec![0u64; COUNTER_SLOTS * VALUE_LENGTH / 8].into_boxed_slice();

        let mut clock = Box::new(aeron_clock_cache_stct {
            pre_pad: [0; 56],
            cached_epoch_time: 1_000,
            cached_nano_time: 0,
            post_pad: [0; 56],
        });

        let mut manager = Box::new(unsafe { std::mem::zeroed::<aeron_counters_manager_stct>() });
        let rc = unsafe {
            aeron_counters_manager_init(
                &mut *manager,
                metadata.as_ptr() as *mut u8,
                metadata.len() * 8,
                values.as_ptr() as *mut u8,
                values.len() * 8,
                &mut *clock,
                10_000, // free_to_reuse_timeout_ms
            )
        };
        assert_eq!(rc, 0, "counters manager init failed");

        CountersManager {
            manager,
            clock,
            metadata,
            values,
        }
    }

    /// Read a live counter slot by id (value, type_id, label).
    fn read(&self, id: i32) -> (i64, i32, String) {
        let meta = unsafe { std::slice::from_raw_parts(self.metadata.as_ptr() as *const u8, self.metadata.len() * 8) };
        let vals = unsafe { std::slice::from_raw_parts(self.values.as_ptr() as *const u8, self.values.len() * 8) };
        let mb = id as usize * METADATA_LENGTH;
        let vb = id as usize * VALUE_LENGTH;
        let value = i64::from_le_bytes(vals[vb..vb + 8].try_into().unwrap());
        let type_id = i32::from_le_bytes(meta[mb + TYPE_ID_OFFSET..mb + TYPE_ID_OFFSET + 4].try_into().unwrap());
        let len = i32::from_le_bytes(
            meta[mb + LABEL_LENGTH_OFFSET..mb + LABEL_LENGTH_OFFSET + 4]
                .try_into()
                .unwrap(),
        );
        let label = if len > 0 {
            String::from_utf8_lossy(&meta[mb + LABEL_OFFSET..mb + LABEL_OFFSET + len as usize]).into_owned()
        } else {
            String::new()
        };
        (value, type_id, label)
    }

    /// Find a live allocated counter by type ID and label substring.
    fn find(&self, type_id: i32, label_needle: &str) -> Option<(i64, i32, String)> {
        let meta = unsafe { std::slice::from_raw_parts(self.metadata.as_ptr() as *const u8, self.metadata.len() * 8) };
        for id in 0..COUNTER_SLOTS {
            let mb = id * METADATA_LENGTH;
            let state = i32::from_le_bytes(meta[mb + STATE_OFFSET..mb + STATE_OFFSET + 4].try_into().unwrap());
            if state != RECORD_ALLOCATED {
                continue;
            }
            let (value, t, label) = self.read(id as i32);
            if t == type_id && label.contains(label_needle) {
                return Some((value, t, label));
            }
        }
        None
    }
}

impl Drop for CountersManager {
    fn drop(&mut self) {
        unsafe { aeron_counters_manager_close(&mut *self.manager) };
    }
}

// ---------------------------------------------------------------------------
// Harness: one native runtime (both ENA ports) + a fake driver context whose
// counters_manager owns the counters the transport registers into.
// ---------------------------------------------------------------------------

struct Harness {
    _env: TestEnv,
    native: *mut c_void,
    bindings: *mut aeron_udp_channel_transport_bindings_stct,
    counters: CountersManager,
    context: Box<aeron_driver_context_stct>,
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

        let mut counters = CountersManager::new();
        let mut context = Box::new(unsafe { std::mem::zeroed::<aeron_driver_context_stct>() });
        context.counters_manager = &mut *counters.manager;

        Harness {
            _env: env,
            native,
            bindings,
            counters,
            context,
            tx: Box::new(unsafe { std::mem::zeroed::<aeron_udp_channel_transport_stct>() }),
            rx: Box::new(unsafe { std::mem::zeroed::<aeron_udp_channel_transport_stct>() }),
            data_paths: Box::new(unsafe { std::mem::zeroed::<aeron_udp_channel_data_paths_stct>() }),
            rx_dgrams: Vec::new(),
            tx_inited: false,
            rx_inited: false,
        }
    }

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
                &mut *self.context,
                aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER,
            )
        };
        assert_eq!(rc, 0, "tx init failed: {}", last_error());
    }

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
                &mut *self.context,
                aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER,
            )
        };
        assert_eq!(rc, 0, "rx init failed: {}", last_error());
    }

    fn send_to(&mut self, dst: ([u8; 4], u16), payloads: &[&[u8]]) -> (c_int, i64) {
        let mut d = SockAddrIn::new(dst.0, dst.1);
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
                d.storage_mut(),
                iovs.as_ptr() as *mut iovec,
                iovs.len(),
                &mut bytes_sent,
            )
        };
        (rc, bytes_sent)
    }

    /// Send with a NULL address on the unconnected transport: the "no
    /// destination" error path.
    fn send_null_addr(&mut self, payloads: &[&[u8]]) -> (c_int, i64) {
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
                ptr::null_mut(),
                iovs.as_ptr() as *mut iovec,
                iovs.len(),
                &mut bytes_sent,
            )
        };
        (rc, bytes_sent)
    }

    fn arp_seed(&self, ip: [u8; 4], mac: [u8; 6]) {
        let ip_str = std::ffi::CString::new(format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])).unwrap();
        let rc = unsafe { rusteron_dpdk_transport_test_arp_seed(self.native, ip_str.as_ptr(), mac.as_ptr()) };
        assert_eq!(rc, 0, "arp_seed failed: {}", last_error());
    }

    fn poller(&self) -> Poller {
        Poller::new(self.bindings)
    }

    fn counter(&self, type_id: i32, needle: &str) -> i64 {
        self.counters
            .find(type_id, needle)
            .unwrap_or_else(|| panic!("no counter type={type_id} label~{needle}"))
            .0
    }

    fn counter_label(&self, type_id: i32, needle: &str) -> String {
        self.counters
            .find(type_id, needle)
            .unwrap_or_else(|| panic!("no counter type={type_id} label~{needle}"))
            .2
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

// ---------------------------------------------------------------------------
// Frame helpers (real checksums so rx_ol_flags = 0 exercises software verify).
// ---------------------------------------------------------------------------

const RECEIVER_PORT: u16 = 1;
const RECEIVER_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x02];
const RECEIVER_IP: [u8; 4] = [10, 0, 1, 1];
const SENDER_IP: [u8; 4] = [10, 0, 0, 1];
const PEER_IP: [u8; 4] = [10, 0, 0, 5];
const PEER_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x50];

const RX_GOOD: u32 = 0b11; // IPV4_CKSUM_GOOD | UDP_CKSUM_GOOD

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
    let hdr = f[14..34].to_vec();
    let mut words = Vec::new();
    for i in 0..10 {
        words.push(((hdr[i * 2] as u16) << 8) | hdr[i * 2 + 1] as u16);
    }
    let ipc = !(ones_sum(&words) as u16);
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

/// A frame addressed to the receiver transport on the given dport.
fn frame_to_receiver(dport: u16, payload: &[u8]) -> Vec<u8> {
    build_udp_frame(RECEIVER_MAC, PEER_MAC, PEER_IP, RECEIVER_IP, 7000, dport, payload)
}

// ---------------------------------------------------------------------------
// Registration and labels
// ---------------------------------------------------------------------------

/// Both role ports register their counter sets with the plan §9 type IDs and
/// labels carrying role / BDF / DPDK port / queue / direction; port-info also
/// carries the PMD driver name and MAC; transport counts the open transports.
#[test]
#[serial]
fn counters_registered_with_type_ids_and_labels() {
    let mut h = Harness::new();
    h.init_tx(SENDER_IP, 40000, None, 1472);
    h.init_rx(RECEIVER_IP, 60000, 1472);

    // Sender port-info: type 75, value 1, driver + MAC in the label.
    let (v, t, label) = h
        .counters
        .find(TYPE_PORT_INFO, "role=sender")
        .expect("sender port-info");
    assert_eq!(t, TYPE_PORT_INFO);
    assert_eq!(v, 1);
    assert!(label.contains("rusteron-dpdk role=sender bdf=0000:00:01.0 port=0 q=0 dir=- port-info"));
    assert!(
        label.contains(" driver=net_ena mac=02:00:00:00:00:01"),
        "label: {label}"
    );

    // Receiver port-info: distinct BDF/port/MAC.
    let (v, t, label) = h
        .counters
        .find(TYPE_PORT_INFO, "role=receiver")
        .expect("receiver port-info");
    assert_eq!(v, 1);
    assert!(label.contains("role=receiver bdf=0000:00:02.0 port=1"));
    assert!(label.contains("mac=02:00:00:00:00:02"), "label: {label}");

    // Each port carries the full counter set for both directions, so the
    // needle must pin role AND direction to disambiguate the match.
    let (_, _, label) = h
        .counters
        .find(TYPE_PKTS, "role=sender bdf=0000:00:01.0 port=0 q=0 dir=tx pkts")
        .expect("sender tx pkts");
    let (_, _, label) = h
        .counters
        .find(TYPE_BYTES, "role=sender bdf=0000:00:01.0 port=0 q=0 dir=tx bytes")
        .expect("sender tx bytes");

    let (_, _, label) = h
        .counters
        .find(TYPE_PKTS, "role=receiver bdf=0000:00:02.0 port=1 q=0 dir=rx pkts")
        .expect("receiver rx pkts");
    let (_, _, label) = h
        .counters
        .find(TYPE_BYTES, "role=receiver bdf=0000:00:02.0 port=1 q=0 dir=rx bytes")
        .expect("receiver rx bytes");

    // Transport counters: one open transport per role port.
    assert_eq!(h.counter(TYPE_TRANSPORT, "role=sender"), 1);
    assert_eq!(h.counter(TYPE_TRANSPORT, "role=receiver"), 1);

    // Mempool availability is populated at registration (fake pool holds 64).
    assert_eq!(h.counter(TYPE_MEMPOOL_AVAILABLE, "role=sender"), 64);
    assert_eq!(h.counter(TYPE_MEMPOOL_AVAILABLE, "role=receiver"), 64);

    assert_no_leak();
}

// ---------------------------------------------------------------------------
// TX counters: monotonic and correctly attributed
// ---------------------------------------------------------------------------

/// TX packet/byte totals are monotonic and attribute exactly the accepted
/// prefix — rejected datagrams (oversized) are errors, never TX bytes.
#[test]
#[serial]
fn tx_pkts_and_bytes_monotonic_and_attributed() {
    let mut h = Harness::new();
    h.init_tx(SENDER_IP, 40000, None, 1472);
    h.arp_seed(PEER_IP, PEER_MAC);

    assert_eq!(h.counter(TYPE_PKTS, "dir=tx"), 0);
    assert_eq!(h.counter(TYPE_BYTES, "dir=tx"), 0);

    let (rc, sent) = h.send_to((PEER_IP, 40123), &[b"hello", b"world!"]);
    assert_eq!((rc, sent), (2, 11));
    assert_eq!(h.counter(TYPE_PKTS, "dir=tx"), 2, "two datagrams");
    assert_eq!(h.counter(TYPE_BYTES, "dir=tx"), 11, "5 + 6 payload bytes");

    let (rc, sent) = h.send_to((PEER_IP, 40123), &[b"xyz"]);
    assert_eq!((rc, sent), (1, 3));
    assert_eq!(h.counter(TYPE_PKTS, "dir=tx"), 3, "monotonic");
    assert_eq!(h.counter(TYPE_BYTES, "dir=tx"), 14, "monotonic");
    assert_no_leak();
}

/// Oversized datagrams: the accepted prefix is still counted (pkts/bytes) and
/// the oversized datagram bumps the error counter.
#[test]
#[serial]
fn oversized_datagram_bumps_error_not_tx_bytes() {
    let mut h = Harness::new();
    h.init_tx(SENDER_IP, 40000, None, 32);
    h.arp_seed(PEER_IP, PEER_MAC);

    // A 32-byte channel MTU: the first iovec fits, the second (33 bytes) is
    // oversized -> prefix flushed, then a permanent validation error.
    let (rc, sent) = h.send_to((PEER_IP, 40123), &[&[1u8; 20], &[2u8; 33]]);
    assert_eq!(rc, -1, "oversized datagram is rejected");
    assert_eq!(sent, 20, "the accepted prefix is reported");
    assert!(last_error().contains("oversized"), "got: {}", last_error());
    assert_eq!(h.counter(TYPE_PKTS, "dir=tx"), 1, "only the prefix datagram");
    assert_eq!(h.counter(TYPE_BYTES, "dir=tx"), 20, "only prefix bytes");
    assert_eq!(
        h.counter(TYPE_ERROR, "role=sender"),
        1,
        "oversized bumps the error counter"
    );
    assert_no_leak();
}

/// Every TX error path bumps its own counter: no destination, non-IPv4,
/// multicast, unresolved ARP, mbuf exhaustion, and NIC backpressure.
#[test]
#[serial]
fn tx_error_paths_counted() {
    let mut h = Harness::new();
    h.init_tx(SENDER_IP, 40000, None, 1472);
    h.arp_seed(PEER_IP, PEER_MAC);

    // No destination address (unconnected transport, NULL addr).
    let (rc, sent) = h.send_null_addr(&[b"x"]);
    assert_eq!(rc, -1);
    assert_eq!(sent, 0);
    assert_eq!(h.counter(TYPE_ERROR, "role=sender"), 1, "no-destination error");

    // Non-IPv4 family: a sockaddr_in6 as the destination.
    let (rc, sent) = {
        let mut v6 = [0u8; 128];
        v6[0] = 10; // AF_INET6
        let iovs = vec![iovec {
            iov_base: b"x".as_ptr() as *mut c_void,
            iov_len: 1,
        }];
        let mut bytes_sent: i64 = 0;
        let rc = unsafe {
            let b = &*h.bindings;
            b.send_func.unwrap()(
                &mut *h.data_paths,
                &mut *h.tx,
                v6.as_mut_ptr() as *mut sockaddr_storage,
                iovs.as_ptr() as *mut iovec,
                iovs.len(),
                &mut bytes_sent,
            )
        };
        (rc, bytes_sent)
    };
    assert_eq!(rc, -1);
    assert_eq!(sent, 0);
    assert_eq!(h.counter(TYPE_ERROR, "role=sender"), 2, "non-IPv4 error");

    // Multicast destination.
    let (rc, sent) = h.send_to(([224, 0, 0, 1], 40123), &[b"x"]);
    assert_eq!(rc, -1);
    assert_eq!(sent, 0);
    assert_eq!(h.counter(TYPE_ERROR, "role=sender"), 3, "multicast error");

    // Unresolved ARP: retryable zero, ARP-miss counter bumps.
    let (rc, sent) = h.send_to(([10, 0, 0, 99], 40123), &[b"x"]);
    assert_eq!((rc, sent), (0, 0), "unresolved ARP is a retryable zero");
    assert_eq!(h.counter(TYPE_ARP_MISS, "role=sender"), 1, "ARP miss counted");

    // Mbuf exhaustion: with no mbufs available, the build fails immediately.
    unsafe { rusteron_dpdk_fake_set_pool_avail(0) };
    let (rc, sent) = h.send_to((PEER_IP, 40123), &[b"x"]);
    assert_eq!((rc, sent), (0, 0));
    assert_eq!(h.counter(TYPE_NOBUFS, "role=sender"), 1, "nobufs counted");
    unsafe { rusteron_dpdk_fake_set_pool_avail(64) };

    // NIC backpressure: the fake accepts nothing in the flush.
    unsafe { rusteron_dpdk_fake_set_tx_burst_cap(0) };
    let (rc, sent) = h.send_to((PEER_IP, 40123), &[b"x"]);
    assert_eq!((rc, sent), (0, 0), "backpressure stops the burst");
    assert_eq!(h.counter(TYPE_TX_EAGAIN, "role=sender"), 1, "tx-eagain counted");
    unsafe { rusteron_dpdk_fake_set_tx_burst_cap(64) };
    assert_no_leak();
}

// ---------------------------------------------------------------------------
// RX counters: per-reject-class attribution
// ---------------------------------------------------------------------------

/// A helper that injects a frame, polls, and returns the poll work count.
fn inject_and_poll(h: &mut Harness, p: &mut Poller, frame: &[u8]) -> c_int {
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, frame.as_ptr(), frame.len(), RX_GOOD, 1) },
        0
    );
    let mut bytes = 0i64;
    let clientd = &mut h.rx_dgrams as *mut Vec<RxDatagram> as *mut c_void;
    p.poll(16, clientd, &mut bytes)
}

/// Accepted frames bump RX pkts/bytes and the poller counter; rejected classes
/// bump exactly their own counters (plan §9).
#[test]
#[serial]
fn rx_pkts_bytes_and_reject_classes_counted() {
    let mut h = Harness::new();
    h.init_rx(RECEIVER_IP, 60000, 1472);
    let mut p = h.poller();
    p.add(&mut *h.rx);

    // Accepted: RX pkts/bytes + poller.
    assert_eq!(inject_and_poll(&mut h, &mut p, &frame_to_receiver(60000, b"accept")), 1);
    assert_eq!(h.rx_dgrams.len(), 1);
    assert_eq!(h.counter(TYPE_PKTS, "dir=rx"), 1);
    assert_eq!(h.counter(TYPE_BYTES, "dir=rx"), 6, "payload length, not the frame");
    assert_eq!(h.counter(TYPE_POLLER, "dir=rx"), 1, "one frame processed per poll");

    // IPv6 ethertype -> discard.
    let mut frame = frame_to_receiver(60000, b"x");
    frame[12] = 0x86;
    frame[13] = 0xDD;
    assert_eq!(inject_and_poll(&mut h, &mut p, &frame), 0);
    assert_eq!(h.counter(TYPE_RX_RECEIVER_DISCARD, "role=receiver"), 1);

    // VLAN -> discard.
    let mut frame = frame_to_receiver(60000, b"x");
    frame[12] = 0x81;
    frame[13] = 0x00;
    assert_eq!(inject_and_poll(&mut h, &mut p, &frame), 0);
    assert_eq!(h.counter(TYPE_RX_RECEIVER_DISCARD, "role=receiver"), 2);

    // Unknown ethertype -> unsupported-ethertype.
    let mut frame = frame_to_receiver(60000, b"x");
    frame[12] = 0x12;
    frame[13] = 0x34;
    assert_eq!(inject_and_poll(&mut h, &mut p, &frame), 0);
    assert_eq!(h.counter(TYPE_UNSUPPORTED_ETHERTYPE, "role=receiver"), 1);

    // IP fragment -> fragmented.
    let mut frame = frame_to_receiver(60000, b"x");
    frame[20] = 0x20; // MF set
    assert_eq!(inject_and_poll(&mut h, &mut p, &frame), 0);
    assert_eq!(h.counter(TYPE_FRAGMENTED, "role=receiver"), 1);

    // Non-UDP protocol -> unsupported-protocol.
    let mut frame = frame_to_receiver(60000, b"x");
    frame[23] = 6; // TCP
    assert_eq!(inject_and_poll(&mut h, &mut p, &frame), 0);
    assert_eq!(h.counter(TYPE_UNSUPPORTED_PROTOCOL, "role=receiver"), 1);

    // Bad checksum (NIC verdict) -> checksum-failure.
    let frame = frame_to_receiver(60000, b"x");
    assert_eq!(
        unsafe { rusteron_dpdk_fake_rx_inject(RECEIVER_PORT, frame.as_ptr(), frame.len(), 0b100, 1) },
        0
    );
    let mut bytes = 0i64;
    let clientd = &mut h.rx_dgrams as *mut Vec<RxDatagram> as *mut c_void;
    p.poll(16, clientd, &mut bytes);
    assert_eq!(h.counter(TYPE_CHECKSUM_FAILURE, "role=receiver"), 1);

    // Foreign destination / unregistered port -> discard; queue-drop for the
    // endpoint-map miss only.
    let foreign = build_udp_frame(RECEIVER_MAC, PEER_MAC, PEER_IP, [10, 0, 1, 99], 7000, 60000, b"f");
    assert_eq!(inject_and_poll(&mut h, &mut p, &foreign), 0);
    let unknown = frame_to_receiver(60099, b"u");
    assert_eq!(inject_and_poll(&mut h, &mut p, &unknown), 0);
    let discard = h.counter(TYPE_RX_RECEIVER_DISCARD, "role=receiver");
    assert_eq!(discard, 4, "ipv6+vlan+foreign+unknown all discarded");
    assert_eq!(
        h.counter(TYPE_QUEUE_DROP, "role=receiver"),
        1,
        "only the unknown-port misses the map"
    );

    // Accepted is unaffected by the rejects.
    assert_eq!(h.counter(TYPE_PKTS, "dir=rx"), 1);
    assert_eq!(h.counter(TYPE_BYTES, "dir=rx"), 6);
    assert_no_leak();
}

// ---------------------------------------------------------------------------
// ENA extended statistics (plan §9: type 92 mirrors, 1 Hz sample)
// ---------------------------------------------------------------------------

/// The fake PMD's xstat table: an ENA SRD-style counter plus a *missed* entry
/// (the ENA PMD exposes rx_missed variants), so both the type-92 mirrors and
/// the plan §9 type-84 missed-packets mirror are exercised.
fn install_xstats(names: &[&str], values: &[u64]) {
    assert_eq!(names.len(), values.len());
    let cnames: Vec<std::ffi::CString> = names.iter().map(|n| std::ffi::CString::new(*n).unwrap()).collect();
    let ptrs: Vec<*const c_char> = cnames.iter().map(|c| c.as_ptr()).collect();
    unsafe { rusteron_dpdk_fake_set_xstats(ptrs.as_ptr(), values.as_ptr(), names.len() as u32) };
}

/// ENA extended statistics appear as type-92 counters named by the PMD, are
/// re-read at most once per second (the sample is driven from the poller hot
/// path, not a metrics thread), and a *missed* xstat mirrors type 84.
#[test]
#[serial]
fn ena_xstats_mirrored_and_sampled_at_1hz() {
    // Construct the harness first: its TestEnv::new() resets the fakes, which
    // would wipe an earlier install_xstats. Install the PMD xstat table and pin
    // the clock before registration so the 1 Hz gate is deterministic.
    let mut h = Harness::new();
    install_xstats(
        &["rx_missed", "ena_srd_llq_rx_packets", "ena_express_rx_drops"],
        &[11, 2200, 3],
    );
    unsafe { rusteron_dpdk_test_set_clock_ms(1_000) };
    h.init_rx(RECEIVER_IP, 60000, 1472);
    let mut p = h.poller();
    p.add(&mut *h.rx);

    // The three PMD xstats are mirrored as type-92 counters with their names.
    let (v, t, label) = h
        .counters
        .find(TYPE_EXTENDED_STATS, "rx_missed")
        .expect("rx_missed mirror");
    assert_eq!(t, TYPE_EXTENDED_STATS);
    assert_eq!(v, 11);
    assert!(label.starts_with("rx_missed"), "label is the PMD name: {label}");
    assert_eq!(h.counter(TYPE_EXTENDED_STATS, "ena_srd_llq_rx_packets"), 2200);
    assert_eq!(h.counter(TYPE_EXTENDED_STATS, "ena_express_rx_drops"), 3);

    // The *missed* mirror (type 84) is populated by the sample, not by
    // registration — it reads 0 until the first 1 Hz refresh.
    assert_eq!(
        h.counter(TYPE_MISSED_PACKETS, "role=receiver"),
        0,
        "mirror waits for the sample"
    );

    // Change the PMD values; within the 1 Hz window nothing is re-read.
    install_xstats(
        &["rx_missed", "ena_srd_llq_rx_packets", "ena_express_rx_drops"],
        &[99, 4400, 7],
    );
    let mut bytes = 0i64;
    let clientd = &mut h.rx_dgrams as *mut Vec<RxDatagram> as *mut c_void;
    assert_eq!(p.poll(16, clientd, &mut bytes), 0, "no traffic; sample gate governs");
    assert_eq!(h.counter(TYPE_EXTENDED_STATS, "rx_missed"), 11, "within 1 Hz: stale");
    assert_eq!(h.counter(TYPE_MISSED_PACKETS, "role=receiver"), 0, "within 1 Hz: stale");

    // Advance a full second: the next poll re-reads the PMD xstats.
    unsafe { rusteron_dpdk_test_set_clock_ms(2_000) };
    assert_eq!(p.poll(16, clientd, &mut bytes), 0);
    assert_eq!(h.counter(TYPE_EXTENDED_STATS, "rx_missed"), 99, "1 Hz sample re-reads");
    assert_eq!(h.counter(TYPE_EXTENDED_STATS, "ena_srd_llq_rx_packets"), 4400);
    assert_eq!(h.counter(TYPE_EXTENDED_STATS, "ena_express_rx_drops"), 7);
    assert_eq!(
        h.counter(TYPE_MISSED_PACKETS, "role=receiver"),
        99,
        "type-84 mirror follows"
    );

    // Mempool availability refreshes on the sample too.
    assert_eq!(h.counter(TYPE_MEMPOOL_AVAILABLE, "role=receiver"), 64);

    unsafe { rusteron_dpdk_test_set_clock_ms(0) }; // clear the pin
    assert_no_leak();
}
