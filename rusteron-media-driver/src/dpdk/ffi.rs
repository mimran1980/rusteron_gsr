//! FFI to the native DPDK transport and the install/lifetime path (plan §5.2, §7).

use crate::dpdk::config::DpdkTransportConfig;
use crate::dpdk::error::DpdkError;
use crate::AeronDriverContext;
use std::sync::Arc;

#[cfg(feature = "dpdk")]
use std::ffi::CStr;
#[cfg(feature = "dpdk")]
use std::os::raw::{c_char, c_int, c_void};

/// Arc-owned handle to the native DPDK transport.
///
/// Cloning shares the same native instance. The native transport is closed
/// exactly once, when the last clone is dropped — including the clone retained
/// in the `AeronDriverContext` dependency graph by [`Self::install`].
#[derive(Clone, Debug)]
pub struct DpdkTransport {
    // Read only in the `dpdk`-feature install path; `allow` so feature-less
    // builds (where install always returns `FeatureDisabled`) don't warn.
    #[allow(dead_code)]
    inner: Arc<NativeTransport>,
}

/// Opaque native transport state. Empty without the `dpdk` feature.
#[derive(Debug)]
struct NativeTransport {
    #[cfg(feature = "dpdk")]
    ptr: *mut c_void,
}

impl DpdkTransport {
    /// Validate the configuration and existing Aeron context, create the native
    /// DPDK runtime, install the transport bindings into `context`, and retain
    /// a clone in the context dependency graph so the native state is destroyed
    /// only after the context (and the driver borrowing it) goes away (§5.2).
    ///
    /// Without the `dpdk` feature this returns [`DpdkError::FeatureDisabled`]
    /// before inspecting the config or context.
    pub fn install(context: &AeronDriverContext, config: DpdkTransportConfig) -> Result<Self, DpdkError> {
        #[cfg(feature = "dpdk")]
        {
            // 3. Validate the Rust configuration and existing Aeron context.
            config.validate()?;
            validate_context(context, &config)?;

            // 4. Create the native DPDK runtime (EAL + both ports land in Ticket 3).
            let transport = Self {
                inner: Arc::new(NativeTransport::create(&config)?),
            };

            // 5. Install the DPDK binding pointer and client state into the context.
            transport.inner.install(context)?;

            // 6. Retain a clone so the caller's guard can be dropped without
            //    stopping DPDK while the context is still alive.
            context.add_dependency(transport.clone());

            return Ok(transport);
        }
        #[cfg(not(feature = "dpdk"))]
        {
            let _ = (context, config);
            return Err(DpdkError::FeatureDisabled);
        }
    }
}

#[cfg(feature = "dpdk")]
impl NativeTransport {
    fn create(config: &DpdkTransportConfig) -> Result<Self, DpdkError> {
        let native_config = native_config(config)?;
        let mut ptr: *mut c_void = std::ptr::null_mut();
        let rc = unsafe { rusteron_dpdk_transport_create(&native_config, &mut ptr) };
        if rc != 0 {
            return Err(DpdkError::Native(last_error()));
        }
        Ok(Self { ptr })
    }

    fn install(&self, context: &AeronDriverContext) -> Result<(), DpdkError> {
        let rc = unsafe { rusteron_dpdk_transport_install(self.ptr, context.get_inner()) };
        if rc != 0 {
            return Err(DpdkError::Aeron(last_error()));
        }
        Ok(())
    }
}

// The DPDK runtime is designed for the multi-threaded media driver (the send
// and receive agents both touch it); the raw pointer is an opaque handle into
// it. Mirrors the crate's existing `unsafe impl Send/Sync for AeronDriverContext`.
#[cfg(feature = "dpdk")]
unsafe impl Send for NativeTransport {}
#[cfg(feature = "dpdk")]
unsafe impl Sync for NativeTransport {}

#[cfg(feature = "dpdk")]
impl Drop for NativeTransport {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { rusteron_dpdk_transport_close(self.ptr) };
        }
    }
}

#[cfg(not(feature = "dpdk"))]
impl Drop for NativeTransport {
    fn drop(&mut self) {}
}

