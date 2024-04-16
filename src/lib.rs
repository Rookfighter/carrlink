//! carrlink is a library for interfacing with a Carrera control unit either
//! using a bluetooth connector or a direct serial connection.

mod backend;
mod backend_ble;
mod control_unit;
mod error;
mod lap_time;
mod messages;
mod status;

pub use backend::Backend;
pub use backend_ble::{discover_first_ble, BackendBLE};
pub use control_unit::ControlUnit;
pub use error::Error;
pub use lap_time::LapTime;
pub use status::{LapStatus, StartSignal, Status, TrackStatus, MAX_CONTROLLER_COUNT};

/// Convenience type for a result using the carrlink [`Error`] type.
pub type Result<T> = std::result::Result<T, Error>;
