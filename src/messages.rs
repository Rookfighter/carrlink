use std::time::Duration;

use super::StartSignal;
use super::MAX_CONTROLLER_COUNT;

use super::{LapStatus, Status, TrackStatus};

const MIN_CHECKSUM_MESSAGE_LEN: usize = 2;
pub const STATUS_REQUEST: [u8; 1] = [b'?'];
pub const VERSION_REQUEST: [u8; 1] = [b'0'];

/// Computes the checksum of the given slice of data.
fn compute_checksum(data: &[u8]) -> u8 {
    let sum: u32 = data.iter().map(|c| *c as u32).sum();
    (sum & 0x0F) as u8
}

/// Verifies if the given data slice has a valid checksum.
fn check_checksum(data: &[u8]) -> bool {
    if data.len() < MIN_CHECKSUM_MESSAGE_LEN {
        return false;
    }

    let expected = compute_checksum(&data[1..data.len() - 1]);
    let actual = data.last().unwrap() & 0x0F;
    actual == expected
}

const UINT32_SIZE: usize = 8;

fn decode_uint32(data: &[u8]) -> u32 {
    return ((data[0] & 0x0F) as u32) << 24
        | ((data[1] & 0x0F) as u32) << 28
        | ((data[2] & 0x0F) as u32) << 16
        | ((data[3] & 0x0F) as u32) << 20
        | ((data[4] & 0x0F) as u32) << 8
        | ((data[5] & 0x0F) as u32) << 12
        | ((data[6] & 0x0F) as u32)
        | ((data[7] & 0x0F) as u32) << 4;
}

// All values accepted by the control unti have to be added on top of this base.
const VALUE_BASE: u8 = b'0';

/// Encodes the lower nibble of the given value onto the buffer.
const fn encode_nibble(value: u8) -> u8 {
    VALUE_BASE + (value & 0x0F)
}

/// Encodes the address of the player for writing a word.
const fn encode_player_address(address_offset: u8, player: u8) -> u8 {
    let player_validity_mask: u8 = 0x07;
    let address_validity_mask: u8 = 0x1F;
    ((player & player_validity_mask) << 5) | (address_offset & address_validity_mask)
}

fn decode_track_status(data: &[u8]) -> Option<TrackStatus> {
    const FUEL_LEVEL_OFFSET: usize = 2;
    const START_SIGNAL_OFFSET: usize = FUEL_LEVEL_OFFSET + 8;
    const TRACK_MODE_OFFSET: usize = START_SIGNAL_OFFSET + 1;
    const IS_REFUELING_OFFSET: usize = TRACK_MODE_OFFSET + 1;
    const CONTROLLER_COUNT_OFFSET: usize = IS_REFUELING_OFFSET + 2;
    const CHECKSUM_OFFSET: usize = CONTROLLER_COUNT_OFFSET + 1;
    const SHORT_RESPONSE_SIZE: usize = CHECKSUM_OFFSET + 1;
    const LONG_RESPONSE_SIZE: usize = SHORT_RESPONSE_SIZE + 2;

    if data.len() != SHORT_RESPONSE_SIZE && data.len() != LONG_RESPONSE_SIZE {
        return None;
    }

    if data[0] != b'?' && data[1] != b':' {
        return None;
    }

    if !check_checksum(data) {
        return None;
    }

    let mut result = TrackStatus::new();

    // parse fuel levels
    let fuel_level_data = &data[FUEL_LEVEL_OFFSET..FUEL_LEVEL_OFFSET + MAX_CONTROLLER_COUNT];
    for (fuel_level, value) in result.fuel_levels.iter_mut().zip(fuel_level_data.iter()) {
        *fuel_level = (value & 0x0F) as usize;
    }

    // parse start light indicator
    match StartSignal::try_from(data[START_SIGNAL_OFFSET] & 0x0F) {
        Ok(start_signal) => result.start_signal = start_signal,
        Err(_) => return None,
    };

    // parse track mode
    let track_mode = data[TRACK_MODE_OFFSET];
    result.is_fuel_enabled = (track_mode & 0x01) != 0x00;
    result.is_real_fuel_enabled = (track_mode & 0x02) != 0x00;
    result.is_pit_lane_connected = (track_mode & 0x01) != 0x00;
    result.is_lap_counter_connected = (track_mode & 0x02) != 0x00;

    // parse is_refueling
    let refuel_mask =
        (data[IS_REFUELING_OFFSET] & 0x0F) | ((data[IS_REFUELING_OFFSET] & 0x0F) << 4);
    for (i, is_refueling) in result.is_refueling.iter_mut().enumerate() {
        *is_refueling = (refuel_mask & (0x01 << i)) != 0x00;
    }

    result.controller_count = (data[CONTROLLER_COUNT_OFFSET] & 0x0F) as usize;

    Some(result)
}

