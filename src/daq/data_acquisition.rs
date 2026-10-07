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

use crate::config::Config;
use crate::constants::{
    MAX_ANALOG_INPUT_OUTPUT, NUM_TANK, NUM_TEMPERATURE_CHANNEL, NUM_TEMPERATURE_HUB,
    NUMS_FLOWMETER, NUMS_PRESSURE_TRANSDUCER,
};
use crate::daq::{
    flowmeter::Flowmeter, modbus_communicator::ModbusCommunicator,
    power_grid_monitor::PowerGridMonitor, tank_level::TankLevel,
};
use crate::enums::{AnalogInput, AnalogOutput, DigitalOutput};
use crate::event_queue::EventQueue;
use crate::mock::mock_plant::MockPlant;
use crate::telemetry::telemetry_data_acquisition::TelemetryDataAcquisition;
use log::warn;

pub struct DataAcquisition {
    // Configuration
    pub config: Config,
    // Events to publish
    pub event_queue: EventQueue,
    // The latest telemetry data from the data acquisition system (DAQ)
    _latest_telemetry: TelemetryDataAcquisition,
    // Tank level sensors
    _tank_levels: Vec<TankLevel>,
    // Modbus communicator
    _modbus: ModbusCommunicator,
    // Plant model
    pub plant: Option<MockPlant>,
}

impl DataAcquisition {
    /// Create a new Data Acquisition instance. This communicates with the
    /// hardwares.
    ///
    /// # Arguments
    /// * `is_simulation_mode` - A boolean indicating whether to run in
    ///   simulation mode.
    ///
    /// # Returns
    /// A new instance of DataAcquisition.
    pub fn new(is_simulation_mode: bool) -> Self {
        // Tank level sensors
        let config = Config::new();
        let tank_levels = Self::get_tank_levels(&config);

        // Plant model
        let plant = if is_simulation_mode {
            Some(MockPlant::new())
        } else {
            None
        };

        Self {
            config,

            event_queue: EventQueue::new(),
            _latest_telemetry: TelemetryDataAcquisition::new(),

            _tank_levels: tank_levels,
            _modbus: ModbusCommunicator::new(),

            plant,
        }
    }

    /// Get the tank levels based on the configuration.
    ///
    /// # Arguments
    /// * `config` - The configuration containing tank level settings.
    ///
    /// # Returns
    /// A vector of `TankLevel` instances.
    fn get_tank_levels(config: &Config) -> Vec<TankLevel> {
        let tank_level_configuration = &config.tank_levels;
        let sensor_max_ranges = &tank_level_configuration["sensor_max_range"];
        let distance_fulls = &tank_level_configuration["distance_full"];
        let distance_empties = &tank_level_configuration["distance_empty"];

        sensor_max_ranges
            .iter()
            .zip(distance_fulls.iter().zip(distance_empties.iter()))
            .map(|(sensor_max_range, (distance_full, distance_empty))| {
                TankLevel::new(*sensor_max_range, *distance_full, *distance_empty)
            })
            .collect::<Vec<TankLevel>>()
    }

    /// Is the simulation mode or not.
    ///
    /// # Returns
    /// True if the simulation mode is enabled. Otherwise, false.
    pub fn is_simulation_mode(&self) -> bool {
        self.plant.is_some()
    }

