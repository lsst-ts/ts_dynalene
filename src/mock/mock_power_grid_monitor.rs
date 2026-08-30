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

use crc::{CRC_16_MODBUS, Crc};

use crate::constants::CODE_READ_HOLDING_REGISTERS;
use crate::daq::power_grid_monitor::PowerGridMonitor;
use crate::mock::mock_constants::{
    PLANT_POWER_GRID_MONITOR_ACTIVE_POWER_TOTAL, PLANT_POWER_GRID_MONITOR_APPARENT_POWER_TOTAL,
    PLANT_POWER_GRID_MONITOR_CURRENT_AVG, PLANT_POWER_GRID_MONITOR_FREQUENCY,
    PLANT_POWER_GRID_MONITOR_POWER_FACTOR_TOTAL, PLANT_POWER_GRID_MONITOR_REACTIVE_POWER_TOTAL,
    PLANT_POWER_GRID_MONITOR_VOLTAGE_LL_AVG, PLANT_POWER_GRID_MONITOR_VOLTAGE_LN_AVG,
};
use crate::utility::calculate_modbus_crc_and_update_frame;

pub struct MockPowerGridMonitor {
    // Cyclic redundancy check (CRC) calculator for the Modbus communication.
    _crc: Crc<u16>,
    // Power grid monitor.
    pub power_grid_monitor: PowerGridMonitor,
}

impl MockPowerGridMonitor {
    /// Mock power grid monitor to simulate the power grid measurements.
    ///
    /// # Arguments
    /// * `address` - The address of the grid monitor.
    ///
    /// # Returns
    /// A new instance of `MockPowerGridMonitor`.
    pub fn new(address: u8) -> Self {
        let mut power_grid_monitor = PowerGridMonitor::new(address);
        power_grid_monitor.current_avg = PLANT_POWER_GRID_MONITOR_CURRENT_AVG;
        power_grid_monitor.voltage_ll_avg = PLANT_POWER_GRID_MONITOR_VOLTAGE_LL_AVG;
        power_grid_monitor.voltage_ln_avg = PLANT_POWER_GRID_MONITOR_VOLTAGE_LN_AVG;
        power_grid_monitor.active_power_total = PLANT_POWER_GRID_MONITOR_ACTIVE_POWER_TOTAL;
        power_grid_monitor.reactive_power_total = PLANT_POWER_GRID_MONITOR_REACTIVE_POWER_TOTAL;
        power_grid_monitor.apparent_power_total = PLANT_POWER_GRID_MONITOR_APPARENT_POWER_TOTAL;
        power_grid_monitor.power_factor_total = PLANT_POWER_GRID_MONITOR_POWER_FACTOR_TOTAL;
        power_grid_monitor.frequency = PLANT_POWER_GRID_MONITOR_FREQUENCY;

        Self {
            _crc: Crc::<u16>::new(&CRC_16_MODBUS),

            power_grid_monitor,
        }
    }

    /// Request the specified number of registers from the power grid monitor.
    ///
    /// # Arguments
    /// * `num` - The number of registers to request.
    ///
    /// # Returns
    /// A vector containing the Modbus frame response.
    pub fn request(&self, num: u16) -> Vec<u8> {
        // Each register consists of 2 bytes.
        let data_bytes = num * 2;

        let mut frame_response = vec![0; 5 + (data_bytes as usize)];
        frame_response[0] = self.power_grid_monitor.address;
        frame_response[1] = CODE_READ_HOLDING_REGISTERS;
        frame_response[2] = data_bytes as u8;

        // For the indices, see the PowerGridMonitor.from_frame().
        if num >= 10 {
            frame_response[23..27]
                .copy_from_slice(&self.power_grid_monitor.current_avg.to_be_bytes());
        }

        if num >= 26 {
            frame_response[55..59]
                .copy_from_slice(&self.power_grid_monitor.voltage_ll_avg.to_be_bytes());
        }

        if num >= 36 {
            frame_response[75..79]
                .copy_from_slice(&self.power_grid_monitor.voltage_ln_avg.to_be_bytes());
        }

        if num >= 60 {
            frame_response[123..127]
                .copy_from_slice(&self.power_grid_monitor.active_power_total.to_be_bytes());
        }

        if num >= 68 {
            frame_response[139..143]
                .copy_from_slice(&self.power_grid_monitor.reactive_power_total.to_be_bytes());
        }

        if num >= 76 {
            frame_response[155..159]
                .copy_from_slice(&self.power_grid_monitor.apparent_power_total.to_be_bytes());
        }

        if num >= 84 {
            // Note we do not consider the quadrant here to simplify the mock
            // implementation.
            frame_response[171..175]
                .copy_from_slice(&self.power_grid_monitor.power_factor_total.to_be_bytes());
        }

        if num >= 110 {
            frame_response[223..227]
                .copy_from_slice(&self.power_grid_monitor.frequency.to_be_bytes());
        }

        calculate_modbus_crc_and_update_frame(&self._crc, &mut frame_response);

        frame_response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::constants::NUM_REGISTER_POWER_GRID_MONITOR;

    #[test]
    fn test_request() {
        let mock_monitor = MockPowerGridMonitor::new(3);

        let response = mock_monitor.request(NUM_REGISTER_POWER_GRID_MONITOR);

        assert_eq!(response.len(), 229);
        assert_eq!(response[0], 3);
        assert_eq!(response[1], CODE_READ_HOLDING_REGISTERS);
        assert_eq!(response[2], 224);

        let monitor = PowerGridMonitor::from_frame(&response).unwrap();

        assert_eq!(monitor, mock_monitor.power_grid_monitor);
    }
}
