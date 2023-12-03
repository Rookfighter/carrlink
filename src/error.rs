//! Defines the general error type of carrlink.

use std::error;
use std::{fmt, time::Duration};

#[derive(Debug)]
pub enum Error {
    PermissionDenied,
    DeviceNotFound,
    NotConnected,
    NotSupported(String),
    TimedOut(Duration),
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