    /// Switch the digital output.
    ///
    /// # Arguments
    /// * `digital_output` - The digital output to switch.
    /// * `switch_on` - `true` to switch on, `false` to switch off.
    ///
    /// # Returns
    /// Some if the digital output is switched successfully. Otherwise, None.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    pub fn switch_digital_output(
        &mut self,
        digital_output: DigitalOutput,
        switch_on: bool,
    ) -> Option<()> {
        if let Some(plant) = &mut self.plant {
            plant.switch_digital_output(digital_output, switch_on);

            Some(())
        } else {
            panic!("Not implemented yet.");
        }
    }

    /// Open the specified motor operated valve (MOV).
    ///
    /// # Arguments
    /// * `idx` - The index of the motor operated valve (0-4).
    /// * `switch_on` - `true` to open the valve, `false` to close it.
    ///
    /// # Returns
    /// Some if the valve is switched successfully. Otherwise, None.
    pub fn open_motor_operated_valve(&mut self, idx: usize, switch_on: bool) -> Option<()> {
        let digital_output = match idx {
            0 => DigitalOutput::OpenMov1,
            1 => DigitalOutput::OpenMov2,
            2 => DigitalOutput::OpenMov3,
            3 => DigitalOutput::OpenMov4,
            4 => DigitalOutput::OpenMov5,
            _ => return None,
        };

        self.switch_digital_output(digital_output, switch_on)
    }

    /// Open the specified pressure control valve (PCV) to a given percentage.
    ///
    /// # Arguments
    /// * `idx` - The index of the pressure control valve (0-1).
    /// * `percentage` - The percentage to open the valve (0.0 to 100.0). 0.0
    ///   means fully closed. 100.0 means fully open to allow all coolant goes
    ///   through it.
    ///
    /// # Returns
    /// Some if the valve is opened successfully. Otherwise, None.
    pub fn open_pressure_control_valve(&mut self, idx: usize, percentage: f64) -> Option<()> {
        self.open_valve(
            idx,
            &[
                AnalogOutput::CommandValuePcv1,
                AnalogOutput::CommandValuePcv2,
            ],
            percentage,
        )
    }

    /// Open the specified control mixing valve (CMV) to a given percentage.
    ///
    /// # Arguments
    /// * `idx` - The index of the control mixing valve (0-2).
    /// * `percentage` - The percentage to open the valve (0.0 to 100.0). 0.0
    ///   means fully closed (recirculating for the upper loop). 100.0 means
    ///   fully open to the return.
    ///
    /// # Returns
    /// Some if the valve is opened successfully. Otherwise, None.
    pub fn open_control_mixing_valve(&mut self, idx: usize, percentage: f64) -> Option<()> {
        self.open_valve(
            idx,
            &[
                AnalogOutput::CommandValueCmv1,
                AnalogOutput::CommandValueCmv2,
                AnalogOutput::CommandValueCmv20,
            ],
            percentage,
        )
    }

    /// Open the specified valve to a given percentage.
    ///
    /// # Arguments
    /// * `idx` - The index of the valve.
    /// * `analog_outputs` - The list of analog outputs corresponding to the
    ///   valves.
    /// * `percentage` - The percentage to open the valve (0.0 to 100.0). 0.0
    ///   means fully closed. 100.0 means fully open.
    ///
    /// # Returns
    /// Some if the valve is opened successfully. Otherwise, None.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    fn open_valve(
        &mut self,
        idx: usize,
        analog_outputs: &[AnalogOutput],
        percentage: f64,
    ) -> Option<()> {
        let analog_output = analog_outputs.get(idx)?;
        let value = percentage.clamp(0.0, 100.0) / 100.0 * MAX_ANALOG_INPUT_OUTPUT;
        if let Some(plant) = &mut self.plant {
            plant.write_analog_output(*analog_output, value);

            Some(())
        } else {
            panic!("Not implemented yet.");
        }
    }

    /// Get the telemetry data from the hardware.
    ///
    /// # Returns
    /// Telemetry data.
    pub fn get_telemetry(&mut self) -> TelemetryDataAcquisition {
        let mut telemetry = TelemetryDataAcquisition::new();

        // Read the temperatures
        telemetry.temperatures = self.read_temperatures();

        // Read the pressures
        telemetry.pressures = self.read_pressures();

        // Read the power grid monitors
        telemetry.power_grid_monitors = self.read_power_grid_monitors();

        // Read the flowmeters
        telemetry.flowmeters = self.read_flowmeters();

        // Read the pier fans

        // Read the recirculation pumps

        // Read the chillers

        // Read the PCVs
        telemetry.percentages_pcv = self.read_percentage_of_pressure_control_valves();

        // Read the CMVs
        telemetry.percentages_cmv = self.read_percentage_of_control_mixing_valves();

        // Read the tank levels
        let (distances, percentages) = self.read_percentage_of_tank_levels();
        telemetry
            .tank_levels
            .insert("distance".to_string(), distances);
        telemetry
            .tank_levels
            .insert("percentage".to_string(), percentages);

        //Read the digital inputs and outputs
        telemetry.digital_inputs_mod4 = self.get_digital_inputs_mod4();
        telemetry.digital_inputs_mod7 = self.get_digital_inputs_mod7();
        telemetry.digital_outputs = self.get_digital_outputs();

        // Cache the latest telemetry data
        self._latest_telemetry = telemetry;

        self._latest_telemetry.clone()
    }

    /// Read the current temperatures from the temperature hubs.
    ///
    /// # Returns
    /// A vector containing the temperatures in degrees Celsius.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    fn read_temperatures(&self) -> Vec<f32> {
        let mut temperatures = self._latest_telemetry.temperatures.clone();
        for idx in 0..NUM_TEMPERATURE_HUB {
            let mut frame_response = Vec::new();
            if let Some(plant) = &self.plant {
                if let Some(reading) = plant.request_sensor_temperatures(idx) {
                    frame_response = reading;
                } else {
                    // TODO: After defining the error code, send the related
                    // error code event.
                    warn!(
                        "Failed to read the temperature hub at index {}. Use the cached value instead.",
                        idx
                    );
                }
            } else {
                panic!("Not implemented yet.");
            }

            if let Some(values) = self._modbus.read_temperature_from_frame(&frame_response) {
                temperatures
                    [(idx * NUM_TEMPERATURE_CHANNEL)..((idx + 1) * NUM_TEMPERATURE_CHANNEL)]
                    .copy_from_slice(&values);
            }
        }

        temperatures
    }

    /// Read the current pressures from the pressure transducers.
    ///
    /// # Returns
    /// A vector containing the pressures in PSI.
    fn read_pressures(&self) -> Vec<f32> {
        let mut pressures = self._latest_telemetry.pressures.clone();

        let addresses_bus_0 = &self.config.addresses["pressure_transducer_bus_0"];
        let addresses_bus_1 = &self.config.addresses["pressure_transducer_bus_1"];
        let addresses_bus_2 = &self.config.addresses["pressure_transducer_bus_2"];

        let num_sensors = NUMS_PRESSURE_TRANSDUCER.iter().sum();
        const INDEX_END_PRESSURE_TRANSDUCER_BUS_0: usize = NUMS_PRESSURE_TRANSDUCER[0];
        const INDEX_END_PRESSURE_TRANSDUCER_BUS_1: usize =
            NUMS_PRESSURE_TRANSDUCER[0] + NUMS_PRESSURE_TRANSDUCER[1];
        for idx in 0..num_sensors {
            let bus;
            let address_request;
            match idx {
                0..INDEX_END_PRESSURE_TRANSDUCER_BUS_0 => {
                    bus = 0;
                    address_request = addresses_bus_0[idx];
                }
                INDEX_END_PRESSURE_TRANSDUCER_BUS_0..INDEX_END_PRESSURE_TRANSDUCER_BUS_1 => {
                    bus = 1;
                    address_request = addresses_bus_1[idx - INDEX_END_PRESSURE_TRANSDUCER_BUS_0];
                }
                _ => {
                    bus = 2;
                    address_request = addresses_bus_2[idx - INDEX_END_PRESSURE_TRANSDUCER_BUS_1];
                }
            }

            if let Some(pressure) = self.read_pressure(bus, address_request) {
                pressures[idx] = pressure;
            }
        }

        pressures
    }

    /// Read the current pressure from a specific pressure transducer.
    ///
    /// # Arguments
    /// * `bus` - The bus number where the pressure transducer is connected.
    /// * `address` - The address of the pressure transducer.
    ///
    /// # Returns
    /// The pressure in PSI if the reading is successful, otherwise `None`.
    ///
    /// # Panics
    /// Panics if the plant is not implemented.
    fn read_pressure(&self, bus: usize, address: u8) -> Option<f32> {
        let frame_request = self._modbus.create_frame_read_pressure(address);

        let frame_response;
        if let Some(plant) = &self.plant {
            if let Some(reading) = plant.request_sensor_pressure(bus, &frame_request) {
                frame_response = reading;
            } else {
                // TODO: After defining the error code, send the related
                // error code event.
                warn!(
                    "Failed to read the pressure transducer address {} at bus {}.",
                    address, bus
                );

                return None;
            }
        } else {
            panic!("Not implemented yet.");
        }

        if let Some((address_response, pressure)) =
            self._modbus.read_pressure_from_frame(&frame_response)
        {
            if address_response == address {
                return Some(pressure);
            } else {
                // TODO: After defining the error code, send the related
                // error code event.
                warn!(
                    "Pressure transducer address mismatch (expected: {}, got: {}).",
                    address, address_response
                );
            }
        }

        None
    }

    /// Read the current data from the power grid monitors.
    ///
    /// # Returns
    /// A vector containing the power grid monitor data.
    fn read_power_grid_monitors(&self) -> Vec<PowerGridMonitor> {
        let mut power_grid_monitors = Vec::new();

        let addresses = &self.config.addresses["power_grid_monitor"];
        for (idx, address) in addresses.iter().enumerate() {
            match self.read_power_grid_monitor(*address) {
                Some(power_grid_monitor) => {
                    power_grid_monitors.push(power_grid_monitor);
                }
                None => match self._latest_telemetry.power_grid_monitors.get(idx) {
                    Some(power_grid_monitor) => {
                        power_grid_monitors.push(power_grid_monitor.clone());
                    }
                    None => {
                        power_grid_monitors.push(PowerGridMonitor::new(0));
                    }
                },
            }
        }

        power_grid_monitors
    }

    /// Read the current data from a specific power grid monitor.
    ///
    /// # Arguments
    /// * `address` - The address to read.
    ///
    /// # Returns
    /// An `Option` containing the power grid monitor data if successful, or
    /// `None` if the read failed.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    fn read_power_grid_monitor(&self, address: u8) -> Option<PowerGridMonitor> {
        let frame_request = self._modbus.create_frame_read_power_grid_monitor(address);

        let frame_response;
        if let Some(plant) = &self.plant {
            if let Some(reading) = plant.request_power_grid_monitor(&frame_request) {
                frame_response = reading;
            } else {
                // TODO: After defining the error code, send the related
                // error code event.
                warn!("Failed to read the power grid monitor address {}.", address);

                return None;
            }
        } else {
            panic!("Not implemented yet.");
        }

        if let Some(power_grid_monitor) = self
            ._modbus
            .read_power_grid_monitor_from_frame(&frame_response)
        {
            let address_received = power_grid_monitor.address;
            if address_received == address {
                return Some(power_grid_monitor);
            } else {
                // TODO: After defining the error code, send the related
                // error code event.
                warn!(
                    "Power grid monitor address mismatch (expected: {}, got: {}).",
                    address, address_received
                );

                return None;
            }
        }

        None
    }

    /// Read the current data from the flowmeters.
    ///
    /// # Returns
    /// A vector containing the flowmeter data.
    fn read_flowmeters(&self) -> Vec<Flowmeter> {
        let mut flowmeters = Vec::new();

        let addresses_bus_0 = &self.config.addresses["flowmeter_bus_0"];
        let addresses_bus_1 = &self.config.addresses["flowmeter_bus_1"];
        let addresses_bus_2 = &self.config.addresses["flowmeter_bus_2"];

        let num_sensors = NUMS_FLOWMETER.iter().sum();
        const INDEX_END_FLOWMETER_BUS_0: usize = NUMS_FLOWMETER[0];
        const INDEX_END_FLOWMETER_BUS_1: usize = NUMS_FLOWMETER[0] + NUMS_FLOWMETER[1];
        for idx in 0..num_sensors {
            let bus;
            let address_request;
            match idx {
                0..INDEX_END_FLOWMETER_BUS_0 => {
                    bus = 0;
                    address_request = addresses_bus_0[idx];
                }
                INDEX_END_FLOWMETER_BUS_0..INDEX_END_FLOWMETER_BUS_1 => {
                    bus = 1;
                    address_request = addresses_bus_1[idx - INDEX_END_FLOWMETER_BUS_0];
                }
                _ => {
                    bus = 2;
                    address_request = addresses_bus_2[idx - INDEX_END_FLOWMETER_BUS_1];
                }
            }

            match self.read_flowmeter(bus, address_request) {
                Some(flowmeter) => {
                    flowmeters.push(flowmeter);
                }
                None => match self._latest_telemetry.flowmeters.get(idx) {
                    Some(power_grid_monitor) => {
                        flowmeters.push(power_grid_monitor.clone());
                    }
                    None => {
                        flowmeters.push(Flowmeter::new(0));
                    }
                },
            }
        }

        flowmeters
    }

    /// Read the current data of a specific flowmeter.
    ///
    /// # Arguments
    /// * `bus` - The bus number where the flowmeter is connected.
    /// * `address` - The address of the flowmeter on the bus.
    ///
    /// # Returns
    /// An `Option` containing the flowmeter data if the read was successful,
    /// or `None` if it failed.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    fn read_flowmeter(&self, bus: usize, address: u8) -> Option<Flowmeter> {
        let frame_request = self._modbus.create_frame_read_flowmeter(address);

        let frame_response;
        if let Some(plant) = &self.plant {
            if let Some(reading) = plant.request_sensor_flowmeter(bus, &frame_request) {
                frame_response = reading;
            } else {
                // TODO: After defining the error code, send the related
                // error code event.
                warn!(
                    "Failed to read the flowmeter address {} at bus {}.",
                    address, bus
                );

                return None;
            }
        } else {
            panic!("Not implemented yet.");
        }

        if let Some(flowmeter) = self._modbus.read_flowmeter_from_frame(&frame_response) {
            let address_received = flowmeter.address;
            if address_received == address {
                return Some(flowmeter);
            } else {
                // TODO: After defining the error code, send the related
                // error code event.
                warn!(
                    "Flowmeter address mismatch (expected: {}, got: {}).",
                    address, address_received
                );

                return None;
            }
        }

        None
    }

    /// Read the current percentage openings of all pressure control valves
    /// (PCVs).
    ///
    /// # Returns
    /// A vector containing the percentage openings of all PCVs.
    fn read_percentage_of_pressure_control_valves(&self) -> Vec<f64> {
        let mut percentages = self._latest_telemetry.percentages_pcv.clone();
        percentages.iter_mut().enumerate().for_each(|(idx, percentage)| {
            if let Some(value) = self.read_percentage_of_pressure_control_valve(idx) {
                *percentage = value;
            } else {
                // TODO: After defining the error code, send the related error
                // code event.
                warn!(
                    "Failed to read the percentage of pressure control valve at index {}. Use the cached value instead.",
                    idx
                );
            }
        });

        percentages
    }

    /// Read the current percentage opening of the specified pressure control
    /// valve (PCV).
    ///
    /// # Arguments
    /// * `idx` - The index of the pressure control valve (0-1).
    ///
    /// # Returns
    /// Some containing the percentage opening if successful. Otherwise, None.
    fn read_percentage_of_pressure_control_valve(&self, idx: usize) -> Option<f64> {
        self.read_percentage_of_analog_input(
            idx,
            &[AnalogInput::ReadoutPcv1, AnalogInput::ReadoutPcv2],
        )
    }

    /// Read the current percentage openings of all control mixing valves
    /// (CMVs).
    ///
    /// # Returns
    /// A vector containing the percentage openings of all CMVs.
    fn read_percentage_of_control_mixing_valves(&self) -> Vec<f64> {
        let mut percentages = self._latest_telemetry.percentages_cmv.clone();
        percentages.iter_mut().enumerate().for_each(|(idx, percentage)| {
            if let Some(value) = self.read_percentage_of_control_mixing_valve(idx) {
                *percentage = value;
            } else {
                // TODO: After defining the error code, send the related error
                // code event.
                warn!(
                    "Failed to read the percentage of control mixing valve at index {}. Use the cached value instead.",
                    idx
                );
            }
        });

        percentages
    }

    /// Read the current percentage opening of the specified control mixing
    /// valve (CMV).
    ///
    /// # Arguments
    /// * `idx` - The index of the control mixing valve (0-2).
    ///
    /// # Returns
    /// Some containing the percentage opening if successful. Otherwise, None.
    fn read_percentage_of_control_mixing_valve(&self, idx: usize) -> Option<f64> {
        self.read_percentage_of_analog_input(
            idx,
            &[
                AnalogInput::ReadoutCmv1,
                AnalogInput::ReadoutCmv2,
                AnalogInput::ReadoutCmv20,
            ],
        )
    }

    /// Read the all the current tank levels as the percentages and the
    /// distances to full.
    ///
    /// # Returns
    /// A tuple containing two vectors:
    /// - The first vector contains the distances to full (in millimeters) for
    ///   each tank.
    /// - The second vector contains the remaining percentages for each tank.
    fn read_percentage_of_tank_levels(&self) -> (Vec<f64>, Vec<f64>) {
        let mut distances = self._latest_telemetry.tank_levels["distance"].clone();
        let mut percentages = self._latest_telemetry.tank_levels["percentage"].clone();
        for idx in 0..NUM_TANK {
            if let Some((distance_to_full, remaining_percentage)) =
                self.read_percentage_of_tank_level(idx)
            {
                distances[idx] = distance_to_full;
                percentages[idx] = remaining_percentage;
            } else {
                // TODO: After defining the error code, send the related error
                // code event.
                warn!(
                    "Failed to read the tank level for tank {}. Use the cached value instead.",
                    idx
                );
            }
        }

        (distances, percentages)
    }

    /// Read the current tank level as a percentage and the distance to full.
    ///
    /// # Arguments
    /// * `idx` - The index of the tank (0-1).
    ///
    /// # Returns
    /// Some containing a tuple of the distance to full (in millimeters) and
    /// the remaining percentage if successful. Otherwise, None.
    fn read_percentage_of_tank_level(&self, idx: usize) -> Option<(f64, f64)> {
        let percentage = self.read_percentage_of_analog_input(
            idx,
            &[AnalogInput::ReadoutTank1, AnalogInput::ReadoutTank2],
        )?;

        let (distance_to_full, remaining_percentage) = self
            ._tank_levels
            .get(idx)?
            .calculate_distance_and_percentage(percentage / 100.0);
        Some((distance_to_full, remaining_percentage))
    }

    /// Read the current percentage of the analog input.
    ///
    /// # Arguments
    /// * `idx` - Index of the `analog_inputs`.
    /// * `analog_inputs` - The list of analog inputs corresponding to the
    ///   `idx`.
    ///
    /// # Returns
    /// Some containing the percentage of the analog input if successful.
    /// Otherwise, None.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    fn read_percentage_of_analog_input(
        &self,
        idx: usize,
        analog_inputs: &[AnalogInput],
    ) -> Option<f64> {
        let analog_input = analog_inputs.get(idx)?;

        let value;
        if let Some(plant) = &self.plant {
            value = plant.read_analog_input(*analog_input);
        } else {
            panic!("Not implemented yet.");
        }

        Some(value / MAX_ANALOG_INPUT_OUTPUT * 100.0)
    }

    /// Get the digital inputs (Mod4, NI-9425).
    ///
    /// # Returns
    /// The digital inputs.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    fn get_digital_inputs_mod4(&self) -> u32 {
        if let Some(plant) = &self.plant {
            plant.digital_inputs_mod4
        } else {
            panic!("Not implemented yet.");
        }
    }

    /// Get the digital inputs (Mod7, NI-9425).
    ///
    /// # Returns
    /// The digital inputs.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    fn get_digital_inputs_mod7(&self) -> u32 {
        if let Some(plant) = &self.plant {
            plant.digital_inputs_mod7
        } else {
            panic!("Not implemented yet.");
        }
    }

    /// Get the digital outputs (Mod6, NI-9476).
    ///
    /// # Returns
    /// The digital outputs.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    fn get_digital_outputs(&self) -> u32 {
        if let Some(plant) = &self.plant {
            plant.digital_outputs
        } else {
            panic!("Not implemented yet.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;

    use ts_control_utils::enums::BitEnum;

    use crate::constants::{NUM_CMV, NUM_PCV};
    use crate::enums::DigitalInputMod4;
    use crate::mock::mock_constants::{PLANT_PRESSURE, PLANT_TEMPERATURE};

    const EPSILON: f64 = 1e-6;

    #[test]
    fn test_is_simulation_mode() {
        let data_acquisition = DataAcquisition::new(true);
        assert!(data_acquisition.is_simulation_mode());

        let data_acquisition = DataAcquisition::new(false);
        assert!(!data_acquisition.is_simulation_mode());
    }

    #[test]
    fn test_switch_digital_output() {
        let mut data_acquisition = DataAcquisition::new(true);

        // Switch the digital output on
        let digital_output = DigitalOutput::OpenMov5;
        data_acquisition.switch_digital_output(digital_output, true);

        assert_eq!(
            data_acquisition.plant.as_ref().unwrap().digital_outputs,
            DigitalOutput::OpenMov5.bit_value()
        );

        // Switch the digital output off
        data_acquisition.switch_digital_output(digital_output, false);

        assert_eq!(data_acquisition.plant.as_ref().unwrap().digital_outputs, 0);
    }

    #[test]
    fn test_open_motor_operated_valve() {
        let mut data_acquisition = DataAcquisition::new(true);

        // Valid indices
        let digital_input_open_feedbacks = [
            DigitalInputMod4::OpenFeedbackMov1,
            DigitalInputMod4::OpenFeedbackMov2,
            DigitalInputMod4::OpenFeedbackMov3,
            DigitalInputMod4::OpenFeedbackMov4,
            DigitalInputMod4::OpenFeedbackMov5,
        ];
        let digital_input_close_feedbacks = [
            DigitalInputMod4::CloseFeedbackMov1,
            DigitalInputMod4::CloseFeedbackMov2,
            DigitalInputMod4::CloseFeedbackMov3,
            DigitalInputMod4::CloseFeedbackMov4,
            DigitalInputMod4::CloseFeedbackMov5,
        ];
        for idx in 0..5 {
            // Open the valve
            assert!(
                data_acquisition
                    .open_motor_operated_valve(idx, true)
                    .is_some()
            );
            assert_eq!(
                data_acquisition.plant.as_ref().unwrap().digital_outputs,
                DigitalOutput::OpenMov1.bit_value() << idx
            );
            assert!(
                data_acquisition.plant.as_ref().unwrap().digital_inputs_mod4
                    & digital_input_open_feedbacks[idx].bit_value()
                    != 0
            );
            assert!(
                data_acquisition.plant.as_ref().unwrap().digital_inputs_mod4
                    & digital_input_close_feedbacks[idx].bit_value()
                    == 0
            );

            // Close the valve
            assert!(
                data_acquisition
                    .open_motor_operated_valve(idx, false)
                    .is_some()
            );
            assert_eq!(data_acquisition.plant.as_ref().unwrap().digital_outputs, 0);
            assert!(
                data_acquisition.plant.as_ref().unwrap().digital_inputs_mod4
                    & digital_input_close_feedbacks[idx].bit_value()
                    != 0
            );
            assert!(
                data_acquisition.plant.as_ref().unwrap().digital_inputs_mod4
                    & digital_input_open_feedbacks[idx].bit_value()
                    == 0
            );
        }

        // Invalid index
        assert!(
            data_acquisition
                .open_motor_operated_valve(5, true)
                .is_none()
        );
    }

    #[test]
    fn test_open_pressure_control_valve_invalid_index() {
        let mut data_acquisition = DataAcquisition::new(true);

        assert!(
            data_acquisition
                .open_pressure_control_valve(2, 50.0)
                .is_none()
        );
    }

    #[test]
    fn test_open_control_mixing_valve_invalid_index() {
        let mut data_acquisition = DataAcquisition::new(true);

        assert!(
            data_acquisition
                .open_control_mixing_valve(3, 50.0)
                .is_none()
        );
    }

    #[test]
    fn test_get_telemetry() {
        let mut data_acquisition = DataAcquisition::new(true);
        let percentage = 54.0;
        for idx in 0..NUM_PCV {
            data_acquisition.open_pressure_control_valve(idx, percentage);
        }
        for idx in 0..NUM_CMV {
            data_acquisition.open_control_mixing_valve(idx, percentage);
        }

        data_acquisition.switch_digital_output(DigitalOutput::OpenMov1, true);
        data_acquisition.switch_digital_output(DigitalOutput::PowerChiller1, true);
        data_acquisition.switch_digital_output(DigitalOutput::PowerChiller2, true);
        data_acquisition.switch_digital_output(DigitalOutput::PowerPierFan1, true);
        data_acquisition.switch_digital_output(DigitalOutput::PowerPierFan2, true);
        data_acquisition.switch_digital_output(DigitalOutput::PowerRecirculationPump1, true);
        data_acquisition.switch_digital_output(DigitalOutput::PowerRecirculationPump2, true);

        // Check the telemetry
        let telemetry = data_acquisition.get_telemetry();

        assert_eq!(telemetry.temperatures[0], PLANT_TEMPERATURE);
        assert_eq!(telemetry.pressures[0], PLANT_PRESSURE);

        assert_eq!(telemetry.power_grid_monitors[0].address, 1);
        assert_eq!(telemetry.flowmeters[0].address, 1);

        assert_eq!(telemetry.percentages_pcv[0], percentage);
        assert_eq!(telemetry.percentages_cmv[0], percentage);

        assert_eq!(telemetry.tank_levels["distance"][0], 263.1773);

        assert_eq!(telemetry.digital_outputs, 2013266689);
        assert_eq!(telemetry.digital_inputs_mod4, 62121);
        assert_eq!(telemetry.digital_inputs_mod7, 3);

        assert_eq!(data_acquisition._latest_telemetry, telemetry);
    }

    #[test]
    fn test_read_temperatures() {
        let data_acquisition = DataAcquisition::new(true);

        let temperatures = data_acquisition.read_temperatures();

        assert_eq!(
            temperatures,
            vec![PLANT_TEMPERATURE; NUM_TEMPERATURE_HUB * NUM_TEMPERATURE_CHANNEL]
        );
    }

    #[test]
    fn test_read_pressures() {
        let data_acquisition = DataAcquisition::new(true);

        let pressures = data_acquisition.read_pressures();

        assert_eq!(
            pressures,
            vec![PLANT_PRESSURE; NUMS_PRESSURE_TRANSDUCER.iter().sum()]
        );
    }

    #[test]
    fn test_read_power_grid_monitors() {
        let data_acquisition = DataAcquisition::new(true);

        let power_grid_monitors = data_acquisition.read_power_grid_monitors();

        let addresses: Vec<u8> = power_grid_monitors
            .iter()
            .map(|monitor| monitor.address)
            .collect();
        assert_eq!(addresses, [1, 2, 3]);
    }

    #[test]
    fn test_read_flowmeters() {
        let data_acquisition = DataAcquisition::new(true);

        let flowmeters = data_acquisition.read_flowmeters();

        let addresses: Vec<u8> = flowmeters
            .iter()
            .map(|flowmeter| flowmeter.address)
            .collect();
        assert_eq!(addresses, [1, 3, 6, 20, 2, 4, 7, 21, 22, 23, 24, 25]);
    }

    #[test]
    fn test_read_percentage_of_pressure_control_valves() {
        let mut data_acquisition = DataAcquisition::new(true);

        let percentage = 54.0;
        for idx in 0..NUM_PCV {
            data_acquisition.open_pressure_control_valve(idx, percentage);
        }

        let percentages = data_acquisition.read_percentage_of_pressure_control_valves();
        for idx in 0..NUM_PCV {
            assert_eq!(percentages[idx], percentage);
        }
    }

    #[test]
    fn test_read_percentage_of_pressure_control_valve_invalid_index() {
        let data_acquisition = DataAcquisition::new(true);

        assert!(
            data_acquisition
                .read_percentage_of_pressure_control_valve(2)
                .is_none()
        );
    }

    #[test]
    fn test_read_percentage_of_control_mixing_valve_valid_index() {
        let data_acquisition = DataAcquisition::new(true);

        assert!(
            data_acquisition
                .read_percentage_of_control_mixing_valve(3)
                .is_none()
        );
    }

    #[test]
    fn test_open_pressure_control_valve_and_read_percentage() {
        let mut data_acquisition = DataAcquisition::new(true);

        let percentage = 54.0;
        for idx in 0..NUM_PCV {
            assert!(
                data_acquisition
                    .open_pressure_control_valve(idx, percentage)
                    .is_some()
            );
            assert_eq!(
                data_acquisition
                    .read_percentage_of_pressure_control_valve(idx)
                    .unwrap(),
                percentage
            );
        }
    }

    #[test]
    fn test_read_percentage_of_control_mixing_valves() {
        let mut data_acquisition = DataAcquisition::new(true);

        let percentage = 54.0;
        for idx in 0..NUM_CMV {
            data_acquisition.open_control_mixing_valve(idx, percentage);
        }

        let percentages = data_acquisition.read_percentage_of_control_mixing_valves();
        for idx in 0..NUM_CMV {
            assert_eq!(percentages[idx], percentage);
        }
    }

    #[test]
    fn test_open_control_mixing_valve_and_read_percentage() {
        let mut data_acquisition = DataAcquisition::new(true);

        let percentage = 54.0;
        for idx in 0..NUM_CMV {
            assert!(
                data_acquisition
                    .open_control_mixing_valve(idx, percentage)
                    .is_some()
            );
            assert_eq!(
                data_acquisition
                    .read_percentage_of_control_mixing_valve(idx)
                    .unwrap(),
                percentage
            );
        }
    }

    #[test]
    fn test_read_percentage_of_tank_levels() {
        let data_acquisition = DataAcquisition::new(true);

        let (distances, percentages) = data_acquisition.read_percentage_of_tank_levels();

        for idx in 0..NUM_TANK {
            assert_eq!(distances[idx], 263.1773);
            assert_relative_eq!(percentages[idx], 69.3979884, epsilon = EPSILON);
        }
    }

    #[test]
    fn test_read_percentage_of_tank_level() {
        let data_acquisition = DataAcquisition::new(true);

        for idx in 0..NUM_TANK {
            let (distance_to_full, remaining_percentage) =
                data_acquisition.read_percentage_of_tank_level(idx).unwrap();

            assert_eq!(distance_to_full, 263.1773);
            assert_relative_eq!(remaining_percentage, 69.3979884, epsilon = EPSILON);
        }
    }
}
