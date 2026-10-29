// This file is part of ts_dynalene.
//
// Developed for the Vera C. Rubin Observatory Systems.
// This product includes software developed by the LSST Project
// (https://www.lsst.org).
// See the COPYRIGHT file at the top-level directory of this distribution
// for details of code ownership.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use crc::Crc;
use serde_json::{Value, json};
use std::mem::size_of;

use crate::enums::CommandStatus;

/// Trait for numeric types that can be parsed from a big-endian byte slice.
pub trait FromBeByteSlice: Copy + Default {
    /// Number of bytes needed to represent a single value.
    const SIZE: usize;

    /// Parse a single value from a big-endian byte slice of length `SIZE`.
    fn from_be_byte_slice(bytes: &[u8]) -> Self;
}

/// Implement `FromBeByteSlice` for one or more numeric types that provide an
/// inherent `from_be_bytes()` constructor (e.g. `u16`, `i32`, `f64`).
macro_rules! impl_from_be_byte_slice {
    ($($type:ty),+ $(,)?) => {
        $(
            impl FromBeByteSlice for $type {
                const SIZE: usize = size_of::<$type>();

                fn from_be_byte_slice(bytes: &[u8]) -> Self {
                    <$type>::from_be_bytes(bytes.try_into().unwrap())
                }
            }
        )+
    };
}

impl_from_be_byte_slice!(u16, u32, u64, i16, i32, i64, f32, f64);

/// Get the `T` values from a byte array. The byte array is expected to
/// contain `N` consecutive big-endian values of `T`.
///
/// # Arguments
/// * `frame` - The byte array containing the values.
///
/// # Returns
/// An array of `N` values of `T` extracted from the byte array. Returns
/// `None` if the length of the byte array does not match the expected
/// length of `N * size_of::<T>()` bytes.
pub fn get_values_from_u8_array<T: FromBeByteSlice, const N: usize>(
    frame: &[u8],
) -> Option<[T; N]> {
    if frame.len() != N * T::SIZE {
        return None;
    }

    let mut values = [T::default(); N];
    for (index, value) in values.iter_mut().enumerate() {
        *value = T::from_be_byte_slice(&frame[index * T::SIZE..(index + 1) * T::SIZE]);
    }

    Some(values)
}

/// Calculate the Modbus cyclic redundancy check (CRC) checksum for the given
/// frame and update it.
///
/// # Arguments
/// * `crc` - The CRC calculator to compute the checksum.
/// * `frame` - The frame to be updated with the CRC checksum (final two
///   bytes).
///
/// # Panics
/// Panics if the frame length is less than 3 bytes, as at least 1 byte of
/// data and 2 bytes for CRC are required.
pub fn calculate_modbus_crc_and_update_frame(crc: &Crc<u16>, frame: &mut [u8]) {
    let length = frame.len();
    if length < 3 {
        panic!("Modbus frame must have at least 3 bytes to accommodate data and CRC.");
    }

    let checksum = crc.checksum(&frame[..length - 2]);

    // Convert to Little-Endian (Low byte first)
    let crc_bytes = checksum.to_le_bytes();

    frame[length - 2] = crc_bytes[0];
    frame[length - 1] = crc_bytes[1];
}

/// Verify the Modbus cyclic redundancy check (CRC) of a received frame.
///
/// # Arguments
/// * `frame` - The received frame to be verified.
///
/// # Returns
/// `true` if the CRC is valid, `false` otherwise.
pub fn verify_modbus_crc(crc: &Crc<u16>, frame: &[u8]) -> bool {
    // Not enough data for CRC-16 (2 bytes) + at least 1 byte of data
    let frame_length = frame.len();
    if frame_length < 3 {
        return false;
    }

    let data = &frame[..frame_length - 2];
    let crc_received = u16::from_le_bytes([frame[frame_length - 2], frame[frame_length - 1]]);
    let crc_calculated = crc.checksum(data);

    crc_received == crc_calculated
}

/// Get the index of a value in an array.
///
/// # Arguments
/// * `array` - The array to search.
/// * `value` - The value to find.
///
/// # Returns
/// The index of the value if found, `None` otherwise.
pub fn get_index_from_array<T: PartialEq>(array: &[T], value: &T) -> Option<usize> {
    array.iter().position(|x| *x == *value)
}

/// Convert Celsius temperature to Fahrenheit.
///
/// # Arguments
/// * `celsius` - The temperature in degrees Celsius.
///
/// # Returns
/// The temperature converted to degrees Fahrenheit.
pub fn celsius_to_fahrenheit(celsius: f32) -> f32 {
    celsius * 9.0 / 5.0 + 32.0
}

