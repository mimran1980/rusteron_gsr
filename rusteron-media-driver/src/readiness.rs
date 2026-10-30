//! The readiness interceptor: the receiver marks which streams have new data, in counters a
//! client reads with [`AeronReadiness`](crate::AeronReadiness).
//!
//! It is an incoming UDP channel interceptor. On the receiver it passes each datagram on
//! first, which writes the frame to its image before returning
//! (`aeron_publication_image_insert_packet` stores the frame length last, with release), and
//! then sets the stream's bit. On the sender and conductor it only passes datagrams on.

use crate::bindings::*;
use crate::{AERON_READINESS_COUNTER_TYPE_ID, AeronCError, AeronDriverContext};
use std::ffi::{c_int, c_void};
use std::sync::atomic::{AtomicI64, Ordering, fence};

/// Readiness words a driver may keep: 64 streams per word.
pub const MAX_READINESS_WORDS: usize = 8;

/// `AERON_HDR_TYPE_DATA` in `aeron_udp_protocol.h`.
const HDR_TYPE_DATA: i16 = 0x01;
const DATA_HEADER_LENGTH: usize = size_of::<aeron_data_header_t>();

/// Marks `datagram`'s stream in `words` if it carries a data frame with a payload, as the
/// interceptor does once the receiver has written the datagram to its image.
#[inline]
pub fn mark_ready(words: &[&AtomicI64], datagram: &[u8]) {
    let Some(header) = datagram.get(..DATA_HEADER_LENGTH) else {
        return;
    };
    let field = |at: usize| i32::from_le_bytes([header[at], header[at + 1], header[at + 2], header[at + 3]]);
    let frame_type = i16::from_le_bytes([header[6], header[7]]);
    // a heartbeat has frame length 0; a header-only frame has nothing to poll for
    if frame_type != HDR_TYPE_DATA || field(0) <= DATA_HEADER_LENGTH as i32 || words.is_empty() {
        return;
    }
    let bit = field(16).rem_euclid(words.len() as i32 * 64) as usize;
    let (word, mask) = (words[bit / 64], 1i64 << (bit % 64));
    // Orders the frame write before the bit read. A client clears the bit, fences, then
    // polls: either it sees this frame, or this read sees the bit clear and sets it again.
    fence(Ordering::SeqCst);
    // a busy stream finds its bit set and writes nothing
    if word.load(Ordering::Relaxed) & mask == 0 {
        word.fetch_or(mask, Ordering::Release);
    }
}

