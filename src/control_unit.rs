//! Module which implements the core logic to interact with a control unit.

use super::{messages::*, Error, Status};
use crate::Backend;
use std::time::Duration;

pub struct ControlUnit<T: Backend> {
    backend: T,
    timeout: Duration,
}

fn decode_result_to_error<T>(result: Option<T>) -> Result<T, Error> {
    match result {
        Some(value) => Ok(value),
        None => Err(Error::InvalidResponse),
    }
}

const BUTTON_ESCAPE: u8 = 1;
const BUTTON_ENTER: u8 = 2;
const BUTTON_SPEED: u8 = 5;
const BUTTON_BRAKE: u8 = 6;
const BUTTON_FUEL: u8 = 7;
const BUTTON_CODE: u8 = 8;

impl<T: Backend> ControlUnit<T> {
    pub fn new(backend: T) -> ControlUnit<T> {
        ControlUnit {
            backend,
            timeout: Duration::from_secs(2),
        }
    }

    /// Sets the timeout which is used for control unit communication.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Connects the control unit with the configured backend.
    pub async fn connect(&mut self) -> Result<(), Error> {
        self.backend.connect().await
    }

    /// Disconnects the control unit from the configured backend.
    pub async fn disconnect(&mut self) -> Result<(), Error> {
        self.backend.disconnect().await
    }

    /// Determines if the control unti is currently connected.
    pub async fn is_connected(&self) -> Result<bool, Error> {
        self.backend.is_connected().await
    }

    /// Reads the current status during a race.
    /// The control unit can either return a track status or a lap status object.
    pub async fn get_status(&mut self) -> Result<Status, Error> {
        let response = self.backend.request(&STATUS_REQUEST, self.timeout).await?;
        decode_result_to_error(decode_status(&response))
    }

    /// Requests the current firmware version of the control unit.
    pub async fn get_version(&mut self) -> Result<String, Error> {
        let response = self.backend.request(&VERSION_REQUEST, self.timeout).await?;
        decode_result_to_error(decode_version(&response))
    }

    /// Causes a press of the enter button of the control unit.
    pub async fn press_enter(&mut self) -> Result<(), Error> {
        self.press_button(BUTTON_ENTER).await
    }

    /// Causes a press of the escape button of the control unit.
    pub async fn press_esc(&mut self) -> Result<(), Error> {
        self.press_button(BUTTON_ESCAPE).await
    }

    /// Causes a press of the speed button of the control unit.
    pub async fn press_speed(&mut self) -> Result<(), Error> {
        self.press_button(BUTTON_SPEED).await
    }

    /// Causes a press of the brake button of the control unit.
    pub async fn press_brake(&mut self) -> Result<(), Error> {
        self.press_button(BUTTON_BRAKE).await
    }

    /// Causes a press of the fuel button of the control unit.
    pub async fn press_fuel(&mut self) -> Result<(), Error> {
        self.press_button(BUTTON_FUEL).await
    }

    /// Causes a press of the code button of the control unit.
    pub async fn press_code(&mut self) -> Result<(), Error> {
        self.press_button(BUTTON_CODE).await
    }

    /// Simulates a button press with the given button ID.
    async fn press_button(&mut self, button: u8) -> Result<(), Error> {
        let request = make_button_press_request(button);
        let response = self.backend.request(&request, self.timeout).await?;
        decode_result_to_error(decode_empty(&request, &response))
    }

    /// Resets the positions of the players displayed on the position tower.
    pub async fn reset_positions(&mut self) -> Result<(), Error> {
        let request = make_reset_positions_request();
        let response = self.backend.request(&request, self.timeout).await?;
        decode_result_to_error(decode_empty(&request, &response))
    }

    /// Resets the clock for all players.
    pub async fn reset_clock(&mut self) -> Result<(), Error> {
        let request = make_reset_clock_request();
        let response = self.backend.request(&request, self.timeout).await?;
        decode_result_to_error(decode_empty(&request, &response))
    }

    /// Sets the speed level of the given player to the given value.
    /// The speed value will be clamped to [0, 15].
    pub async fn set_speed_level(&mut self, player: usize, speed: usize) -> Result<(), Error> {
        let request = make_set_speed_level_request(player as u8, speed as u8);
        let response = self.backend.request(&request, self.timeout).await?;
        decode_result_to_error(decode_empty(&request, &response))
    }

    /// Sets the brake level of the given player to the given value.
    /// The brake value will be clamped to [0, 15].
    pub async fn set_brake_level(&mut self, player: usize, brake: usize) -> Result<(), Error> {
        let request = make_set_brake_level_request(player as u8, brake as u8);
        let response = self.backend.request(&request, self.timeout).await?;
        decode_result_to_error(decode_empty(&request, &response))
    }

    /// Sets the fuel level of the given player to the given value.
    /// The fuel value will be clamped to [0, 15].
    pub async fn set_fuel_level(&mut self, player: usize, brake: usize) -> Result<(), Error> {
        let request = make_set_fuel_level_request(player as u8, brake as u8);
        let response = self.backend.request(&request, self.timeout).await?;
        decode_result_to_error(decode_empty(&request, &response))
    }

    async fn set_lap_low(&mut self, lap: usize) -> Result<(), Error> {
        let request = make_set_lap_low_request((lap as u8) & 0x0F);
        let response = self.backend.request(&request, self.timeout).await?;
        decode_result_to_error(decode_empty(&request, &response))
    }

    async fn set_lap_high(&mut self, lap: usize) -> Result<(), Error> {
        let request = make_set_lap_high_request((lap as u8) >> 4);
        let response = self.backend.request(&request, self.timeout).await?;
        decode_result_to_error(decode_empty(&request, &response))
    }

    /// Sets the lap currently displayed lap by the position tower.
    pub async fn set_lap(&mut self, lap: usize) -> Result<(), Error> {
        self.set_lap_high(lap).await?;
        self.set_lap_low(lap).await?;
        Ok(())
    }
}
