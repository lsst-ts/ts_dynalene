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

use std::collections::HashMap;
use std::str::from_utf8;

use crate::config::Config;
use crate::constants::{
    CODE_READ_HOLDING_REGISTERS, CODE_WRITE_SINGLE_REGISTER, NUM_TEMPERATURE_CHANNEL,
    NUM_TEMPERATURE_HUB, REGISTER_ADDRESS_PIER_FAN_RESET,
};
use crate::mock::{
    mock_flowmeter_group::MockFlowmeterGroup, mock_pier_fan::MockPierFan,
    mock_power_grid_monitor::MockPowerGridMonitor,
    mock_pressure_transducer_group::MockPressureTransducerGroup,
    mock_temperature_hub::MockTemperatureHub,
};
use crate::utility::get_index_from_array;

pub struct MockPlant {
    // Addresses for various devices on different buses.
    _addresses: HashMap<String, Vec<u8>>,
    // Sensor of the temperature hubs.
    _sensor_temperature_hubs: Vec<MockTemperatureHub>,
    // Groups of the pressure transducers.
    _sensor_pressure_transducer_groups: Vec<MockPressureTransducerGroup>,
    // Groups of the flowmeters.
    _sensor_flowmeter_groups: Vec<MockFlowmeterGroup>,
    // Power grid monitors.
    _sensor_power_grid_monitors: Vec<MockPowerGridMonitor>,
    // Pier fans.
    _pier_fans: Vec<MockPierFan>,
}

impl Default for MockPlant {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPlant {
    /// Mock plant model to support the simulation mode of the dynalene control
    /// system.
    ///
    /// # Returns
    /// A new instance of `MockPlant`.
    pub fn new() -> Self {
        let config = Config::new();
        let addresses = config.addresses;

        Self {
            _sensor_temperature_hubs: (0..NUM_TEMPERATURE_HUB)
                .map(|_| MockTemperatureHub::new())
                .collect(),
            _sensor_pressure_transducer_groups: vec![
                MockPressureTransducerGroup::new(&addresses["pressure_transducer_bus_0"]),
                MockPressureTransducerGroup::new(&addresses["pressure_transducer_bus_1"]),
                MockPressureTransducerGroup::new(&addresses["pressure_transducer_bus_2"]),
            ],
            _sensor_flowmeter_groups: vec![
                MockFlowmeterGroup::new(&addresses["flowmeter_bus_0"]),
                MockFlowmeterGroup::new(&addresses["flowmeter_bus_1"]),
                MockFlowmeterGroup::new(&addresses["flowmeter_bus_2"]),
            ],
            _sensor_power_grid_monitors: addresses["power_grid_monitor"]
                .iter()
                .map(|address| MockPowerGridMonitor::new(*address))
                .collect(),
            _pier_fans: addresses["pier_fan"]
                .iter()
                .map(|address| MockPierFan::new(*address))
                .collect(),

            _addresses: addresses,
        }
    }

    /// Set the sensor of temperatures in the mock plant.
    ///
    /// # Arguments
    /// * `idx` - The index of the temperature hub. This index must be less
    ///   than `NUM_TEMPERATURE_HUB`.
    /// * `temperatures` - An array of temperatures in degree Celsius. The
    ///   length of the array must be equal to `NUM_TEMPERATURE_CHANNEL`.
    pub fn set_sensor_temperatures(
        &mut self,
        idx: usize,
        temperatures: &[f32; NUM_TEMPERATURE_CHANNEL],
    ) {
        if idx < NUM_TEMPERATURE_HUB {
            self._sensor_temperature_hubs[idx].set_temperatures(temperatures);
        }
    }

