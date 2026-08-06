//! DPDK transport error contract (plan §5.1).

use std::fmt;

/// Errors from selecting, configuring, and installing the DPDK ENA transport.
///
/// Every variant carries enough context to identify the failed operation and
/// the exact invalid input — variable name, config field, PCI address, or port
/// role. No variant implies a silent fallback to the socket transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DpdkError {
    /// `dpdk-ena` was selected but the crate was not built with the `dpdk` feature.
    FeatureDisabled,
    /// The transport was requested outside Linux x86_64 (Amazon Linux 2023 / EKS Nitro).
    UnsupportedPlatform,
    /// An EAL/DPDK transport already exists in this process.
    AlreadyInitialized,
    /// A required environment variable is absent.
    MissingEnvironment(String),
    /// A selector or environment value cannot be parsed.
    InvalidEnvironment(String),
    /// A typed configuration violates a cross-field invariant.
    InvalidConfiguration(String),
    /// DPDK or the native transport returned an error code and message.
    Native(String),
    /// Installation through the Aeron context failed.
    Aeron(String),
}

impl fmt::Display for DpdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FeatureDisabled => write!(
                f,
                "the `dpdk-ena` transport was selected but rusteron-media-driver was not built with the `dpdk` feature"
            ),
            Self::UnsupportedPlatform => write!(
                f,
                "the `dpdk-ena` transport requires Linux x86_64 (Amazon Linux 2023 / EKS Nitro)"
            ),
            Self::AlreadyInitialized => write!(
                f,
                "a DPDK EAL/transport is already initialized in this process; DPDK permits only one"
            ),
            Self::MissingEnvironment(var) => write!(f, "missing required environment variable {var}"),
            Self::InvalidEnvironment(msg) => write!(f, "invalid environment value: {msg}"),
            Self::InvalidConfiguration(msg) => write!(f, "invalid DPDK configuration: {msg}"),
            Self::Native(msg) => write!(f, "DPDK native error: {msg}"),
            Self::Aeron(msg) => write!(f, "Aeron installation error: {msg}"),
        }
    }
}

impl std::error::Error for DpdkError {}
