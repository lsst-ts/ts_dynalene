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

use std::collections::HashMap;
use std::str::from_utf8;
use ts_control_utils::enums::BitEnum;

use crate::config::Config;
use crate::constants::{
    CODE_READ_HOLDING_REGISTERS, CODE_WRITE_SINGLE_REGISTER, NUM_TEMPERATURE_CHANNEL,
    NUM_TEMPERATURE_HUB, REGISTER_ADDRESS_CHILLER_TEMPERATURE_SETPOINT,
    REGISTER_ADDRESS_PIER_FAN_RESET,
};
use crate::enums::{AnalogInput, AnalogOutput, DigitalInputMod4, DigitalInputMod7, DigitalOutput};
use crate::mock::{
    mock_chiller::MockChiller,
    mock_constants::{PLANT_NUM_CHANNEL_ANALOG_INPUT_OUTPUT, PLANT_TANK_LEVEL_VOLTAGE},
    mock_flowmeter_group::MockFlowmeterGroup,
    mock_pier_fan::MockPierFan,
    mock_power_grid_monitor::MockPowerGridMonitor,
    mock_pressure_transducer_group::MockPressureTransducerGroup,
    mock_recirculation_pump::MockRecirculationPump,
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
    // Recirculation pumps.
    _recirculation_pumps: Vec<MockRecirculationPump>,
    // Chillers.
    _chillers: Vec<MockChiller>,
    // Digital inputs from Mod4, NI-9425. Each bit is defined in
    // `DigitalInputMod4` in enums.rs.
    pub digital_inputs_mod4: u32,
    // Analog inputs from Mod5, NI-9207. See the `AnalogInput` in enums.rs for
    // details.
    _analog_inputs: Vec<f64>,
    // Digital outputs to Mod6, NI-9476. Each bit is defined in
    // `DigitalOutput` in enums.rs.
    pub digital_outputs: u32,
    // Digital inputs from Mod7, NI-9425. Each bit is defined in
    // `DigitalInputMod7` in enums.rs.
    pub digital_inputs_mod7: u32,
    // Analog outputs to Mod8, NI-9264. See the `AnalogOutput` in enums.rs for
    // details.
    _analog_outputs: Vec<f64>,
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
            _recirculation_pumps: addresses["recirculation_pump"]
                .iter()
                .map(|address| MockRecirculationPump::new(*address))
                .collect(),
            _chillers: addresses["chiller"]
                .iter()
                .map(|address| MockChiller::new(*address))
                .collect(),

            _addresses: addresses,

            digital_inputs_mod4: Self::get_default_digital_inputs_mod4(),
            digital_inputs_mod7: 0,

            digital_outputs: 0,

            _analog_inputs: Self::get_default_analog_inputs(),
            _analog_outputs: vec![0.0; PLANT_NUM_CHANNEL_ANALOG_INPUT_OUTPUT],
        }
    }

    /// Get the default digital inputs for mod4.
    ///
    /// # Returns
    /// Default digital inputs.
    fn get_default_digital_inputs_mod4() -> u32 {
        let bits = [
            DigitalInputMod4::CloseFeedbackMov1,
            DigitalInputMod4::CloseFeedbackMov2,
            DigitalInputMod4::CloseFeedbackMov3,
            DigitalInputMod4::CloseFeedbackMov4,
            DigitalInputMod4::CloseFeedbackMov5,
        ];
        bits.iter().fold(0, |acc, x| acc | x.bit_value())
    }

    /// Get the default analog inputs for the mock plant.
    ///
    /// # Returns
    /// A vector containing the default analog input values.
    fn get_default_analog_inputs() -> Vec<f64> {
        let mut analog_inputs = vec![0.0; PLANT_NUM_CHANNEL_ANALOG_INPUT_OUTPUT];
        analog_inputs[AnalogInput::ReadoutTank1 as usize] = PLANT_TANK_LEVEL_VOLTAGE;
        analog_inputs[AnalogInput::ReadoutTank2 as usize] = PLANT_TANK_LEVEL_VOLTAGE;

        analog_inputs
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

    /// Request the recirculation pump data.
    ///
    /// # Arguments
    /// * `command` - The command to read the recirculation pump data.
    ///
    /// # Returns
    /// Payload containing the recirculation pump data if the command is valid.
    /// Otherwise, `None` is returned.
    pub fn request_recirculation_pump(&mut self, command: &[u8]) -> Option<Vec<u8>> {
        if let Some((address, num_registers)) = self.get_address_and_num_registers(command) {
            let idx = get_index_from_array(&self._addresses["recirculation_pump"], &address)?;
            let register_address = u16::from_be_bytes([command[2], command[3]]);

            Some(self._recirculation_pumps[idx].request(register_address, num_registers))
        } else {
            None
        }
    }

    /// Request the chiller measurement or change the setpoint.
    ///
    /// # Arguments
    /// * `idx` - The index of the chiller to request. Note that two chillers
    ///   have the same address because they are under the different TCP
    ///   connections. Therefore, we need specify the index to distinguish
    ///   them. The value should be 0 or 1.
    /// * `command` - The command to read the chiller measurement or change the
    ///   setpoint.
    ///
    /// # Returns
    /// Payload containing the chiller measurement or change the setpoint
    /// response if the command is valid. Otherwise, `None` is returned.
    pub fn request_chiller(&mut self, idx: usize, command: &[u8]) -> Option<Vec<u8>> {
        if let Some((_, num_registers_or_value)) = self.get_address_and_num_registers(command) {
            let function_code = command[1];
            let register_address = u16::from_be_bytes([command[2], command[3]]);
            match function_code {
                CODE_READ_HOLDING_REGISTERS => {
                    Some(self._chillers[idx].request(num_registers_or_value))
                }
                CODE_WRITE_SINGLE_REGISTER => {
                    if register_address == REGISTER_ADDRESS_CHILLER_TEMPERATURE_SETPOINT {
                        self._chillers[idx].set_temperature(num_registers_or_value);
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

    /// Switch the specified digital output on or off.
    ///
    /// # Arguments
    /// * `digital_output` - The digital output to switch.
    /// * `switch_on` - `true` to switch on, `false` to switch off.
    pub fn switch_digital_output(&mut self, digital_output: DigitalOutput, switch_on: bool) {
        if switch_on {
            self.digital_outputs |= digital_output.bit_value();
        } else {
            self.digital_outputs &= !digital_output.bit_value();
        }

        self.update_digital_inputs(digital_output, switch_on);
    }

    /// Update the digital inputs based on the state of the specified digital
    /// output.
    ///
    /// # Arguments
    /// * `digital_output` - The digital output to switch.
    /// * `switch_on` - `true` to switch on, `false` to switch off.
    fn update_digital_inputs(&mut self, digital_output: DigitalOutput, switch_on: bool) {
        match digital_output {
            DigitalOutput::OpenMov1 => {
                if switch_on {
                    self.digital_inputs_mod4 |= DigitalInputMod4::OpenFeedbackMov1.bit_value();
                    self.digital_inputs_mod4 &= !DigitalInputMod4::CloseFeedbackMov1.bit_value();
                } else {
                    self.digital_inputs_mod4 &= !DigitalInputMod4::OpenFeedbackMov1.bit_value();
                    self.digital_inputs_mod4 |= DigitalInputMod4::CloseFeedbackMov1.bit_value();
                }
            }
            DigitalOutput::OpenMov2 => {
                if switch_on {
                    self.digital_inputs_mod4 |= DigitalInputMod4::OpenFeedbackMov2.bit_value();
                    self.digital_inputs_mod4 &= !DigitalInputMod4::CloseFeedbackMov2.bit_value();
                } else {
                    self.digital_inputs_mod4 &= !DigitalInputMod4::OpenFeedbackMov2.bit_value();
                    self.digital_inputs_mod4 |= DigitalInputMod4::CloseFeedbackMov2.bit_value();
                }
            }
            DigitalOutput::OpenMov3 => {
                if switch_on {
                    self.digital_inputs_mod4 |= DigitalInputMod4::OpenFeedbackMov3.bit_value();
                    self.digital_inputs_mod4 &= !DigitalInputMod4::CloseFeedbackMov3.bit_value();
                } else {
                    self.digital_inputs_mod4 &= !DigitalInputMod4::OpenFeedbackMov3.bit_value();
                    self.digital_inputs_mod4 |= DigitalInputMod4::CloseFeedbackMov3.bit_value();
                }
            }
            DigitalOutput::OpenMov4 => {
                if switch_on {
                    self.digital_inputs_mod4 |= DigitalInputMod4::OpenFeedbackMov4.bit_value();
                    self.digital_inputs_mod4 &= !DigitalInputMod4::CloseFeedbackMov4.bit_value();
                } else {
                    self.digital_inputs_mod4 &= !DigitalInputMod4::OpenFeedbackMov4.bit_value();
                    self.digital_inputs_mod4 |= DigitalInputMod4::CloseFeedbackMov4.bit_value();
                }
            }
            DigitalOutput::OpenMov5 => {
                if switch_on {
                    self.digital_inputs_mod4 |= DigitalInputMod4::OpenFeedbackMov5.bit_value();
                    self.digital_inputs_mod4 &= !DigitalInputMod4::CloseFeedbackMov5.bit_value();
                } else {
                    self.digital_inputs_mod4 &= !DigitalInputMod4::OpenFeedbackMov5.bit_value();
                    self.digital_inputs_mod4 |= DigitalInputMod4::CloseFeedbackMov5.bit_value();
                }
            }
            DigitalOutput::PowerRecirculationPump1 => {
                if switch_on {
                    self.digital_inputs_mod4 |= DigitalInputMod4::StatusK1.bit_value();
                } else {
                    self.digital_inputs_mod4 &= !DigitalInputMod4::StatusK1.bit_value();
                }
            }

            DigitalOutput::PowerRecirculationPump2 => {
                if switch_on {
                    self.digital_inputs_mod4 |= DigitalInputMod4::StatusK2.bit_value();
                } else {
                    self.digital_inputs_mod4 &= !DigitalInputMod4::StatusK2.bit_value();
                }
            }

            DigitalOutput::PowerPierFan1 => {
                if switch_on {
                    self.digital_inputs_mod7 |= DigitalInputMod7::StatusK3.bit_value();
                } else {
                    self.digital_inputs_mod7 &= !DigitalInputMod7::StatusK3.bit_value();
                }
            }

            DigitalOutput::PowerPierFan2 => {
                if switch_on {
                    self.digital_inputs_mod7 |= DigitalInputMod7::StatusK4.bit_value();
                } else {
                    self.digital_inputs_mod7 &= !DigitalInputMod7::StatusK4.bit_value();
                }
            }

            DigitalOutput::PowerChiller1 => {
                if switch_on {
                    self.digital_inputs_mod4 |= DigitalInputMod4::StatusK21.bit_value();
                } else {
                    self.digital_inputs_mod4 &= !DigitalInputMod4::StatusK21.bit_value();
                }
            }

            DigitalOutput::PowerChiller2 => {
                if switch_on {
                    self.digital_inputs_mod4 |= DigitalInputMod4::StatusK22.bit_value();
                } else {
                    self.digital_inputs_mod4 &= !DigitalInputMod4::StatusK22.bit_value();
                }
            }
            _ => {}
        }
    }

    /// Write a value to the specified analog output.
    ///
    /// # Arguments
    /// * `analog_output` - The analog output to write to.
    /// * `value` - The value to write.
    pub fn write_analog_output(&mut self, analog_output: AnalogOutput, value: f64) {
        self._analog_outputs[analog_output as usize] = value;

        match analog_output {
            AnalogOutput::CommandValuePcv1 => {
                self._analog_inputs[AnalogInput::ReadoutPcv1 as usize] = value;
            }
            AnalogOutput::CommandValuePcv2 => {
                self._analog_inputs[AnalogInput::ReadoutPcv2 as usize] = value;
            }
            AnalogOutput::CommandValueCmv1 => {
                self._analog_inputs[AnalogInput::ReadoutCmv1 as usize] = value;
            }
            AnalogOutput::CommandValueCmv2 => {
                self._analog_inputs[AnalogInput::ReadoutCmv2 as usize] = value;
            }
            AnalogOutput::CommandValueCmv20 => {
                self._analog_inputs[AnalogInput::ReadoutCmv20 as usize] = value;
            }
            _ => {
                // Do nothing for the mock plant.
            }
        }
    }

    /// Read the value of the specified analog input.
    ///
    /// # Arguments
    /// * `analog_input` - The analog input to read from.
    ///
    /// # Returns
    /// The value of the specified analog input.
    pub fn read_analog_input(&self, analog_input: AnalogInput) -> f64 {
        self._analog_inputs[analog_input as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::constants::{
        BYTES_HOLDING_REGISTER, BYTES_RESPONSE_TEMPERATURE, NUM_BUS_PRESSURE_TRANSDUCER,
        NUM_REGISTER_FLOWMETER, NUM_REGISTER_POWER_GRID_MONITOR,
        NUM_REGISTER_RECIRCULATION_PUMP_CIM_CONFIGURATION, REGISTER_ADDRESS_FLOWMETER,
        REGISTER_ADDRESS_PIER_FAN_MAXIMUM_SPEED, REGISTER_ADDRESS_POWER_GRID_MONITOR,
        REGISTER_ADDRESS_RECIRCULATION_PUMP_CIM_CONFIGURATION,
    };
    use crate::daq::{
        flowmeter::Flowmeter, power_grid_monitor::PowerGridMonitor,
        recirculation_pump::RecirculationPump,
    };

    fn create_frame_read_holding_registers(
        address: u8,
        register_address: u16,
        num_register: u16,
    ) -> [u8; BYTES_HOLDING_REGISTER] {
        let mut frame = [0; BYTES_HOLDING_REGISTER];
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
    ) -> [u8; BYTES_HOLDING_REGISTER] {
        let mut frame = [0; BYTES_HOLDING_REGISTER];
        frame[0] = address;
        frame[1] = CODE_WRITE_SINGLE_REGISTER;
        frame[2..4].copy_from_slice(&register_address.to_be_bytes());
        frame[4..6].copy_from_slice(&value.to_be_bytes());
        // CRC verify code is not calculated for the mock frame.
        frame
    }

    #[test]
    fn test_new() {
        let plant = MockPlant::new();

        assert_eq!(plant.digital_inputs_mod4, 682);
        assert_eq!(
            plant._analog_inputs[AnalogInput::ReadoutTank1 as usize],
            PLANT_TANK_LEVEL_VOLTAGE
        );
        assert_eq!(
            plant._analog_inputs[AnalogInput::ReadoutTank2 as usize],
            PLANT_TANK_LEVEL_VOLTAGE
        );
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
    fn test_request_pier_fan_write_single_register() {
        let mut plant = MockPlant::new();
        plant._pier_fans[0].pier_fan.motor_status = 10;
        plant._pier_fans[0].pier_fan.warning = 10;

        let command = create_frame_write_single_register(12, REGISTER_ADDRESS_PIER_FAN_RESET, 1);

        let response = plant.request_pier_fan(&command).unwrap();

        assert_eq!(response, command);

        assert_eq!(plant._pier_fans[1].pier_fan.motor_status, 0);
        assert_eq!(plant._pier_fans[1].pier_fan.warning, 0);
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

    #[test]
    fn test_request_recirculation_pump() {
        let mut plant = MockPlant::new();

        // Valid to read the holding registers
        let command = create_frame_read_holding_registers(
            1,
            REGISTER_ADDRESS_RECIRCULATION_PUMP_CIM_CONFIGURATION,
            NUM_REGISTER_RECIRCULATION_PUMP_CIM_CONFIGURATION,
        );

        let response = plant.request_recirculation_pump(&command).unwrap();
        let recirculation_pump = RecirculationPump::from_frame(&response, &[], &[], &[]);

        assert_eq!(recirculation_pump.address, 1);
        assert_eq!(
            recirculation_pump
                .cim_configuration
                .unwrap()
                .actual_modbus_address,
            1
        );
    }

    #[test]
    fn test_request_chiller_write_single_register() {
        let mut plant = MockPlant::new();

        let command = create_frame_write_single_register(
            1,
            REGISTER_ADDRESS_CHILLER_TEMPERATURE_SETPOINT,
            13,
        );
        let response = plant.request_chiller(0, &command).unwrap();

        assert_eq!(response, command);

        assert_eq!(plant._chillers[0].chiller.temperature_setpoint, 13);
    }

    #[test]
    fn test_request_chiller_read_holding_registers() {
        let mut plant = MockPlant::new();

        // Valid to read the holding registers
        let mut command = create_frame_read_holding_registers(
            1,
            REGISTER_ADDRESS_CHILLER_TEMPERATURE_SETPOINT,
            1,
        );

        let response = plant.request_chiller(0, &command).unwrap();

        assert_eq!(response.len(), 7);
        assert_eq!(response[0], 1);

        // Invalid
        command[1] = 8;
        assert!(plant.request_chiller(0, &command).is_none());
    }

    #[test]
    fn test_switch_digital_output() {
        let mut plant = MockPlant::new();

        // First time to switch on the digital output
        plant.switch_digital_output(DigitalOutput::OpenMov5, true);
        assert_eq!(plant.digital_outputs, DigitalOutput::OpenMov5.bit_value());

        // Switch the same digital output on again to ensure it remains on
        plant.switch_digital_output(DigitalOutput::OpenMov5, true);
        assert_eq!(plant.digital_outputs, DigitalOutput::OpenMov5.bit_value());

        // Switch on another digital output to ensure multiple outputs can be
        // on simultaneously
        plant.switch_digital_output(DigitalOutput::PowerPierFan1, true);
        assert_eq!(
            plant.digital_outputs,
            DigitalOutput::OpenMov5.bit_value() + DigitalOutput::PowerPierFan1.bit_value()
        );
        assert!(plant.digital_inputs_mod7 & DigitalInputMod7::StatusK3.bit_value() != 0);

        // Switch off the first digital output to ensure it is turned off
        // correctly
        plant.switch_digital_output(DigitalOutput::OpenMov5, false);
        assert_eq!(
            plant.digital_outputs,
            DigitalOutput::PowerPierFan1.bit_value()
        );

        // Switch off a digital output that is already off to ensure no change
        // occurs
        plant.switch_digital_output(DigitalOutput::OpenMov5, false);
        assert_eq!(
            plant.digital_outputs,
            DigitalOutput::PowerPierFan1.bit_value()
        );

        // Switch off the last remaining digital output to ensure all outputs
        // are off
        plant.switch_digital_output(DigitalOutput::PowerPierFan1, false);
        assert_eq!(plant.digital_outputs, 0);
        assert!(plant.digital_inputs_mod7 & DigitalInputMod7::StatusK3.bit_value() == 0);
    }

    #[test]
    fn test_update_digital_inputs() {
        let mut plant = MockPlant::new();

        plant.update_digital_inputs(DigitalOutput::PowerRecirculationPump1, true);
        assert!(plant.digital_inputs_mod4 & DigitalInputMod4::StatusK1.bit_value() != 0);

        plant.update_digital_inputs(DigitalOutput::PowerRecirculationPump1, false);
        assert!(plant.digital_inputs_mod4 & DigitalInputMod4::StatusK1.bit_value() == 0);

        plant.update_digital_inputs(DigitalOutput::PowerRecirculationPump2, true);
        assert!(plant.digital_inputs_mod4 & DigitalInputMod4::StatusK2.bit_value() != 0);

        plant.update_digital_inputs(DigitalOutput::PowerRecirculationPump2, false);
        assert!(plant.digital_inputs_mod4 & DigitalInputMod4::StatusK2.bit_value() == 0);

        plant.update_digital_inputs(DigitalOutput::PowerPierFan1, true);
        assert!(plant.digital_inputs_mod7 & DigitalInputMod7::StatusK3.bit_value() != 0);

        plant.update_digital_inputs(DigitalOutput::PowerPierFan1, false);
        assert!(plant.digital_inputs_mod7 & DigitalInputMod7::StatusK3.bit_value() == 0);

        plant.update_digital_inputs(DigitalOutput::PowerPierFan2, true);
        assert!(plant.digital_inputs_mod7 & DigitalInputMod7::StatusK4.bit_value() != 0);

        plant.update_digital_inputs(DigitalOutput::PowerPierFan2, false);
        assert!(plant.digital_inputs_mod7 & DigitalInputMod7::StatusK4.bit_value() == 0);

        plant.update_digital_inputs(DigitalOutput::PowerChiller1, true);
        assert!(plant.digital_inputs_mod4 & DigitalInputMod4::StatusK21.bit_value() != 0);

        plant.update_digital_inputs(DigitalOutput::PowerChiller1, false);
        assert!(plant.digital_inputs_mod4 & DigitalInputMod4::StatusK21.bit_value() == 0);

        plant.update_digital_inputs(DigitalOutput::PowerChiller2, true);
        assert!(plant.digital_inputs_mod4 & DigitalInputMod4::StatusK22.bit_value() != 0);

        plant.update_digital_inputs(DigitalOutput::PowerChiller2, false);
        assert!(plant.digital_inputs_mod4 & DigitalInputMod4::StatusK22.bit_value() == 0);
    }

    #[test]
    fn test_write_analog_output() {
        let mut plant = MockPlant::new();

        let value = 55.0;
        plant.write_analog_output(AnalogOutput::CommandValueCmv2, value);

        assert_eq!(
            plant._analog_outputs[AnalogOutput::CommandValueCmv2 as usize],
            value
        );
        assert_eq!(
            plant._analog_inputs[AnalogInput::ReadoutCmv2 as usize],
            value
        );
    }

    #[test]
    fn test_read_analog_input() {
        let mut plant = MockPlant::new();

        let value = 42.0;
        plant._analog_inputs[AnalogInput::ReadoutPcv1 as usize] = value;

        assert_eq!(plant.read_analog_input(AnalogInput::ReadoutPcv1), value);
    }
}
