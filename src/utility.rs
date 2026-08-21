// This file is part of ts_dynalene.
//
// Developed for the Vera Rubin Observatory Systems.
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
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crc::Crc;
use std::mem::size_of;

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
}