/// The receiver's readiness words, as the interceptor state.
struct Words(Vec<&'static AtomicI64>);

/// Allocates `W` readiness counters for the receiver; other agents get no state.
unsafe extern "C" fn init<const W: usize>(
    state: *mut *mut c_void,
    context: *mut aeron_driver_context_t,
    affinity: aeron_udp_channel_transport_affinity_t,
) -> c_int {
    // SAFETY: the driver passes a valid state slot and its context, whose counters manager
    // exists before the receiver initialises
    unsafe {
        *state = std::ptr::null_mut();
        if affinity != aeron_udp_channel_transport_affinity_t::AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER {
            return 0;
        }
        let manager = (*context).counters_manager;
        let mut words = Vec::with_capacity(W);
        for word in 0..W {
            let key = (word as i32).to_le_bytes();
            let label = format!(
                "readiness word {word}: streams % {} in {}..{}",
                W * 64,
                word * 64,
                word * 64 + 64
            );
            let id = aeron_counters_manager_allocate(
                manager,
                AERON_READINESS_COUNTER_TYPE_ID,
                key.as_ptr(),
                key.len(),
                label.as_ptr().cast(),
                label.len(),
            );
            if id < 0 {
                return -1;
            }
            let value = (*manager)
                .values
                .add(id as usize * size_of::<aeron_counter_value_descriptor_t>());
            words.push(AtomicI64::from_ptr(value.cast::<i64>()));
        }
        *state = Box::into_raw(Box::new(Words(words))).cast();
        0
    }
}

unsafe extern "C" fn incoming(
    state: *mut c_void,
    delegate: *mut aeron_udp_channel_incoming_interceptor_t,
    transport: *mut aeron_udp_channel_transport_t,
    receiver_clientd: *mut c_void,
    endpoint_clientd: *mut c_void,
    destination_clientd: *mut c_void,
    buffer: *mut u8,
    length: usize,
    addr: *mut sockaddr_storage,
    media_timestamp: *mut timespec,
) {
    // SAFETY: the driver passes the next interceptor in the chain and a datagram of `length`
    // bytes; `state` is null or the `Words` made by `init`, alive until `close`
    unsafe {
        if let Some(next) = (*delegate).incoming_func {
            next(
                (*delegate).interceptor_state,
                (*delegate).next_interceptor,
                transport,
                receiver_clientd,
                endpoint_clientd,
                destination_clientd,
                buffer,
                length,
                addr,
                media_timestamp,
            );
        }
        if let Some(Words(words)) = state.cast::<Words>().as_ref() {
            mark_ready(words, std::slice::from_raw_parts(buffer, length));
        }
    }
}

unsafe extern "C" fn close(state: *mut c_void) -> c_int {
    if !state.is_null() {
        // SAFETY: made by `init` with `Box::into_raw`, and closed once
        drop(unsafe { Box::from_raw(state.cast::<Words>()) });
    }
    0
}

/// `init` for each supported word count, the count being fixed when the bindings are made.
const INITS: [aeron_udp_channel_interceptor_init_func_t; MAX_READINESS_WORDS] = [
    Some(init::<1>),
    Some(init::<2>),
    Some(init::<3>),
    Some(init::<4>),
    Some(init::<5>),
    Some(init::<6>),
    Some(init::<7>),
    Some(init::<8>),
];

impl AeronDriverContext {
    /// Installs the readiness interceptor ahead of any incoming interceptors already set, so
    /// the receiver keeps `words` readiness words (64 streams each) for clients to read with
    /// [`AeronReadiness`](crate::AeronReadiness). Call before the driver starts. Each data
    /// datagram then costs the receiver a fence and a load, plus an atomic OR when its bit
    /// is clear.
    ///
    /// # Errors
    ///
    /// `words` is not in `1..=MAX_READINESS_WORDS`, or the bindings could not be allocated.
    pub fn set_readiness_interceptor(&self, words: usize) -> Result<(), AeronCError> {
        if !(1..=MAX_READINESS_WORDS).contains(&words) {
            return Err(AeronCError::with_message(
                -1,
                format!("readiness words must be 1..={MAX_READINESS_WORDS}, not {words}"),
            ));
        }
        let context = self.get_inner();
        // SAFETY: the context is live and not started; the driver frees the bindings with
        // aeron_free when the context closes, so they come from aeron_alloc
        unsafe {
            let mut bindings: *mut c_void = std::ptr::null_mut();
            if aeron_alloc(&mut bindings, size_of::<aeron_udp_channel_interceptor_bindings_t>()) < 0 {
                return Err(AeronCError::from_code(-1).capture_errmsg());
            }
            let bindings = bindings.cast::<aeron_udp_channel_interceptor_bindings_t>();
            bindings.write(aeron_udp_channel_interceptor_bindings_t {
                outgoing_init_func: None,
                incoming_init_func: INITS[words - 1],
                outgoing_send_func: None,
                incoming_func: Some(incoming),
                outgoing_close_func: None,
                incoming_close_func: Some(close),
                outgoing_transport_notification_func: None,
                incoming_transport_notification_func: None,
                outgoing_publication_notification_func: None,
                incoming_publication_notification_func: None,
                outgoing_image_notification_func: None,
                incoming_image_notification_func: None,
                meta_info: aeron_udp_channel_interceptor_bindings_stct_interceptor_meta_info_fields {
                    name: c"readiness".as_ptr(),
                    type_: c"interceptor".as_ptr(),
                    next_interceptor_bindings: aeron_driver_context_get_udp_channel_incoming_interceptors(context),
                    source_symbol: None,
                },
            });
            let result = aeron_driver_context_set_udp_channel_incoming_interceptors(context, bindings);
            if result < 0 {
                return Err(AeronCError::from_code(result).capture_errmsg());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::EmbeddedDriver;
    use crate::{Aeron, AeronContext, AeronReadiness, Handlers, IntoCString};
    use serial_test::serial;
    use std::sync::atomic::Ordering;
    use std::time::{Duration, Instant};

    const DATA: i16 = 0x01;
    const SETUP: i16 = 0x05;

    fn frame(frame_type: i16, frame_length: i32, stream_id: i32) -> [u8; 64] {
        let mut f = [0u8; 64];
        f[0..4].copy_from_slice(&frame_length.to_le_bytes());
        f[6..8].copy_from_slice(&frame_type.to_le_bytes());
        f[16..20].copy_from_slice(&stream_id.to_le_bytes());
        f
    }

    fn words(n: usize) -> Vec<AtomicI64> {
        (0..n).map(|_| AtomicI64::new(0)).collect()
    }

    fn values(words: &[AtomicI64]) -> Vec<i64> {
        words.iter().map(|w| w.load(Ordering::Relaxed)).collect()
    }

    #[test]
    fn data_frame_sets_its_stream_bit() {
        let w = words(2);
        let refs: Vec<&AtomicI64> = w.iter().collect();
        mark_ready(&refs, &frame(DATA, 40, 5));
        assert_eq!(values(&w), [1 << 5, 0]);
        mark_ready(&refs, &frame(DATA, 40, 70));
        assert_eq!(values(&w), [1 << 5, 1 << 6]);
        // 133 % 128 == 5: shares a bit, which stays set
        mark_ready(&refs, &frame(DATA, 40, 133));
        assert_eq!(values(&w), [1 << 5, 1 << 6]);
    }

    #[test]
    fn stream_maps_to_bit_modulo_word_count() {
        let w = words(4);
        let refs: Vec<&AtomicI64> = w.iter().collect();
        mark_ready(&refs, &frame(DATA, 40, 133));
        assert_eq!(values(&w), [0, 0, 1 << 5, 0]);
        mark_ready(&refs, &frame(DATA, 40, -1));
        assert_eq!(values(&w), [0, 0, 1 << 5, 1 << 63]);
    }

    #[test]
    fn frames_without_payload_or_data_set_nothing() {
        let w = words(2);
        let refs: Vec<&AtomicI64> = w.iter().collect();
        // heartbeat: a data header with frame length 0
        mark_ready(&refs, &frame(DATA, 0, 5));
        // empty message: header only
        mark_ready(&refs, &frame(DATA, 32, 5));
        mark_ready(&refs, &frame(SETUP, 40, 5));
        mark_ready(&refs, &frame(DATA, 40, 5)[..31]);
        assert_eq!(values(&w), [0, 0]);
    }

    #[test]
    fn rejects_word_counts_out_of_range() {
        let context = AeronDriverContext::new().unwrap();
        assert!(context.set_readiness_interceptor(0).is_err());
        assert!(context.set_readiness_interceptor(MAX_READINESS_WORDS + 1).is_err());
    }

    /// A message on a UDP stream sets that stream's bit in the driver's readiness words, and
    /// taking it clears it.
    #[test]
    #[serial]
    fn receiver_marks_streams_with_new_data() -> Result<(), Box<dyn std::error::Error>> {
        let driver = EmbeddedDriver::launch_with(|ctx| {
            ctx.set_readiness_interceptor(2)?;
            Ok(())
        })?;
        let context = AeronContext::new()?;
        context.set_dir(&driver.dir().into_c_string())?;
        let aeron = Aeron::new(&context)?;
        aeron.start()?;
        let readiness = AeronReadiness::find(&aeron).ok_or("driver keeps no readiness words")?;
        assert_eq!(128, readiness.bits());
        assert_eq!((1, 6), readiness.bit_of(70));

        let channel = c"aeron:udp?endpoint=127.0.0.1:24570";
        let subscription = aeron
            .async_add_subscription(channel, 70, Handlers::NONE, Handlers::NONE)?
            .poll_blocking(Duration::from_secs(5))?;
        let publication = aeron
            .async_add_exclusive_publication(channel, 70)?
            .poll_blocking(Duration::from_secs(5))?;
        let deadline = Instant::now() + Duration::from_secs(5);
        while !publication.is_connected() || !subscription.is_connected() {
            assert!(Instant::now() < deadline, "stream never connected");
            std::thread::sleep(Duration::from_millis(10));
        }
        // connecting sends only setup frames and heartbeats, which set nothing
        assert_eq!(0, readiness.take(0, u64::MAX) | readiness.take(1, u64::MAX));

        while publication.offer(b"ready").is_err() {
            assert!(Instant::now() < deadline, "offer never succeeded");
        }
        let mut received = 0;
        while received == 0 {
            received = subscription.poll_fn(|_, _| {}, 10)?;
            assert!(Instant::now() < deadline, "message never arrived");
        }
        // the receiver marks after writing the frame, so the bit is set by now
        assert_eq!(1 << 6, readiness.take(1, u64::MAX));
        assert_eq!(0, readiness.take(1, u64::MAX), "take clears the bit");
        assert_eq!(0, readiness.take(0, u64::MAX));
        Ok(())
    }

    #[test]
    #[serial]
    fn no_readiness_words_without_the_interceptor() -> Result<(), Box<dyn std::error::Error>> {
        let driver = EmbeddedDriver::launch()?;
        let context = AeronContext::new()?;
        context.set_dir(&driver.dir().into_c_string())?;
        let aeron = Aeron::new(&context)?;
        aeron.start()?;
        assert!(AeronReadiness::find(&aeron).is_none());
        Ok(())
    }
}
