use std::time::Duration;

pub enum Error {
    PermissionDenied,
    DeviceNotFound,
    NotConnected,
    UnexpectedCallback,
    NotSupported(String),
    TimedOut(Duration),
    RuntimeError(String),
    InvalidResponse,
    NoResponse,
    Other,
}
