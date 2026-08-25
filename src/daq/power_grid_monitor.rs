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

use crate::constants::{
    NUM_REGISTER_POWER_GRID_MONITOR, UNEXPECTED_VALUE_POWER_GRID_MONITOR_POWER_FACTOR,
};
use crate::utility::get_values_from_u8_array;

#[derive(Debug, PartialEq)]
pub struct PowerGridMonitor {
    // Address of the power grid monitor.
    pub address: u8,
    // Average current in amperes.
    pub current_avg: f32,
    // Average line-to-line voltage in volts.
    pub voltage_ll_avg: f32,
    // Average line-to-neutral voltage in volts.
    pub voltage_ln_avg: f32,
    // Total active power in kW.
    pub active_power_total: f32,
    // Total reactive power in kVAR.
    pub reactive_power_total: f32,
    // Total apparent power in kVA.
    pub apparent_power_total: f32,
    // Total power factor (unitless). Ratio between power and apparent power.
    pub power_factor_total: f32,
    // Frequency in Hz.
    pub frequency: f32,
}

impl PowerGridMonitor {
    /// Power grid monitor to have the measured electrical parameters.
    ///
    /// # Arguments
    /// * `address` - The address of the power grid monitor.
    ///
    /// # Returns
    /// A new instance of `PowerGridMonitor`.
    pub fn new(address: u8) -> Self {
        Self {
            address,

            current_avg: 0.0,
            voltage_ll_avg: 0.0,
            voltage_ln_avg: 0.0,
            active_power_total: 0.0,
            reactive_power_total: 0.0,
            apparent_power_total: 0.0,
            power_factor_total: 0.0,
            frequency: 0.0,
        }
    }

    /// Create a `PowerGridMonitor` instance from a Modbus frame.
    ///
    /// # Arguments
    /// * `frame` - The Modbus frame containing the power grid monitor data.
    ///
    /// # Returns
    /// An `Option` containing the `PowerGridMonitor` if the frame is valid, or
    /// `None` otherwise.
    pub fn from_frame(frame: &[u8]) -> Option<PowerGridMonitor> {
        const DATA_BYTES_POWER_GRID_MONITOR: usize = 2 * (NUM_REGISTER_POWER_GRID_MONITOR as usize);
        const FRAME_LENGTH_POWER_GRID_MONITOR: usize = 5 + DATA_BYTES_POWER_GRID_MONITOR;
        if (frame.len() != FRAME_LENGTH_POWER_GRID_MONITOR)
            || (frame[2] != (DATA_BYTES_POWER_GRID_MONITOR as u8))
        {
            return None;
        }

        let address = frame[0];

        // To calculate the index of frame, substract the register address from
        // the starting address (3000, see REGISTER_ADDRESS_POWER_GRID_MONITOR
        // in constants.rs), multiply by 2 (each register is 2 bytes), and add
        // 3 (the first 3 bytes of the Modbus frame: address, function code,
        // and byte count) to get the byte index.

        // Register address: 3010
        let current_avg = get_values_from_u8_array::<f32, 1>(&frame[23..27])?[0];

        // Register address: 3026
        let voltage_ll_avg = get_values_from_u8_array::<f32, 1>(&frame[55..59])?[0];

        // Register address: 3036
        let voltage_ln_avg = get_values_from_u8_array::<f32, 1>(&frame[75..79])?[0];

        // Register address: 3060
        let active_power_total = get_values_from_u8_array::<f32, 1>(&frame[123..127])?[0];

        // Register address: 3068
        let reactive_power_total = get_values_from_u8_array::<f32, 1>(&frame[139..143])?[0];

        // Register address: 3076
        let apparent_power_total = get_values_from_u8_array::<f32, 1>(&frame[155..159])?[0];

        // Register address: 3084
        let power_factor_total = get_values_from_u8_array::<f32, 1>(&frame[171..175])?[0];

        // Register address: 3110
        let frequency = get_values_from_u8_array::<f32, 1>(&frame[223..227])?[0];

        Some(PowerGridMonitor {
            address,
            current_avg,
            voltage_ll_avg,
            voltage_ln_avg,
            active_power_total,
            reactive_power_total,
            apparent_power_total,
            power_factor_total: PowerGridMonitor::calculate_power_factor(power_factor_total),
            frequency,
        })
    }

    /// Calculate the power factor (PF) from the register value.
    ///
    /// # Notes
    /// The meter also provides PF information (including sign and quadrant) in
    /// single floating-point registers for each of the PF values (for example,
    /// per-phase and total values for true and displacement PF, and associated
    /// minimums and maximums). The meter performs a simple algorithm to the PF
    /// value then stores it in the appropriate PF register.
    ///
    /// The PF value is calculated from the PF register value using the
    /// following formulas:
    ///
    /// Quadrant   | PF range | PF register range | PF formula
    /// Quadrant 1 | 0 to 1   |       0 to 1      | register_value
    /// Quadrant 2 | -1 to 0  |      -2 to -1     | -2 - register_value
    /// Quadrant 3 | 0 to -1  |      -1 to 0      | register_value
    /// Quadrant 4 | 1 to 0   |       1 to 2      | 2 - register_value
    ///
    /// # Arguments
    /// * `register_value` - The raw power factor register value.
    ///
    /// # Returns
    /// The calculated power factor.
    fn calculate_power_factor(register_value: f32) -> f32 {
        match register_value {
            // Quadrant 1
            x if (0.0..=1.0).contains(&x) => x,
            // Quadrant 2
            x if (-2.0..=-1.0).contains(&x) => -2.0 - x,
            // Quadrant 3
            x if (x > -1.0) && (x <= 0.0) => x,
            // Quadrant 4
            x if (x > 1.0) && (x <= 2.0) => 2.0 - x,
            // Fallback for unexpected values
            _ => UNEXPECTED_VALUE_POWER_GRID_MONITOR_POWER_FACTOR,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_frame_invalid() {
        // Frame with incorrect length
        let frame_short: [u8; 228] = [0; 228];
        assert!(PowerGridMonitor::from_frame(&frame_short).is_none());

        // Frame with incorrect data bytes
        let mut frame_wrong_data_bytes: [u8; 229] = [0; 229];
        frame_wrong_data_bytes[2] = 223;
        assert!(PowerGridMonitor::from_frame(&frame_wrong_data_bytes).is_none());
    }

    #[test]
    fn test_calculate_power_factor() {
        // Quadrant 1
        assert_eq!(PowerGridMonitor::calculate_power_factor(0.6), 0.6);
        assert_eq!(PowerGridMonitor::calculate_power_factor(0.0), 0.0);
        assert_eq!(PowerGridMonitor::calculate_power_factor(1.0), 1.0);

        // Quadrant 2
        assert_eq!(
            PowerGridMonitor::calculate_power_factor(-1.6),
            -2.0 - (-1.6)
        );
        assert_eq!(
            PowerGridMonitor::calculate_power_factor(-2.0),
            -2.0 - (-2.0)
        );

        // Quadrant 3
        assert_eq!(PowerGridMonitor::calculate_power_factor(-0.6), -0.6);
        assert_eq!(PowerGridMonitor::calculate_power_factor(-1.0), -1.0);

        // Quadrant 4
        assert_eq!(PowerGridMonitor::calculate_power_factor(1.6), 2.0 - 1.6);
        assert_eq!(PowerGridMonitor::calculate_power_factor(2.0), 2.0 - 2.0);

        // Unexpected value
        assert_eq!(
            PowerGridMonitor::calculate_power_factor(3.0),
            UNEXPECTED_VALUE_POWER_GRID_MONITOR_POWER_FACTOR
        );
    }
}