fn decode_lap_status(data: &[u8]) -> Option<LapStatus> {
    const CONTROLLER_OFFSET: usize = 1;
    const TIME_OFFSET: usize = CONTROLLER_OFFSET + 1;
    const SECTOR_OFFSET: usize = TIME_OFFSET + 8;
    const CHECKSUM_OFFSET: usize = SECTOR_OFFSET + 1;
    const RESPONSE_SIZE: usize = CHECKSUM_OFFSET + 1;

    if data.len() != RESPONSE_SIZE {
        return None;
    }

    if data[0] != b'?' {
        return None;
    }

    if !check_checksum(data) {
        return None;
    }

    let mut result = LapStatus::new();

    result.controller = ((data[CONTROLLER_OFFSET] & 0x0F) - 1) as usize;
    result.time =
        Duration::from_millis(decode_uint32(&data[TIME_OFFSET..TIME_OFFSET + UINT32_SIZE]) as u64);
    result.sector = (data[SECTOR_OFFSET] & 0x0F) as usize;

    Some(result)
}

pub fn decode_status(data: &[u8]) -> Option<Status> {
    match decode_track_status(data) {
        Some(status) => Some(Status::Track(status)),
        None => match decode_lap_status(data) {
            Some(status) => Some(Status::Lap(status)),
            None => None,
        },
    }
}

pub fn decode_version(data: &[u8]) -> Option<String> {
    const RESPONSE_SIZE: usize = 6;

    if data.len() != RESPONSE_SIZE {
        return None;
    }

    if data[0] != b'0' {
        return None;
    }

    if !check_checksum(data) {
        return None;
    }

    let result: String = data[1..data.len() - 1].iter().map(|v| *v as char).collect();
    Some(result)
}

pub fn decode_empty(data_in: &[u8], data_out: &[u8]) -> Option<()> {
    if data_out.len() > 0 && data_in.len() > 0 && data_out[0] == data_in[0] {
        return Some(());
    } else {
        return None;
    }
}

pub fn make_button_press_request(button: u8) -> [u8; 3] {
    let mut result: [u8; 3] = [b'T', encode_nibble(button), 0];
    result[2] = compute_checksum(&result[..2]);
    result
}

fn make_set_word_request(address: u8, value: u8, repetitions: u8) -> [u8; 6] {
    let mut result: [u8; 6] = [
        b'J',
        address & 0x0F,
        address >> 4,
        encode_nibble(value),
        encode_nibble(repetitions),
        0,
    ];

    result[5] = compute_checksum(&result[..5]);
    result
}

pub fn make_reset_positions_request() -> [u8; 6] {
    const WORD_ADDRESS: u8 = 0x06;
    const WORD_VALUE: u8 = 0x09;
    const WORD_REPETITIONS: u8 = 0x01;

    make_set_word_request(WORD_ADDRESS, WORD_VALUE, WORD_REPETITIONS)
}

pub fn make_reset_clock_request() -> [u8; 4] {
    let mut result: [u8; 4] = [b'=', encode_nibble(0x01), encode_nibble(0x00), 0];

    result[3] = compute_checksum(&result[..3]);
    result
}

pub fn make_set_speed_level_request(player: u8, value: u8) -> [u8; 6] {
    const ADDRESS_OFFSET: u8 = 0x00;
    const WORD_REPETITIONS: u8 = 0x02;
    let word_address = encode_player_address(ADDRESS_OFFSET, player);

    make_set_word_request(word_address, value, WORD_REPETITIONS)
}

pub fn make_set_brake_level_request(player: u8, value: u8) -> [u8; 6] {
    const ADDRESS_OFFSET: u8 = 0x01;
    const WORD_REPETITIONS: u8 = 0x02;
    let word_address = encode_player_address(ADDRESS_OFFSET, player);

    make_set_word_request(word_address, value, WORD_REPETITIONS)
}

pub fn make_set_fuel_level_request(player: u8, value: u8) -> [u8; 6] {
    const ADDRESS_OFFSET: u8 = 0x02;
    const WORD_REPETITIONS: u8 = 0x02;
    let word_address = encode_player_address(ADDRESS_OFFSET, player);

    make_set_word_request(word_address, value, WORD_REPETITIONS)
}

pub fn make_set_lap_low_request(value: u8) -> [u8; 6] {
    const WORD_ADDRESS: u8 = 0xF2;
    const WORD_REPETITIONS: u8 = 0x01;
    make_set_word_request(WORD_ADDRESS, value, WORD_REPETITIONS)
}

pub fn make_set_lap_high_request(value: u8) -> [u8; 6] {
    const WORD_ADDRESS: u8 = 0xF1;
    const WORD_REPETITIONS: u8 = 0x01;
    make_set_word_request(WORD_ADDRESS, value, WORD_REPETITIONS)
}