/// Mirror of the C `rusteron_dpdk_config_t` layout guard (Ticket 1).
#[cfg(feature = "dpdk")]
#[repr(C)]
struct rusteron_dpdk_config_t {
    struct_size: u32,
    file_prefix: [c_char; 65],
    hugepage_dir: [c_char; 4096],
    sender_pci: [c_char; 16],
    sender_ipv4: [c_char; 16],
    sender_prefix_len: u8,
    sender_gateway: [c_char; 16],
    receiver_pci: [c_char; 16],
    receiver_ipv4: [c_char; 16],
    receiver_prefix_len: u8,
    receiver_gateway: [c_char; 16],
    rx_descriptors: u16,
    tx_descriptors: u16,
    mbufs_per_port: u32,
    mempool_cache: u16,
    burst_size: u16,
    max_aeron_mtu: usize,
}

#[cfg(feature = "dpdk")]
#[link(name = "rusteron_dpdk", kind = "static")]
extern "C" {
    fn rusteron_dpdk_transport_create(config: *const rusteron_dpdk_config_t, transport: *mut *mut c_void) -> c_int;
    fn rusteron_dpdk_transport_install(
        transport: *mut c_void,
        context: *mut crate::bindings::aeron_driver_context_t,
    ) -> c_int;
    fn rusteron_dpdk_transport_close(transport: *mut c_void) -> c_int;
    fn rusteron_dpdk_last_error() -> *const c_char;
}

#[cfg(feature = "dpdk")]
fn native_config(config: &DpdkTransportConfig) -> Result<rusteron_dpdk_config_t, DpdkError> {
    let mut c = rusteron_dpdk_config_t {
        struct_size: std::mem::size_of::<rusteron_dpdk_config_t>() as u32,
        file_prefix: [0; 65],
        hugepage_dir: [0; 4096],
        sender_pci: [0; 16],
        sender_ipv4: [0; 16],
        sender_prefix_len: config.sender.prefix_len,
        sender_gateway: [0; 16],
        receiver_pci: [0; 16],
        receiver_ipv4: [0; 16],
        receiver_prefix_len: config.receiver.prefix_len,
        receiver_gateway: [0; 16],
        rx_descriptors: config.rx_descriptors,
        tx_descriptors: config.tx_descriptors,
        mbufs_per_port: config.mbufs_per_port,
        mempool_cache: config.mempool_cache,
        burst_size: config.burst_size,
        max_aeron_mtu: config.max_aeron_mtu,
    };
    fill(&mut c.file_prefix, &config.file_prefix, "file_prefix")?;
    let huge_dir = config
        .hugepage_dir
        .to_str()
        .ok_or_else(|| DpdkError::InvalidConfiguration("hugepage_dir is not valid UTF-8".to_string()))?;
    fill(&mut c.hugepage_dir, huge_dir, "hugepage_dir")?;
    fill(&mut c.sender_pci, &config.sender.pci_address, "sender.pci_address")?;
    fill(
        &mut c.sender_ipv4,
        &config.sender.local_ipv4.to_string(),
        "sender.local_ipv4",
    )?;
    fill(
        &mut c.sender_gateway,
        &config.sender.gateway_ipv4.to_string(),
        "sender.gateway_ipv4",
    )?;
    fill(
        &mut c.receiver_pci,
        &config.receiver.pci_address,
        "receiver.pci_address",
    )?;
    fill(
        &mut c.receiver_ipv4,
        &config.receiver.local_ipv4.to_string(),
        "receiver.local_ipv4",
    )?;
    fill(
        &mut c.receiver_gateway,
        &config.receiver.gateway_ipv4.to_string(),
        "receiver.gateway_ipv4",
    )?;
    Ok(c)
}

/// Copy a NUL-terminated string into a fixed C buffer.
#[cfg(feature = "dpdk")]
fn fill(buf: &mut [c_char], value: &str, field: &str) -> Result<(), DpdkError> {
    let bytes = value.as_bytes();
    if bytes.len() >= buf.len() {
        return Err(DpdkError::InvalidConfiguration(format!(
            "{field} value {value:?} is too long for the native config buffer"
        )));
    }
    for (i, b) in bytes.iter().enumerate() {
        buf[i] = *b as c_char;
    }
    buf[bytes.len()] = 0;
    Ok(())
}