    /// Request to read the measured temperatures from sensor.
    ///
    /// # Notes
    /// The temperatures in index 0 are:
    /// [TS01, TS02, TS03, TS04, TS05, TS06, TS07, TS08]
    ///
    /// The temperatures in index 1 are:
    /// [TS20, TS21, TS22, TS23, TS24, TS25, TS26, TS27]
    ///
    /// The temperatures in index 2 are:
    /// [TS30, TS31, TS32, TS33, None, TS40, TS41, TS42]
    ///
    /// See the svi_HMI.vi in dynalene_system LabVIEW project.
    ///
    /// # Arguments
    /// * `idx` - The index of the temperature hub. This index must be less
    ///   than `NUM_TEMPERATURE_HUB`.
    ///
    /// # Returns
    /// Payload containing the temperature measurements if the index is valid.
    /// Otherwise, `None` is returned.
    pub fn request_sensor_temperatures(&self, idx: usize) -> Option<Vec<u8>> {
        if idx < NUM_TEMPERATURE_HUB {
            Some(self._sensor_temperature_hubs[idx].request())
        } else {
            None
        }
    }

    /// Request to read the measured pressure from sensor.
    ///
    /// # Notes
    /// The format of the command is "#<address>P\r\n". The format of address
    /// is "%03d".
    ///
    /// See the svi_PS_Bus_Test.vi in dynalene_system LabVIEW project.
    ///
    /// # Arguments
    /// * `idx` - The index of the pressure transducer bus. This index must be
    ///   less than `NUM_BUS_PRESSURE_TRANSDUCER`.
    /// * `command` - The command to read the measured pressure.
    ///
    /// # Returns
    /// Payload containing the pressure measurement if the idx and command are
    /// valid. Otherwise, `None` is returned.
    pub fn request_sensor_pressure(&self, idx: usize, command: &[u8]) -> Option<Vec<u8>> {
        let address = from_utf8(command)
            .ok()?
            .trim_start_matches('#')
            .trim_end_matches("P\r\n")
            .parse::<u8>()
            .ok()?;

        match idx {
            0 => {
                let idx_sensor =
                    get_index_from_array(&self._addresses["pressure_transducer_bus_0"], &address)?;

                Some(self._sensor_pressure_transducer_groups[0].request(idx_sensor))
            }
            1 => {
                let idx_sensor =
                    get_index_from_array(&self._addresses["pressure_transducer_bus_1"], &address)?;

                Some(self._sensor_pressure_transducer_groups[1].request(idx_sensor))
            }
            2 => {
                let idx_sensor =
                    get_index_from_array(&self._addresses["pressure_transducer_bus_2"], &address)?;

                Some(self._sensor_pressure_transducer_groups[2].request(idx_sensor))
            }
            _ => None,
        }
    }

