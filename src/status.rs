use std::time::Duration;

/// Status of the lap of a specific controller.
/// Contains the sector and time of the controller.
pub struct LapStatus {
    /// Identifier of the controller.
    /// Range is typically [0, 8].
    pub controller: usize,

    /// Sector of the track where the time was taken.
    pub sector: usize,

    /// Time measurement for the corresponding controller.
    pub time: Duration,
}

impl LapStatus {
    /// Creates a default initialized status.
    pub fn new() -> LapStatus {
        LapStatus {
            controller: 0,
            sector: 0,
            time: Duration::from_secs(0),
        }
    }
}

/// Maximum number of controllers which can be supported.
pub const MAX_CONTROLLER_COUNT: usize = 8;

/// Start signal which is emitted by the track
pub enum StartSignal {
    None = 0,
    Five = 2,
    Four = 3,
    Three = 4,
    Two = 5,
    One = 6,
    Go = 7,
}

impl TryFrom<u8> for StartSignal {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == StartSignal::None as u8 => Ok(StartSignal::None),
            x if x == StartSignal::Five as u8 => Ok(StartSignal::Five),
            x if x == StartSignal::Four as u8 => Ok(StartSignal::Four),
            x if x == StartSignal::Three as u8 => Ok(StartSignal::Three),
            x if x == StartSignal::Two as u8 => Ok(StartSignal::Two),
            x if x == StartSignal::One as u8 => Ok(StartSignal::One),
            x if x == StartSignal::Go as u8 => Ok(StartSignal::Go),
            _ => Err(()),
        }
    }
}

pub struct TrackStatus {
    /// The fuel level of each controller.
    /// Values are in range [0,15].
    pub fuel_levels: [usize; MAX_CONTROLLER_COUNT],

    /// Determines which controller is refueling at the pit lane.
    pub is_refueling: [bool; MAX_CONTROLLER_COUNT],

    /// Countdown indicator for the start of a race.
    pub start_signal: StartSignal,

    pub is_fuel_enabled: bool,

    /// Determines if real fuel mode is enabled on the track.
    pub is_real_fuel_enabled: bool,

    /// Determines if a pit lane adapter is connected.
    pub is_pit_lane_connected: bool,

    /// Determines oif a lap counter adapter is connected.
    pub is_lap_counter_connected: bool,

    /// Number of controllers which are currently in use.
    pub controller_count: usize,
}

impl TrackStatus {
    pub fn new() -> TrackStatus {
        TrackStatus {
            fuel_levels: [0; MAX_CONTROLLER_COUNT],
            is_refueling: [false; MAX_CONTROLLER_COUNT],
            start_signal: StartSignal::None,
            is_fuel_enabled: false,
            is_real_fuel_enabled: false,
            is_pit_lane_connected: false,
            is_lap_counter_connected: false,
            controller_count: 0,
        }
    }
}

/// Status message that can be returned by the control unit.
/// Either contains a lap status or a track status.
pub enum Status {
    Lap(LapStatus),
    Track(TrackStatus),
}