#[cfg(feature = "dpdk")]
fn last_error() -> String {
    let ptr = unsafe { rusteron_dpdk_last_error() };
    if ptr.is_null() {
        "<null>".to_string()
    } else {
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned()
    }
}

/// Verify the required `AeronDriverContext` state (plan §6.4). The DPDK
/// transport replaces only the sender/receiver bindings; the conductor's UDP
/// resolver stays on the kernel socket, so the context must still be a sane
/// dedicated-thread driver configuration.
#[cfg(feature = "dpdk")]
fn validate_context(context: &AeronDriverContext, config: &DpdkTransportConfig) -> Result<(), DpdkError> {
    use crate::aeron_threading_mode_t;

    if context.get_threading_mode() != aeron_threading_mode_t::AERON_THREADING_MODE_DEDICATED {
        return Err(DpdkError::InvalidConfiguration(
            "aeron context threading mode must be AERON_THREADING_MODE_DEDICATED".to_string(),
        ));
    }

    let sender_aff = context.get_sender_cpu_affinity();
    let receiver_aff = context.get_receiver_cpu_affinity();
    if sender_aff < 0 || receiver_aff < 0 {
        return Err(DpdkError::InvalidConfiguration(format!(
            "aeron context sender/receiver cpu affinity must be nonnegative (sender={sender_aff}, receiver={receiver_aff})"
        )));
    }
    if sender_aff == receiver_aff {
        return Err(DpdkError::InvalidConfiguration(format!(
            "aeron context sender and receiver cpu affinity must differ (both {sender_aff})"
        )));
    }

    if context.get_sender_idle_strategy() != "spin" || context.get_receiver_idle_strategy() != "spin" {
        return Err(DpdkError::InvalidConfiguration(
            "aeron context sender and receiver idle strategies must be `spin`".to_string(),
        ));
    }

    let (s_low, s_high) = wildcard_port_range(context, "sender")?;
    let (r_low, r_high) = wildcard_port_range(context, "receiver")?;
    for (role, low, high) in [("sender", s_low, s_high), ("receiver", r_low, r_high)] {
        if low == 0 || high == 0 {
            return Err(DpdkError::InvalidConfiguration(format!(
                "aeron context {role} wildcard port range ({low}-{high}) must be nonzero"
            )));
        }
        if low >= high {
            return Err(DpdkError::InvalidConfiguration(format!(
                "aeron context {role} wildcard port range ({low}-{high}) must satisfy low < high"
            )));
        }
    }
    if !(s_high < r_low || r_high < s_low) {
        return Err(DpdkError::InvalidConfiguration(format!(
            "aeron context sender wildcard port range ({s_low}-{s_high}) and receiver ({r_low}-{r_high}) must be disjoint"
        )));
    }

    let mtu = context.get_mtu_length();
    if mtu > config.max_aeron_mtu {
        return Err(DpdkError::InvalidConfiguration(format!(
            "aeron context mtu {mtu} exceeds max_aeron_mtu {}",
            config.max_aeron_mtu
        )));
    }

    Ok(())
}

// ponytail: role is the C getter suffix in the error message — keep it
// byte-identical to the bindings' aeron_driver_context_get_{role}_wildcard_port_range.
#[cfg(feature = "dpdk")]
fn wildcard_port_range(context: &AeronDriverContext, role: &str) -> Result<(u16, u16), DpdkError> {
    let (mut low, mut high) = (0u16, 0u16);
    let result = if role == "sender" {
        context.get_sender_wildcard_port_range(&mut low, &mut high)
    } else {
        context.get_receiver_wildcard_port_range(&mut low, &mut high)
    };
    result.map_err(|e| {
        DpdkError::Aeron(format!(
            "aeron_driver_context_get_{role}_wildcard_port_range failed: {e}"
        ))
    })?;
    Ok((low, high))
}
