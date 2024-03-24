//! Defines the general error type of carrlink.

use std::error;
use std::fmt;

/// Enumeration of error cases.
#[derive(Debug)]
pub enum Error {
    PermissionDenied,
    DeviceNotFound,
    NotConnected,
    NotSupported(String),
    TimedOut,
    RuntimeError(String),
    InvalidResponse,
    NoResponse,
    Other(Box<dyn error::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

impl error::Error for Error {}