/// Convert Fahrenheit temperature to Celsius.
///
/// # Arguments
/// * `fahrenheit` - The temperature in degrees Fahrenheit.
///
/// # Returns
/// The temperature converted to degrees Celsius.
pub fn fahrenheit_to_celsius(fahrenheit: f32) -> f32 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

/// Check if the message is a command.
///
/// # Arguments
/// * `name` - Name of the message.
///
/// # Returns
/// True if the message is a command, false otherwise.
pub fn is_command(name: &str) -> bool {
    name.starts_with("cmd_")
}

/// Acknowledge the command.
///
/// # Arguments
/// * `command_name` - Name of the command.
/// * `command_status` - Command status.
/// * `sequence_id` - Sequence ID.
/// * `fail_reason` - Optional reason for failure if the command failed.
///
/// # Returns
/// Acknowledged command.
pub fn acknowledge_command(
    command_name: Option<&str>,
    command_status: CommandStatus,
    sequence_id: i64,
    fail_reason: Option<&str>,
) -> Value {
    let mut ack: Value = json!({
        "id": command_name.unwrap_or(""),
        "state": command_status.as_ref().to_lowercase(),
        "sequence_id": sequence_id,
    });

    if (command_status == CommandStatus::NoAck) || (command_status == CommandStatus::Fail) {
        ack["fail_reason"] = json!(fail_reason.unwrap_or("Unknown reason."));
    }

    ack
}

/// Get the message name.
///
/// # Arguments
/// * `message` - Message that should have the "id" field.
///
/// # Returns
/// Message name. Return an empty string if the name is not found.
pub fn get_message_name(message: &Value) -> String {
    match message["id"].as_str() {
        Some(id) => String::from(id),
        None => String::new(),
    }
}

