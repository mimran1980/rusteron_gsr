//! DPDK ENA kernel-bypass transport (optional, `dpdk` feature).
//!
//! The module is always compiled so applications can write transport-selection
//! code without conditional imports; native DPDK code is linked only with the
//! `dpdk` feature. Selecting `dpdk-ena` without the feature returns
//! [`DpdkError::FeatureDisabled`].
//!
//! Contracts: plan §5 (Public Rust Contract) and §6 (Environment Contract).

pub mod config;
pub mod env;
pub mod error;
pub mod ffi;

pub use config::{DpdkPortConfig, DpdkTransportConfig};
pub use env::Selector;
pub use error::DpdkError;
pub use ffi::DpdkTransport;

use crate::AeronDriverContext;

/// Read the `RUSTERON_MEDIA_DRIVER_TRANSPORT` selector (§6.1). When it selects
/// `dpdk-ena`, parse the DPDK environment (§6.2/§6.3), validate it (§6.5),
/// install the transport into `context`, and return the guard.
///
/// Returns `Ok(None)` when the selector is absent or `default` — the existing
/// socket-based behaviour is unchanged.
pub fn configure_media_transport_from_env(
    context: &AeronDriverContext,
) -> Result<Option<DpdkTransport>, DpdkError> {
    match env::selector()? {
        Selector::Default => Ok(None),
        Selector::DpdkEna => install_from_env(context),
    }
}

#[cfg(feature = "dpdk")]
fn install_from_env(context: &AeronDriverContext) -> Result<Option<DpdkTransport>, DpdkError> {
    let config = env::config_from_env()?;
    let transport = DpdkTransport::install(context, config)?;
    Ok(Some(transport))
}

#[cfg(not(feature = "dpdk"))]
fn install_from_env(_context: &AeronDriverContext) -> Result<Option<DpdkTransport>, DpdkError> {
    Err(DpdkError::FeatureDisabled)
}
