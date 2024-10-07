use libaeron_driver_sys::*;

pub mod media_driver;

pub enum AeronError {
    DriverTimeout,
    ClientTimeout,
    ConductorServiceTimeout,
    BufferFull,
    Other(i32),
}

impl From<i32> for AeronError {
    fn from(code: i32) -> Self {
        match code {
            c if c == AERON_CLIENT_ERROR_DRIVER_TIMEOUT => AeronError::DriverTimeout,
            c if c == AERON_CLIENT_ERROR_CLIENT_TIMEOUT => AeronError::ClientTimeout,
            c if c == AERON_CLIENT_ERROR_CONDUCTOR_SERVICE_TIMEOUT => AeronError::ConductorServiceTimeout,
            c if c == AERON_CLIENT_ERROR_BUFFER_FULL => AeronError::BufferFull,
            c => AeronError::Other(c),
        }
    }
}