/// Get the message sequence ID.
///
/// # Arguments
/// * `message` - Message.
///
/// # Returns
/// Message sequence ID. Return -1 if the sequence ID is not found.
pub fn get_message_sequence_id(message: &Value) -> i64 {
    message["sequence_id"].as_i64().unwrap_or(-1)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crc::CRC_16_MODBUS;

    #[test]
    fn test_get_values_from_u8_array_u16() {
        let frame: [u8; 4] = [0x12, 0x34, 0xAB, 0xCD];
        assert_eq!(
            get_values_from_u8_array::<u16, 2>(&frame),
            Some([0x1234, 0xABCD])
        );
    }

    #[test]
    fn test_get_values_from_u8_array_i32() {
        let frame: [u8; 8] = [0x00, 0x00, 0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF];
        assert_eq!(get_values_from_u8_array::<i32, 2>(&frame), Some([1, -1]));
    }

    #[test]
    fn test_get_values_from_u8_array_f32() {
        let frame: [u8; 8] = [0x3f, 0x80, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00];
        assert_eq!(get_values_from_u8_array::<f32, 2>(&frame), Some([1.0, 2.0]));
    }

    #[test]
    fn test_get_values_from_u8_array_f64() {
        let frame: [u8; 8] = [0xbf, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(get_values_from_u8_array::<f64, 1>(&frame), Some([-1.0]));
    }

    #[test]
    fn test_calculate_modbus_crc_and_update_frame() {
        let crc = Crc::<u16>::new(&CRC_16_MODBUS);

        let mut frame_1 = [1, 2, 3, 4, 0, 0];
        calculate_modbus_crc_and_update_frame(&crc, &mut frame_1);

        assert_eq!(frame_1, [1, 2, 3, 4, 0xA1, 0x2B]);

        let mut frame_2 = [1, 2, 3, 4, 0, 0, 0];
        calculate_modbus_crc_and_update_frame(&crc, &mut frame_2);

        assert_eq!(frame_2, [1, 2, 3, 4, 0, 0xEA, 0xB8]);

        let mut frame_3 = [0x01, 0x03, 0x0, 0x04, 0x0, 0x02, 0x0, 0x0];
        calculate_modbus_crc_and_update_frame(&crc, &mut frame_3);

        assert_eq!(frame_3, [0x01, 0x03, 0x0, 0x04, 0x0, 0x02, 0x85, 0xCA]);
    }

    #[should_panic(
        expected = "Modbus frame must have at least 3 bytes to accommodate data and CRC."
    )]
    #[test]
    fn test_calculate_modbus_crc_and_update_frame_panic() {
        let crc = Crc::<u16>::new(&CRC_16_MODBUS);

        calculate_modbus_crc_and_update_frame(&crc, &mut [0; 2]);
    }

    #[test]
    fn test_verify_modbus_crc() {
        let crc = Crc::<u16>::new(&CRC_16_MODBUS);

        // Valid frame with correct CRC
        let frame_valid: [u8; 8] = [0x01, 0x03, 0x0, 0x04, 0x0, 0x02, 0x85, 0xCA];

        assert!(verify_modbus_crc(&crc, &frame_valid));

        // Invalid frame with corrupted value
        let frame_invalid: [u8; 8] = [0x00, 0x03, 0x0, 0x04, 0x0, 0x02, 0x85, 0xCA];

        assert!(!verify_modbus_crc(&crc, &frame_invalid));

        // No enough data
        let frame_short: [u8; 2] = [0x01, 0x03];
        assert!(!verify_modbus_crc(&crc, &frame_short));
    }

    #[test]
    fn test_get_index_from_array() {
        let array = [10, 20, 30, 40, 50];

        assert_eq!(get_index_from_array(&array, &30), Some(2));
        assert_eq!(get_index_from_array(&array, &60), None);
    }

    #[test]
    fn test_celsius_to_fahrenheit() {
        assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
        assert_eq!(celsius_to_fahrenheit(100.0), 212.0);
        assert_eq!(celsius_to_fahrenheit(-40.0), -40.0);
    }

    #[test]
    fn test_fahrenheit_to_celsius() {
        assert_eq!(fahrenheit_to_celsius(32.0), 0.0);
        assert_eq!(fahrenheit_to_celsius(212.0), 100.0);
        assert_eq!(fahrenheit_to_celsius(-40.0), -40.0);
    }

    #[test]
    fn test_is_command() {
        assert_eq!(is_command("cmd_test"), true);
        assert_eq!(is_command("evt_test"), false);
        assert_eq!(is_command("tel_test"), false);
        assert_eq!(is_command("test"), false);
    }

    #[test]
    fn test_acknowledge_command() {
        assert_eq!(
            acknowledge_command(Some("cmd_test"), CommandStatus::Success, 1, None),
            json!({"id": "cmd_test", "state": "success", "sequence_id": 1})
        );
        assert_eq!(
            acknowledge_command(None, CommandStatus::Success, 1, None),
            json!({"id": "", "state": "success", "sequence_id": 1})
        );

        assert_eq!(
            acknowledge_command(
                Some("cmd_test"),
                CommandStatus::Fail,
                2,
                Some("Command failed.")
            ),
            json!({"id": "cmd_test", "state": "fail", "sequence_id": 2, "fail_reason": "Command failed."})
        );
        assert_eq!(
            acknowledge_command(None, CommandStatus::Fail, 2, None),
            json!({"id": "", "state": "fail", "sequence_id": 2, "fail_reason": "Unknown reason."})
        );

        assert_eq!(
            acknowledge_command(Some("cmd_test"), CommandStatus::Ack, 3, None),
            json!({"id": "cmd_test", "state": "ack", "sequence_id": 3})
        );
        assert_eq!(
            acknowledge_command(None, CommandStatus::Ack, 3, None),
            json!({"id": "", "state": "ack", "sequence_id": 3})
        );

        assert_eq!(
            acknowledge_command(
                Some("cmd_test"),
                CommandStatus::NoAck,
                4,
                Some("Unknown command.")
            ),
            json!({"id": "cmd_test", "state": "noack", "sequence_id": 4, "fail_reason": "Unknown command."})
        );
        assert_eq!(
            acknowledge_command(None, CommandStatus::NoAck, 4, None),
            json!({"id": "", "state": "noack", "sequence_id": 4, "fail_reason": "Unknown reason."})
        );
    }

    #[test]
    fn test_get_message_name() {
        // There is the "id" field.
        assert_eq!(get_message_name(&json!({"id": "cmd_test"})), "cmd_test");
        assert_eq!(get_message_name(&json!({"id": 1})), "");

        // There is no "id" field.
        assert_eq!(get_message_name(&json!({})), "");
        assert_eq!(get_message_name(&json!({"test": 1})), "");
    }

    #[test]
    fn test_get_message_sequence_id() {
        // There is the "sequence_id" field.
        assert_eq!(get_message_sequence_id(&json!({"sequence_id": 1})), 1);
        assert_eq!(get_message_sequence_id(&json!({"sequence_id": 1.0})), -1);

        // There is no "sequence_id" field.
        assert_eq!(get_message_sequence_id(&json!({})), -1);
        assert_eq!(get_message_sequence_id(&json!({"test": 1})), -1);
    }
}
