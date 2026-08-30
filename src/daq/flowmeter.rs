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

use crate::constants::NUM_REGISTER_FLOWMETER;
use crate::utility::get_values_from_u8_array;

#[derive(Debug, PartialEq)]
pub struct Flowmeter {
    // Address of the flowmeter.
    pub address: u8,
    // No unit for the signal strength.
    pub signal_strength: f32,
    // The unit of flow rate is liter/minute.
    pub flow_rate: f32,
    // Calculates the final difference by subtracting the negative total from
    // the positive total (positive_totalizer - negative_totalizer). The unit
    // is liter.
    pub net_totalizer: f32,
    // Accumulates all fluid volume moving in the designated forward direction.
    // The unit is liter.
    pub positive_totalizer: f32,
    // Accumulates all fluid volume moving backward through the meter during
    // reverse flow. The unit is liter.
    pub negative_totalizer: f32,
}

impl Flowmeter {
    /// Flowmeter to have the measured flow rate and totalizer values.
    ///
    /// # Arguments
    /// * `address` - The address of the flowmeter.
    ///
    /// # Returns
    /// A new instance of `Flowmeter`.
    pub fn new(address: u8) -> Self {
        Self {
            address,

            signal_strength: 0.0,
            flow_rate: 0.0,
            net_totalizer: 0.0,
            positive_totalizer: 0.0,
            negative_totalizer: 0.0,
        }
    }

    /// Create a `Flowmeter` instance from a Modbus frame.
    ///
    /// # Arguments
    /// * `frame` - The Modbus frame containing the flowmeter data.
    ///
    /// # Returns
    /// An `Option` containing the `Flowmeter` if the frame is valid, or
    /// `None` otherwise.
    pub fn from_frame(frame: &[u8]) -> Option<Flowmeter> {
        const DATA_BYTES_FLOWMETER: usize = 2 * (NUM_REGISTER_FLOWMETER as usize);
        const FRAME_LENGTH_FLOWMETER: usize = 5 + DATA_BYTES_FLOWMETER;
        if (frame.len() != FRAME_LENGTH_FLOWMETER) || (frame[2] != (DATA_BYTES_FLOWMETER as u8)) {
            return None;
        }

        let address = frame[0];

        let values = get_values_from_u8_array::<f32, 5>(&frame[3..(3 + DATA_BYTES_FLOWMETER)])?;

        Some(Flowmeter {
            address,
            signal_strength: values[0],
            flow_rate: values[1],
            net_totalizer: values[2],
            positive_totalizer: values[3],
            negative_totalizer: values[4],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_frame_invalid() {
        // Frame with incorrect length
        let frame_short: [u8; 24] = [0; 24];
        assert!(Flowmeter::from_frame(&frame_short).is_none());

        // Frame with incorrect data bytes
        let mut frame_wrong_data_bytes: [u8; 25] = [0; 25];
        frame_wrong_data_bytes[2] = 19;
        assert!(Flowmeter::from_frame(&frame_wrong_data_bytes).is_none());
    }
}
