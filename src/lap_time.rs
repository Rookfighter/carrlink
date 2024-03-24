use std::{
    ops::{Add, Sub},
    time::Duration,
};

#[derive(Debug, Clone, Copy)]
pub struct LapTime {
    milliseconds: u32,
}

impl LapTime {
    pub fn from_millis(milliseconds: u32) -> Self {
        Self { milliseconds }
    }
}

impl Add<Duration> for LapTime {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        let rhs_millis = rhs.as_millis() as u32;
        Self {
            milliseconds: self.milliseconds + rhs_millis,
        }
    }
}

impl Sub<Duration> for LapTime {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        let rhs_millis = rhs.as_millis() as u32;
        Self {
            milliseconds: self.milliseconds - rhs_millis,
        }
    }
}

impl Sub<LapTime> for LapTime {
    type Output = Duration;

    fn sub(self, rhs: LapTime) -> Self::Output {
        let delta = self.milliseconds - rhs.milliseconds;
        Duration::from_millis(delta as u64)
    }
}
