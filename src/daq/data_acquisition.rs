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

use log::warn;
use ts_control_utils::enums::BitEnum;

use crate::config::Config;
use crate::constants::{
    MAX_ANALOG_INPUT_OUTPUT, NUM_TANK, NUM_TEMPERATURE_CHANNEL, NUM_TEMPERATURE_HUB,
    NUMS_FLOWMETER, NUMS_PRESSURE_TRANSDUCER,
};
use crate::daq::{
    chiller::Chiller, flowmeter::Flowmeter, modbus_communicator::ModbusCommunicator,
    pier_fan::PierFan, power_grid_monitor::PowerGridMonitor, recirculation_pump::RecirculationPump,
    tank_level::TankLevel,
};
use crate::enums::{AnalogInput, AnalogOutput, DigitalInputMod4, DigitalInputMod7, DigitalOutput};
use crate::event_queue::EventQueue;
use crate::mock::mock_plant::MockPlant;
use crate::telemetry::telemetry_data_acquisition::TelemetryDataAcquisition;
use crate::utility::celsius_to_fahrenheit;

pub struct DataAcquisition {
    // Configuration
    pub config: Config,
    // Events to publish
    pub event_queue: EventQueue,
    // Latest telemetry data
    _latest_telemetry: TelemetryDataAcquisition,
    // Tank level sensors
    _tank_levels: Vec<TankLevel>,
    // Modbus communicator
    _modbus: ModbusCommunicator,
    // The transaction ID for the Modbus TCP communication
    _transaction_id: u16,
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
    /// A new instance of `DataAcquisition`.
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
            _transaction_id: 0,

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
    /// Ok if the digital output is switched successfully. Otherwise, Err with
    /// a failure reason.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    pub fn switch_digital_output(
        &mut self,
        digital_output: DigitalOutput,
        switch_on: bool,
    ) -> Result<(), String> {
        if let Some(plant) = &mut self.plant {
            plant.switch_digital_output(digital_output, switch_on);

            Ok(())
        } else {
            panic!("Not implemented yet.");
        }
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
    /// Ok if the valve is opened successfully. Otherwise, Err with a failure
    /// reason.
    pub fn open_pressure_control_valve(
        &mut self,
        idx: usize,
        percentage: f64,
    ) -> Result<(), String> {
        if !self.is_valid_percentage(percentage) {
            return Err(String::from("Percentage is not in [0, 100]."));
        }

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
    /// Ok if the valve is opened successfully. Otherwise, Err with a failure
    /// reason.
    pub fn open_control_mixing_valve(&mut self, idx: usize, percentage: f64) -> Result<(), String> {
        if !self.is_valid_percentage(percentage) {
            return Err(String::from("Percentage is not in [0, 100]."));
        }

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

    /// Check if the given percentage is valid (0.0 to 100.0).
    ///
    /// # Arguments
    /// * `percentage` - The percentage to check.
    ///
    /// # Returns
    /// True if the percentage is valid. Otherwise, false.
    fn is_valid_percentage(&self, percentage: f64) -> bool {
        (0.0..=100.0).contains(&percentage)
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
    /// Ok if the valve is opened successfully. Otherwise, Err with a failure
    /// reason.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    fn open_valve(
        &mut self,
        idx: usize,
        analog_outputs: &[AnalogOutput],
        percentage: f64,
    ) -> Result<(), String> {
        let analog_output = analog_outputs
            .get(idx)
            .ok_or(String::from("Invalid valve index."))?;
        let value = percentage.clamp(0.0, 100.0) / 100.0 * MAX_ANALOG_INPUT_OUTPUT;
        if let Some(plant) = &mut self.plant {
            plant.write_analog_output(*analog_output, value);

            Ok(())
        } else {
            panic!("Not implemented yet.");
        }
    }

    /// Reset the specified pier fan.
    ///
    /// # Arguments
    /// * `idx` - The index of the pier fan (0-1).
    ///
    /// # Returns
    /// Ok if the pier fan is reset successfully. Otherwise, Err with a failure
    /// reason.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    pub fn reset_pier_fan(&mut self, idx: usize) -> Result<(), String> {
        let address = *self.config.addresses["pier_fan"]
            .get(idx)
            .ok_or(String::from("Invalid pier fan index."))?;
        if self.is_enabled_pier_fan(idx, self._latest_telemetry.digital_inputs_mod7) {
            let frame_request = self._modbus.create_frame_pier_fan_reset(address);

            let frame_response;
            if let Some(plant) = &mut self.plant {
                if let Some(reading) = plant.request_pier_fan(&frame_request) {
                    frame_response = reading;
                } else {
                    return Err(String::from("Pier fan request failed."));
                }
            } else {
                panic!("Not implemented yet.");
            }

            if frame_response != frame_request {
                return Err(String::from(
                    "Pier fan reset failed: frame response does not match request.",
                ));
            }
        }

        Ok(())
    }

    /// Check if the specified pier fan is enabled.
    ///
    /// # Arguments
    /// * `idx` - The index of the pier fan (0-1).
    /// * `digital_inputs_mod7` - The digital inputs from Mod7.
    ///
    /// # Returns
    /// True if the pier fan is enabled, otherwise false.
    fn is_enabled_pier_fan(&self, idx: usize, digital_inputs_mod7: u32) -> bool {
        match idx {
            0 => (digital_inputs_mod7 & DigitalInputMod7::StatusK3.bit_value()) != 0,
            1 => (digital_inputs_mod7 & DigitalInputMod7::StatusK4.bit_value()) != 0,
            _ => false,
        }
    }

    /// Set the temperature of the specified chiller.
    ///
    /// # Arguments
    /// * `idx` - The index of the chiller (0-1).
    /// * `temperature` - The desired temperature (-17.78 to 121.11) in
    ///   Celsius. The range is 0 to 250 Fahrenheit after conversion in the
    ///   instrument.
    ///
    /// # Returns
    /// Ok if the chiller temperature is set successfully. Otherwise, Err with
    /// a failure reason.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    pub fn set_chiller_temperature(&mut self, idx: usize, temperature: f32) -> Result<(), String> {
        // Change the temperature from Celsius to Fahrenheit and validate the
        // setpoint.
        let temperature_setpoint = celsius_to_fahrenheit(temperature) as u16;
        if temperature_setpoint > 250 {
            return Err(String::from(
                "Chiller temperature setpoint in Fahrenheit is out of range: [0, 250].",
            ));
        }

        // Set the chiller temperature if the chiller is enabled.
        let unit_id = *self.config.addresses["chiller"]
            .get(idx)
            .ok_or(String::from("Invalid chiller index."))?;
        if self.is_enabled_chiller(idx, self._latest_telemetry.digital_inputs_mod4) {
            let transaction_id = self.get_transaction_id();
            let frame_request = self._modbus.create_frame_chiller_set_temperature(
                transaction_id,
                unit_id,
                temperature_setpoint,
            );

            let frame_response;
            if let Some(plant) = &mut self.plant {
                if let Some(reading) = plant.request_chiller(idx, &frame_request) {
                    frame_response = reading;
                } else {
                    return Err(String::from("Chiller request failed."));
                }
            } else {
                panic!("Not implemented yet.");
            }

            if frame_response != frame_request {
                return Err(String::from(
                    "Chiller set temperature failed: frame response does not match request.",
                ));
            }
        } else {
            return Err(String::from("Chiller is not enabled."));
        }

        Ok(())
    }

    /// Check if the specified chiller is enabled.
    ///
    /// # Arguments
    /// * `idx` - The index of the chiller (0-1).
    /// * `digital_inputs_mod4` - The digital inputs from Mod4.
    ///
    /// # Returns
    /// True if the chiller is enabled, otherwise false.
    fn is_enabled_chiller(&self, idx: usize, digital_inputs_mod4: u32) -> bool {
        match idx {
            0 => (digital_inputs_mod4 & DigitalInputMod4::StatusK21.bit_value()) != 0,
            1 => (digital_inputs_mod4 & DigitalInputMod4::StatusK22.bit_value()) != 0,
            _ => false,
        }
    }

    /// Get the transaction ID for the Modbus TCP communication.
    ///
    /// # Notes
    /// The transaction ID is incremented internally and wraps around at
    /// `u16::MAX`.
    ///
    /// # Returns
    /// The transaction ID.
    fn get_transaction_id(&mut self) -> u16 {
        let transaction_id = self._transaction_id;

        if self._transaction_id == u16::MAX {
            self._transaction_id = 0;
        } else {
            self._transaction_id += 1;
        }

        transaction_id
    }

    /// Get the telemetry data from the hardware.
    ///
    /// # Returns
    /// Telemetry data.
    pub fn get_telemetry(&mut self) -> TelemetryDataAcquisition {
        let mut telemetry = TelemetryDataAcquisition::new();

        // Read the digital inputs and outputs
        telemetry.digital_inputs_mod4 = self.read_digital_inputs_mod4();
        telemetry.digital_inputs_mod7 = self.read_digital_inputs_mod7();
        telemetry.digital_outputs = self.read_digital_outputs();

        // Read the temperatures
        telemetry.temperatures = self.read_temperatures();

        // Read the pressures
        telemetry.pressures = self.read_pressures();

        // Read the power grid monitors
        telemetry.power_grid_monitors = self.read_power_grid_monitors();

        // Read the flowmeters
        telemetry.flowmeters = self.read_flowmeters();

        // Read the pier fans
        telemetry.pier_fans = self.read_pier_fans(telemetry.digital_inputs_mod7);

        // Read the recirculation pumps
        telemetry.recirculation_pumps =
            self.read_recirculation_pumps(telemetry.digital_inputs_mod4);

        // Read the chillers
        let transaction_id = self.get_transaction_id();
        telemetry.chillers = self.read_chillers(telemetry.digital_inputs_mod4, transaction_id);

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

        // Cache the latest telemetry data
        self._latest_telemetry = telemetry;

        self._latest_telemetry.clone()
    }

    /// Read the digital inputs (Mod4, NI-9425).
    ///
    /// # Returns
    /// The digital inputs.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    fn read_digital_inputs_mod4(&self) -> u32 {
        if let Some(plant) = &self.plant {
            plant.digital_inputs_mod4
        } else {
            panic!("Not implemented yet.");
        }
    }

    /// Read the digital inputs (Mod7, NI-9425).
    ///
    /// # Returns
    /// The digital inputs.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    fn read_digital_inputs_mod7(&self) -> u32 {
        if let Some(plant) = &self.plant {
            plant.digital_inputs_mod7
        } else {
            panic!("Not implemented yet.");
        }
    }

    /// Read the digital outputs (Mod6, NI-9476).
    ///
    /// # Returns
    /// The digital outputs.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    fn read_digital_outputs(&self) -> u32 {
        if let Some(plant) = &self.plant {
            plant.digital_outputs
        } else {
            panic!("Not implemented yet.");
        }
    }

    /// Read the current temperatures from the temperature hubs.
    ///
    /// # Returns
    /// A vector containing the temperatures in degrees Celsius.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    fn read_temperatures(&self) -> Vec<f64> {
        let mut temperatures = self._latest_telemetry.temperatures.clone();
        for idx in 0..NUM_TEMPERATURE_HUB {
            let mut frame_response = Vec::new();
            if let Some(plant) = &self.plant {
                if let Some(reading) = plant.request_sensor_temperatures(idx) {
                    frame_response = reading;
                }
            } else {
                panic!("Not implemented yet.");
            }

            if let Some(values) = self._modbus.read_temperature_from_frame(&frame_response) {
                let values_f64: Vec<f64> = values.iter().map(|value| *value as f64).collect();
                temperatures
                    [(idx * NUM_TEMPERATURE_CHANNEL)..((idx + 1) * NUM_TEMPERATURE_CHANNEL)]
                    .copy_from_slice(&values_f64);
            } else {
                // TODO: After defining the error code, send the related error
                // code event.
                warn!(
                    "Failed to read the temperature hub at index {}. Use the cached value instead.",
                    idx
                );
            }
        }

        temperatures
    }

    /// Read the current pressures from the pressure transducers.
    ///
    /// # Returns
    /// A vector containing the pressures in PSI.
    fn read_pressures(&self) -> Vec<f64> {
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
                pressures[idx] = pressure as f64;
            } else {
                // TODO: After defining the error code, send the related error
                // code event.
                warn!(
                    "Failed to read the pressure transducer at index {}. Use the cached value instead.",
                    idx
                );
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
    /// Panics if not in simulation mode.
    fn read_pressure(&self, bus: usize, address: u8) -> Option<f32> {
        let frame_request = self._modbus.create_frame_read_pressure(address);

        let frame_response;
        if let Some(plant) = &self.plant {
            if let Some(reading) = plant.request_sensor_pressure(bus, &frame_request) {
                frame_response = reading;
            } else {
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

                return None;
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
                None => {
                    // TODO: After defining the error code, send the related
                    // error code event.
                    warn!(
                        "Failed to read the power grid monitor at address {}. Use the cached value instead.",
                        address
                    );

                    match self._latest_telemetry.power_grid_monitors.get(idx) {
                        Some(power_grid_monitor) => {
                            power_grid_monitors.push(power_grid_monitor.clone());
                        }
                        None => {
                            power_grid_monitors.push(PowerGridMonitor::new(0));
                        }
                    }
                }
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
                None => {
                    // TODO: After defining the error code, send the related
                    // error code event.
                    warn!(
                        "Failed to read the flowmeter at address {}. Use the cached value instead.",
                        address_request
                    );

                    match self._latest_telemetry.flowmeters.get(idx) {
                        Some(power_grid_monitor) => {
                            flowmeters.push(power_grid_monitor.clone());
                        }
                        None => {
                            flowmeters.push(Flowmeter::new(0));
                        }
                    }
                }
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

    /// Read the current data from the pier fans.
    ///
    /// # Arguments
    /// * `digital_inputs_mod7` - The current digital inputs of Mod7, NI-9425.
    ///
    /// # Returns
    /// A vector containing the pier fan data.
    fn read_pier_fans(&mut self, digital_inputs_mod7: u32) -> Vec<PierFan> {
        let mut pier_fans = Vec::new();

        let num = self.config.addresses["pier_fan"].len();
        for idx in 0..num {
            let address = self.config.addresses["pier_fan"][idx];
            if self.is_enabled_pier_fan(idx, digital_inputs_mod7) {
                match self.read_pier_fan(address) {
                    Some(pier_fan) => {
                        pier_fans.push(pier_fan);
                    }
                    None => {
                        // TODO: After defining the error code, send the
                        // related error code event.
                        warn!(
                            "Failed to read the pier fan at address {}. Use the cached value instead.",
                            address
                        );

                        match self._latest_telemetry.pier_fans.get(idx) {
                            Some(pier_fan) => {
                                pier_fans.push(pier_fan.clone());
                            }
                            None => {
                                pier_fans.push(PierFan::new(0));
                            }
                        }
                    }
                }
            } else {
                pier_fans.push(PierFan::new(0));
            }
        }

        pier_fans
    }

    /// Read the current data of a specific pier fan.
    ///
    /// # Arguments
    /// * `address` - The address of the pier fan.
    ///
    /// # Returns
    /// An `Option` containing the pier fan data if the read was successful,
    /// or `None` if it failed.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    fn read_pier_fan(&mut self, address: u8) -> Option<PierFan> {
        let frame_requests = [
            self._modbus.create_frame_read_pier_fan_max_speed(address),
            self._modbus
                .create_frame_read_pier_fan_ref_dc_link_voltage_and_current(address),
            self._modbus.create_frame_read_pier_fan(address),
        ];

        let mut frame_responses = Vec::new();
        if let Some(plant) = &mut self.plant {
            for frame_request in &frame_requests {
                if let Some(reading) = plant.request_pier_fan(frame_request) {
                    frame_responses.push(reading);
                } else {
                    return None;
                }
            }
        } else {
            panic!("Not implemented yet.");
        }

        let max_speed = match self
            ._modbus
            .read_pier_fan_max_speed_from_frame(&frame_responses[0])
        {
            Some(speed) => speed,
            None => {
                // TODO: After defining the error code, send the related error
                // code event.
                warn!(
                    "Failed to read the max speed of the pier fan at address {}.",
                    address
                );

                return None;
            }
        };

        let (ref_dc_link_voltage, ref_dc_link_current) = match self
            ._modbus
            .read_pier_fan_dc_link_voltage_and_current_from_frame(&frame_responses[1])
        {
            Some((voltage, current)) => (voltage, current),
            None => {
                // TODO: After defining the error code, send the related error
                // code event.
                warn!(
                    "Failed to read the reference DC link voltage and current of the pier fan at address {}.",
                    address
                );

                return None;
            }
        };

        if let Some(pier_fan) = self._modbus.read_pier_fan_from_frame(
            max_speed,
            ref_dc_link_voltage,
            ref_dc_link_current,
            &frame_responses[2],
        ) {
            let address_received = pier_fan.address;
            if address_received == address {
                return Some(pier_fan);
            } else {
                // TODO: After defining the error code, send the related
                // error code event.
                warn!(
                    "Pier fan address mismatch (expected: {}, got: {}).",
                    address, address_received
                );

                return None;
            }
        }

        None
    }

    /// Read the current data from the recirculation pumps.
    ///
    /// # Arguments
    /// * `digital_inputs_mod4` - The current digital inputs of Mod4, NI-9425.
    ///
    /// # Returns
    /// A vector containing the recirculation pump data.
    fn read_recirculation_pumps(&mut self, digital_inputs_mod4: u32) -> Vec<RecirculationPump> {
        let mut recirculation_pumps = Vec::new();

        let num = self.config.addresses["recirculation_pump"].len();
        for idx in 0..num {
            let is_enabled = match idx {
                0 => (digital_inputs_mod4 & DigitalInputMod4::StatusK1.bit_value()) != 0,
                1 => (digital_inputs_mod4 & DigitalInputMod4::StatusK2.bit_value()) != 0,
                _ => false,
            };

            let address = self.config.addresses["recirculation_pump"][idx];
            if is_enabled {
                match self.read_recirculation_pump(address) {
                    Some(recirculation_pump) => {
                        recirculation_pumps.push(recirculation_pump);
                    }
                    None => {
                        // TODO: After defining the error code, send the
                        // related error code event.
                        warn!(
                            "Failed to read the recirculation pump at address {}. Use the cached value instead.",
                            address
                        );

                        match self._latest_telemetry.recirculation_pumps.get(idx) {
                            Some(recirculation_pump) => {
                                recirculation_pumps.push(recirculation_pump.clone());
                            }
                            None => {
                                recirculation_pumps.push(RecirculationPump::new(0));
                            }
                        }
                    }
                }
            } else {
                recirculation_pumps.push(RecirculationPump::new(0));
            }
        }

        recirculation_pumps
    }

    /// Read the current data of a specific recirculation pump.
    ///
    /// # Arguments
    /// * `address` - The address of the recirculation pump.
    ///
    /// # Returns
    /// An `Option` containing the recirculation pump data if the read was
    /// successful, or `None` if it failed.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    fn read_recirculation_pump(&mut self, address: u8) -> Option<RecirculationPump> {
        let frame_requests = self._modbus.create_frame_read_recirculation_pump(address);

        let mut frame_responses = Vec::new();
        if let Some(plant) = &mut self.plant {
            for frame_request in &frame_requests {
                if let Some(reading) = plant.request_recirculation_pump(frame_request) {
                    frame_responses.push(reading);
                } else {
                    return None;
                }
            }
        } else {
            panic!("Not implemented yet.");
        }

        if let Some(recirculation_pump) = self._modbus.read_recirculation_pump_from_frame(
            &frame_responses[0],
            &frame_responses[1],
            &frame_responses[2],
            &frame_responses[3],
        ) {
            let address_received = recirculation_pump.address;
            if address_received == address {
                return Some(recirculation_pump);
            } else {
                // TODO: After defining the error code, send the related
                // error code event.
                warn!(
                    "Recirculation pump address mismatch (expected: {}, got: {}).",
                    address, address_received
                );

                return None;
            }
        }

        None
    }

    /// Read the current data from the chillers.
    ///
    /// # Arguments
    /// * `digital_inputs_mod4` - The current digital inputs of Mod4, NI-9425.
    /// * `transaction_id` - The transaction ID for the Modbus TCP
    ///   communication.
    ///
    /// # Returns
    /// A vector containing the chiller data.
    fn read_chillers(&mut self, digital_inputs_mod4: u32, transaction_id: u16) -> Vec<Chiller> {
        let mut chillers = Vec::new();

        let num = self.config.addresses["chiller"].len();
        for idx in 0..num {
            let unit_id = self.config.addresses["chiller"][idx];
            if self.is_enabled_chiller(idx, digital_inputs_mod4) {
                match self.read_chiller(idx, transaction_id, unit_id) {
                    Some(chiller) => {
                        chillers.push(chiller);
                    }
                    None => {
                        // TODO: After defining the error code, send the
                        // related error code event.
                        warn!(
                            "Failed to read chiller at index {}. Use the cached value instead.",
                            idx
                        );

                        match self._latest_telemetry.chillers.get(idx) {
                            Some(chiller) => {
                                chillers.push(chiller.clone());
                            }
                            None => {
                                chillers.push(Chiller::new(0));
                            }
                        }
                    }
                }
            } else {
                chillers.push(Chiller::new(0));
            }
        }

        chillers
    }

    /// Read the current data of a specific chiller.
    ///
    /// # Arguments
    /// * `idx` - The index of the chiller.
    /// * `transaction_id` - The transaction ID for the Modbus TCP
    ///   communication.
    /// * `unit_id` - The unit ID of the chiller.
    ///
    /// # Returns
    /// An `Option` containing the chiller data if the read was successful,
    /// or `None` if it failed.
    ///
    /// # Panics
    /// Panics if not in simulation mode.
    fn read_chiller(&mut self, idx: usize, transaction_id: u16, unit_id: u8) -> Option<Chiller> {
        let frame_request = self
            ._modbus
            .create_frame_read_chiller(transaction_id, unit_id);

        let frame_response;
        if let Some(plant) = &mut self.plant {
            if let Some(reading) = plant.request_chiller(idx, &frame_request) {
                frame_response = reading;
            } else {
                return None;
            }
        } else {
            panic!("Not implemented yet.");
        }

        if let Some(chiller) = self._modbus.read_chiller_from_frame(&frame_response) {
            let transaction_id_received = chiller.transaction_id;
            let unit_id_received = chiller.unit_id;
            if (transaction_id_received == transaction_id) && (unit_id_received == unit_id) {
                return Some(chiller);
            } else {
                // TODO: After defining the error code, send the related
                // error code event.
                warn!(
                    "Chiller transaction ID or unit ID mismatch (expected: ({}, {}), got: ({}, {})).",
                    transaction_id, unit_id, transaction_id_received, unit_id_received
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

    /// Read the current tank levels as the percentages and the distances to
    /// full.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;

    use ts_control_utils::{enums::BitEnum, utility::assert_relative_eq_vector};

    use crate::constants::{NUM_CMV, NUM_PCV};
    use crate::enums::DigitalInputMod4;
    use crate::mock::mock_constants::{PLANT_PRESSURE, PLANT_TEMPERATURE};
    use crate::telemetry::telemetry_default::TelemetryDefault;

    const EPSILON: f64 = 1e-6;

    /// Open the specified motor operated valve (MOV).
    ///
    /// # Arguments
    /// * `idx` - The index of the motor operated valve (0-4).
    /// * `switch_on` - `true` to open the valve, `false` to close it.
    ///
    /// # Returns
    /// Ok if the valve is switched successfully. Otherwise, Err with an error
    /// message.
    fn open_motor_operated_valve(
        data_acquisition: &mut DataAcquisition,
        idx: usize,
        switch_on: bool,
    ) -> Result<(), String> {
        let digital_output = match idx {
            0 => DigitalOutput::OpenMov1,
            1 => DigitalOutput::OpenMov2,
            2 => DigitalOutput::OpenMov3,
            3 => DigitalOutput::OpenMov4,
            4 => DigitalOutput::OpenMov5,
            _ => return Err(String::from("Invalid motor operated valve index.")),
        };

        data_acquisition.switch_digital_output(digital_output, switch_on)
    }

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
        let _ = data_acquisition.switch_digital_output(digital_output, true);

        assert_eq!(
            data_acquisition.plant.as_ref().unwrap().digital_outputs,
            DigitalOutput::OpenMov5.bit_value()
        );

        // Switch the digital output off
        let _ = data_acquisition.switch_digital_output(digital_output, false);

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
            assert!(open_motor_operated_valve(&mut data_acquisition, idx, true).is_ok());
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
            assert!(open_motor_operated_valve(&mut data_acquisition, idx, false).is_ok());
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
        assert_eq!(
            open_motor_operated_valve(&mut data_acquisition, 5, true),
            Err(String::from("Invalid motor operated valve index."))
        );
    }

    #[test]
    fn test_open_pressure_control_valve_invalid_index() {
        let mut data_acquisition = DataAcquisition::new(true);

        assert_eq!(
            data_acquisition.open_pressure_control_valve(2, 50.0),
            Err(String::from("Invalid valve index."))
        );
    }

    #[test]
    fn test_open_control_mixing_valve_invalid_index() {
        let mut data_acquisition = DataAcquisition::new(true);

        assert_eq!(
            data_acquisition.open_control_mixing_valve(3, 50.0),
            Err(String::from("Invalid valve index."))
        );
    }

    #[test]
    fn test_is_valid_percentage() {
        let data_acquisition = DataAcquisition::new(true);

        assert_eq!(data_acquisition.is_valid_percentage(0.0), true);
        assert_eq!(data_acquisition.is_valid_percentage(50.0), true);
        assert_eq!(data_acquisition.is_valid_percentage(100.0), true);
        assert_eq!(data_acquisition.is_valid_percentage(-1.0), false);
        assert_eq!(data_acquisition.is_valid_percentage(101.0), false);
    }

    #[test]
    fn test_reset_pier_fan() {
        let mut data_acquisition = DataAcquisition::new(true);

        // Invalid index
        assert_eq!(
            data_acquisition.reset_pier_fan(2),
            Err(String::from("Invalid pier fan index."))
        );

        // Valid indices but not enabled
        assert!(data_acquisition.reset_pier_fan(0).is_ok());
        assert!(data_acquisition.reset_pier_fan(1).is_ok());

        // Valid indices and enabled
        data_acquisition._latest_telemetry.digital_inputs_mod7 |=
            DigitalInputMod7::StatusK3.bit_value();
        data_acquisition._latest_telemetry.digital_inputs_mod7 |=
            DigitalInputMod7::StatusK4.bit_value();

        assert!(data_acquisition.reset_pier_fan(0).is_ok());
        assert!(data_acquisition.reset_pier_fan(1).is_ok());
    }

    #[test]
    fn test_is_enabled_pier_fan() {
        let data_acquisition = DataAcquisition::new(true);

        let mut digital_inputs_mod7 = 0;
        assert_eq!(
            data_acquisition.is_enabled_pier_fan(0, digital_inputs_mod7),
            false
        );
        assert_eq!(
            data_acquisition.is_enabled_pier_fan(1, digital_inputs_mod7),
            false
        );

        digital_inputs_mod7 |= DigitalInputMod7::StatusK3.bit_value();
        assert_eq!(
            data_acquisition.is_enabled_pier_fan(0, digital_inputs_mod7),
            true
        );
        assert_eq!(
            data_acquisition.is_enabled_pier_fan(1, digital_inputs_mod7),
            false
        );

        digital_inputs_mod7 |= DigitalInputMod7::StatusK4.bit_value();
        assert_eq!(
            data_acquisition.is_enabled_pier_fan(0, digital_inputs_mod7),
            true
        );
        assert_eq!(
            data_acquisition.is_enabled_pier_fan(1, digital_inputs_mod7),
            true
        );
    }

    #[test]
    fn test_set_chiller_temperature() {
        let mut data_acquisition = DataAcquisition::new(true);

        // Should fail because the temperature is out of the valid range.
        assert_eq!(
            data_acquisition.set_chiller_temperature(0, 131.0),
            Err(String::from(
                "Chiller temperature setpoint in Fahrenheit is out of range: [0, 250]."
            ))
        );

        // Should fail because there is no chiller enabled.
        let temperature = 25.0;
        assert_eq!(
            data_acquisition.set_chiller_temperature(0, temperature),
            Err(String::from("Chiller is not enabled."))
        );

        // Should succeed because the chiller is enabled and the temperature is
        // within the valid range.
        data_acquisition._latest_telemetry.digital_inputs_mod4 |=
            DigitalInputMod4::StatusK21.bit_value();
        assert!(
            data_acquisition
                .set_chiller_temperature(0, temperature)
                .is_ok()
        );

        assert_eq!(data_acquisition._transaction_id, 1);

        // Check the telemetry to ensure the chiller temperature was set
        // correctly.
        let chiller = data_acquisition.read_chiller(0, 2, 1).unwrap();
        assert_eq!(
            chiller.temperature_setpoint,
            celsius_to_fahrenheit(temperature) as u16
        );
    }

    #[test]
    fn test_is_enabled_chiller() {
        let data_acquisition = DataAcquisition::new(true);

        let mut digital_inputs_mod4 = 0;
        assert_eq!(
            data_acquisition.is_enabled_chiller(0, digital_inputs_mod4),
            false
        );
        assert_eq!(
            data_acquisition.is_enabled_chiller(1, digital_inputs_mod4),
            false
        );

        digital_inputs_mod4 |= DigitalInputMod4::StatusK21.bit_value();
        assert_eq!(
            data_acquisition.is_enabled_chiller(0, digital_inputs_mod4),
            true
        );
        assert_eq!(
            data_acquisition.is_enabled_chiller(1, digital_inputs_mod4),
            false
        );

        digital_inputs_mod4 |= DigitalInputMod4::StatusK22.bit_value();
        assert_eq!(
            data_acquisition.is_enabled_chiller(0, digital_inputs_mod4),
            true
        );
        assert_eq!(
            data_acquisition.is_enabled_chiller(1, digital_inputs_mod4),
            true
        );
    }

    #[test]
    fn test_get_transaction_id() {
        let mut data_acquisition = DataAcquisition::new(true);

        // Initially, the transaction ID should start at 0. The value should
        // increment with each call.
        assert_eq!(data_acquisition.get_transaction_id(), 0);
        assert_eq!(data_acquisition.get_transaction_id(), 1);

        // Set the transaction ID close to the maximum value to test the
        // wrap-around behavior.
        data_acquisition._transaction_id = u16::MAX - 1;
        assert_eq!(data_acquisition.get_transaction_id(), u16::MAX - 1);
        assert_eq!(data_acquisition.get_transaction_id(), u16::MAX);
        assert_eq!(data_acquisition.get_transaction_id(), 0);
    }

    #[test]
    fn test_get_telemetry() {
        let mut data_acquisition = DataAcquisition::new(true);
        let percentage = 54.0;
        for idx in 0..NUM_PCV {
            let _ = data_acquisition.open_pressure_control_valve(idx, percentage);
        }
        for idx in 0..NUM_CMV {
            let _ = data_acquisition.open_control_mixing_valve(idx, percentage);
        }

        let _ = data_acquisition.switch_digital_output(DigitalOutput::OpenMov1, true);
        let _ = data_acquisition.switch_digital_output(DigitalOutput::PowerPierFan1, true);
        let _ = data_acquisition.switch_digital_output(DigitalOutput::PowerPierFan2, true);
        let _ = data_acquisition.switch_digital_output(DigitalOutput::PowerChiller1, true);
        let _ = data_acquisition.switch_digital_output(DigitalOutput::PowerChiller2, true);
        let _ =
            data_acquisition.switch_digital_output(DigitalOutput::PowerRecirculationPump1, true);
        let _ =
            data_acquisition.switch_digital_output(DigitalOutput::PowerRecirculationPump2, true);

        // Get the telemetry
        let telemetry = data_acquisition.get_telemetry();

        assert_eq!(telemetry.digital_inputs_mod4, 62121);
        assert_eq!(telemetry.digital_inputs_mod7, 3);
        assert_eq!(telemetry.digital_outputs, 2013266689);

        assert_eq!(telemetry.temperatures[0], PLANT_TEMPERATURE as f64);
        assert_eq!(telemetry.pressures[0], PLANT_PRESSURE as f64);

        assert_eq!(telemetry.power_grid_monitors[0].address, 1);
        assert_eq!(telemetry.flowmeters[0].address, 1);
        assert_eq!(telemetry.pier_fans[0].address, 11);
        assert_eq!(telemetry.recirculation_pumps[0].address, 1);

        assert_eq!(telemetry.chillers[0].transaction_id, 0);
        assert_eq!(telemetry.chillers[0].unit_id, 1);

        assert_eq!(data_acquisition._transaction_id, 1);

        assert_eq!(telemetry.percentages_pcv[0], percentage);
        assert_eq!(telemetry.percentages_cmv[0], percentage);

        assert_eq!(telemetry.tank_levels["distance"][0], 263.1773);

        assert_eq!(data_acquisition._latest_telemetry, telemetry);

        // Check the messages
        let messages = telemetry.get_messages(2);

        assert_eq!(messages.len(), 31);
    }

    #[test]
    fn test_read_temperatures() {
        let data_acquisition = DataAcquisition::new(true);

        let temperatures = data_acquisition.read_temperatures();

        assert_eq!(
            temperatures,
            vec![PLANT_TEMPERATURE as f64; NUM_TEMPERATURE_HUB * NUM_TEMPERATURE_CHANNEL]
        );
    }

    #[test]
    fn test_read_pressures() {
        let data_acquisition = DataAcquisition::new(true);

        let pressures = data_acquisition.read_pressures();

        assert_eq!(
            pressures,
            vec![PLANT_PRESSURE as f64; NUMS_PRESSURE_TRANSDUCER.iter().sum()]
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
    fn test_read_pier_fans() {
        let mut data_acquisition = DataAcquisition::new(true);

        // Should be the default data because the control status is not
        // updated yet.
        let mut digital_inputs_mod7 = 0;
        let pier_fans = data_acquisition.read_pier_fans(digital_inputs_mod7);

        let addresses: Vec<u8> = pier_fans.iter().map(|fan| fan.address).collect();
        assert_eq!(addresses, [0, 0]);

        // Should be updated after the control status is changed.
        digital_inputs_mod7 |= DigitalInputMod7::StatusK3.bit_value();
        let pier_fans = data_acquisition.read_pier_fans(digital_inputs_mod7);

        let addresses: Vec<u8> = pier_fans.iter().map(|fan| fan.address).collect();
        assert_eq!(addresses, [11, 0]);

        digital_inputs_mod7 |= DigitalInputMod7::StatusK4.bit_value();
        let pier_fans = data_acquisition.read_pier_fans(digital_inputs_mod7);

        let addresses: Vec<u8> = pier_fans.iter().map(|fan| fan.address).collect();
        assert_eq!(addresses, [11, 12]);
    }

    #[test]
    fn test_read_recirculation_pumps() {
        let mut data_acquisition = DataAcquisition::new(true);

        // Should be the default data because the control status is not
        // updated yet.
        let mut digital_inputs_mod4 = 0;
        let recirculation_pumps = data_acquisition.read_recirculation_pumps(digital_inputs_mod4);

        let addresses: Vec<u8> = recirculation_pumps
            .iter()
            .map(|recirculation_pump| recirculation_pump.address)
            .collect();
        assert_eq!(addresses, [0, 0]);

        // Should be updated after the control status is changed.
        digital_inputs_mod4 |= DigitalInputMod4::StatusK1.bit_value();
        let recirculation_pumps = data_acquisition.read_recirculation_pumps(digital_inputs_mod4);

        let addresses: Vec<u8> = recirculation_pumps
            .iter()
            .map(|recirculation_pump| recirculation_pump.address)
            .collect();
        assert_eq!(addresses, [1, 0]);

        digital_inputs_mod4 |= DigitalInputMod4::StatusK2.bit_value();
        let recirculation_pumps = data_acquisition.read_recirculation_pumps(digital_inputs_mod4);

        let addresses: Vec<u8> = recirculation_pumps
            .iter()
            .map(|recirculation_pump| recirculation_pump.address)
            .collect();
        assert_eq!(addresses, [1, 2]);
    }

    #[test]
    fn test_read_chillers() {
        let mut data_acquisition = DataAcquisition::new(true);

        // Should be the default data because the control status is not
        // updated yet.
        let mut digital_inputs_mod4 = 0;
        let chillers = data_acquisition.read_chillers(digital_inputs_mod4, 1);

        let unit_ids: Vec<u8> = chillers.iter().map(|chiller| chiller.unit_id).collect();
        assert_eq!(unit_ids, [0, 0]);

        // Should be updated after the control status is changed.
        digital_inputs_mod4 |= DigitalInputMod4::StatusK21.bit_value();
        let chillers = data_acquisition.read_chillers(digital_inputs_mod4, 2);

        let unit_ids: Vec<u8> = chillers.iter().map(|chiller| chiller.unit_id).collect();
        assert_eq!(unit_ids, [1, 0]);

        digital_inputs_mod4 |= DigitalInputMod4::StatusK22.bit_value();
        let chillers = data_acquisition.read_chillers(digital_inputs_mod4, 3);

        let unit_ids: Vec<u8> = chillers.iter().map(|chiller| chiller.unit_id).collect();
        assert_eq!(unit_ids, [1, 1]);
    }

    #[test]
    fn test_read_percentage_of_pressure_control_valves() {
        let mut data_acquisition = DataAcquisition::new(true);

        let percentage = 54.0;
        for idx in 0..NUM_PCV {
            let _ = data_acquisition.open_pressure_control_valve(idx, percentage);
        }

        assert_eq!(
            data_acquisition.read_percentage_of_pressure_control_valves(),
            vec![percentage; NUM_PCV]
        );
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
                    .is_ok()
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
            let _ = data_acquisition.open_control_mixing_valve(idx, percentage);
        }

        assert_eq!(
            data_acquisition.read_percentage_of_control_mixing_valves(),
            vec![percentage; NUM_CMV]
        );
    }

    #[test]
    fn test_open_control_mixing_valve_and_read_percentage() {
        let mut data_acquisition = DataAcquisition::new(true);

        let percentage = 54.0;
        for idx in 0..NUM_CMV {
            assert!(
                data_acquisition
                    .open_control_mixing_valve(idx, percentage)
                    .is_ok()
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

        assert_eq!(distances, vec![263.1773; NUM_TANK]);
        assert_relative_eq_vector(&percentages, &vec![69.3979884; NUM_TANK], EPSILON);
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