    /// Request the flowmeter measurement from the specified bus.
    ///
    /// # Arguments
    /// * `idx` - The index of the flowmeter bus.
    /// * `command` - The command to read the flowmeter measurement.
    ///
    /// # Returns
    /// Payload containing the flowmeter measurement if the idx and command are
    /// valid. Otherwise, `None` is returned.
    pub fn request_sensor_flowmeter(&self, idx: usize, command: &[u8]) -> Option<Vec<u8>> {
        if let Some((address, num_registers)) = self.get_address_and_num_registers(command) {
            match idx {
                0 => {
                    let idx_sensor =
                        get_index_from_array(&self._addresses["flowmeter_bus_0"], &address)?;

                    Some(self._sensor_flowmeter_groups[0].request(idx_sensor, num_registers))
                }
                1 => {
                    let idx_sensor =
                        get_index_from_array(&self._addresses["flowmeter_bus_1"], &address)?;

                    Some(self._sensor_flowmeter_groups[1].request(idx_sensor, num_registers))
                }
                2 => {
                    let idx_sensor =
                        get_index_from_array(&self._addresses["flowmeter_bus_2"], &address)?;

                    Some(self._sensor_flowmeter_groups[2].request(idx_sensor, num_registers))
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get the address and number of registers from the command of reading
    /// holding registers.
    ///
    /// # Arguments
    /// * `command` - The command to extract the address and number of
    ///   registers from.
    ///
    /// # Returns
    /// A tuple containing the address and the number of registers.
    fn get_address_and_num_registers(&self, command: &[u8]) -> Option<(u8, u16)> {
        // Bytes of the command should be 8.
        if command.len() != 8 {
            return None;
        }

        Some((command[0], u16::from_be_bytes([command[4], command[5]])))
    }

    /// Request the power grid monitor measurement.
    ///
    /// # Arguments
    /// * `command` - The command to read the power grid monitor measurement.
    ///
    /// # Returns
    /// Payload containing the power grid monitor measurement if the command is
    /// valid. Otherwise, `None` is returned.
    pub fn request_power_grid_monitor(&self, command: &[u8]) -> Option<Vec<u8>> {
        if let Some((address, num_registers)) = self.get_address_and_num_registers(command) {
            let idx = get_index_from_array(&self._addresses["power_grid_monitor"], &address)?;

            Some(self._sensor_power_grid_monitors[idx].request(num_registers))
        } else {
            None
        }
    }

    /// Request the pier fan measurement or reset the status.
    ///
    /// # Arguments
    /// * `command` - The command to read the pier fan measurement or reset the
    ///   status.
    ///
    /// # Returns
    /// Payload containing the pier fan measurement or reset response if the
    /// command is valid. Otherwise, `None` is returned.
    pub fn request_pier_fan(&mut self, command: &[u8]) -> Option<Vec<u8>> {
        if let Some((address, num_registers)) = self.get_address_and_num_registers(command) {
            let idx = get_index_from_array(&self._addresses["pier_fan"], &address)?;
            let function_code = command[1];
            let register_address = u16::from_be_bytes([command[2], command[3]]);
            match function_code {
                CODE_READ_HOLDING_REGISTERS => {
                    Some(self._pier_fans[idx].request(register_address, num_registers))
                }
                CODE_WRITE_SINGLE_REGISTER => {
                    if register_address == REGISTER_ADDRESS_PIER_FAN_RESET {
                        self._pier_fans[idx].reset();
                        // Echo the command back as the response.
                        Some(command.to_vec())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::constants::{
        BYTES_RESPONSE_TEMPERATURE, NUM_BUS_PRESSURE_TRANSDUCER, NUM_REGISTER_FLOWMETER,
        NUM_REGISTER_POWER_GRID_MONITOR, REGISTER_ADDRESS_FLOWMETER,
        REGISTER_ADDRESS_PIER_FAN_MAXIMUM_SPEED, REGISTER_ADDRESS_POWER_GRID_MONITOR,
    };
    use crate::daq::{flowmeter::Flowmeter, power_grid_monitor::PowerGridMonitor};

    fn create_frame_read_holding_registers(
        address: u8,
        register_address: u16,
        num_register: u16,
    ) -> [u8; 8] {
        let mut frame = [0; 8];
        frame[0] = address;
        frame[1] = CODE_READ_HOLDING_REGISTERS;
        frame[2..4].copy_from_slice(&register_address.to_be_bytes());
        frame[4..6].copy_from_slice(&num_register.to_be_bytes());
        // CRC verify code is not calculated for the mock frame.
        frame
    }

    fn create_frame_write_single_register(
        address: u8,
        register_address: u16,
        value: u16,
    ) -> [u8; 8] {
        let mut frame = [0; 8];
        frame[0] = address;
        frame[1] = CODE_WRITE_SINGLE_REGISTER;
        frame[2..4].copy_from_slice(&register_address.to_be_bytes());
        frame[4..6].copy_from_slice(&value.to_be_bytes());
        // CRC verify code is not calculated for the mock frame.
        frame
    }

    #[test]
    fn test_set_sensor_temperatures() {
        let mut plant = MockPlant::new();

        let temperatures = [20.03; NUM_TEMPERATURE_CHANNEL];
        for idx in 0..NUM_TEMPERATURE_HUB {
            plant.set_sensor_temperatures(idx, &temperatures);

            assert_eq!(
                plant._sensor_temperature_hubs[idx].get_temperatures(),
                temperatures
            );
        }
    }

    #[test]
    fn test_request_sensor_temperatures() {
        let plant = MockPlant::new();

        // Valid
        for idx in 0..NUM_TEMPERATURE_HUB {
            assert_eq!(
                plant.request_sensor_temperatures(idx).unwrap().len(),
                BYTES_RESPONSE_TEMPERATURE
            );
        }

        // Invalid
        assert!(
            plant
                .request_sensor_temperatures(NUM_TEMPERATURE_HUB)
                .is_none()
        )
    }

    #[test]
    fn test_request_sensor_pressure() {
        let plant = MockPlant::new();

        // Valid
        assert_eq!(
            from_utf8(
                &plant
                    .request_sensor_pressure(0, &"#005P\r\n".as_bytes())
                    .unwrap()
            )
            .unwrap(),
            "@005 014.700 PSI G\r\n"
        );
        assert_eq!(
            from_utf8(
                &plant
                    .request_sensor_pressure(0, &"#020P\r\n".as_bytes())
                    .unwrap()
            )
            .unwrap(),
            "@020 014.700 PSI G\r\n"
        );
        assert_eq!(
            from_utf8(
                &plant
                    .request_sensor_pressure(2, &"#004P\r\n".as_bytes())
                    .unwrap()
            )
            .unwrap(),
            "@004 014.700 PSI G\r\n"
        );

        // Invalid
        assert!(
            plant
                .request_sensor_pressure(NUM_BUS_PRESSURE_TRANSDUCER, &"005P\r\n".as_bytes())
                .is_none()
        );
        assert!(
            plant
                .request_sensor_pressure(NUM_BUS_PRESSURE_TRANSDUCER, &"#005\r\n".as_bytes())
                .is_none()
        );

        assert!(
            plant
                .request_sensor_pressure(NUM_BUS_PRESSURE_TRANSDUCER, &"#000P\r\n".as_bytes())
                .is_none()
        );
        assert!(
            plant
                .request_sensor_pressure(0, &"#999P\r\n".as_bytes())
                .is_none()
        );
    }

    #[test]
    fn test_request_sensor_flowmeter() {
        let plant = MockPlant::new();

        // Valid
        let mut command = create_frame_read_holding_registers(
            7,
            REGISTER_ADDRESS_FLOWMETER,
            NUM_REGISTER_FLOWMETER,
        );

        let response = plant.request_sensor_flowmeter(1, &command).unwrap();
        let flowmeter = Flowmeter::from_frame(&response).unwrap();

        assert_eq!(flowmeter.address, 7);

        // Invalid
        assert!(plant.request_sensor_flowmeter(3, &command).is_none());

        command[0] = 8;
        assert!(plant.request_sensor_flowmeter(1, &command).is_none());
    }

    #[test]
    fn test_request_power_grid_monitor() {
        let plant = MockPlant::new();

        // Valid
        let mut command = create_frame_read_holding_registers(
            2,
            REGISTER_ADDRESS_POWER_GRID_MONITOR,
            NUM_REGISTER_POWER_GRID_MONITOR,
        );

        let response = plant.request_power_grid_monitor(&command).unwrap();
        let power_grid_monitor = PowerGridMonitor::from_frame(&response).unwrap();

        assert_eq!(power_grid_monitor.address, 2);

        // Invalid
        command[0] = 8;
        assert!(plant.request_power_grid_monitor(&command).is_none());
    }

    #[test]
    fn test_request_pier_fan_write_holding_registers() {
        let mut plant = MockPlant::new();
        plant._pier_fans[0].pier_fan.motor_status = 10;
        plant._pier_fans[0].pier_fan.warning = 10;

        let command = create_frame_write_single_register(12, REGISTER_ADDRESS_PIER_FAN_RESET, 1);

        let response = plant.request_pier_fan(&command).unwrap();

        assert_eq!(response, command);

        assert_eq!(plant._pier_fans[0].pier_fan.motor_status, 0);
        assert_eq!(plant._pier_fans[0].pier_fan.warning, 0);
    }

    #[test]
    fn test_request_pier_fan_read_holding_registers() {
        let mut plant = MockPlant::new();

        // Valid to read the holding registers
        let mut command =
            create_frame_read_holding_registers(12, REGISTER_ADDRESS_PIER_FAN_MAXIMUM_SPEED, 1);

        let response = plant.request_pier_fan(&command).unwrap();

        assert_eq!(response.len(), 7);
        assert_eq!(response[0], 12);

        // Invalid
        command[0] = 8;
        assert!(plant.request_pier_fan(&command).is_none());
    }
}
