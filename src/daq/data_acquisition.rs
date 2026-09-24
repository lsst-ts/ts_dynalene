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
use crate::constants::MAX_ANALOG_INPUT_OUTPUT;
use crate::daq::{modbus_communicator::ModbusCommunicator, tank_level::TankLevel};
use crate::enums::{AnalogInput, AnalogOutput, DigitalOutput};
use crate::mock::mock_plant::MockPlant;

pub struct DataAcquisition {
    // Configuration
    pub config: Config,
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

    /// Read the current percentage opening of the specified pressure control
    /// valve (PCV).
    ///
    /// # Arguments
    /// * `idx` - The index of the pressure control valve (0-1).
    ///
    /// # Returns
    /// Some containing the percentage opening if successful. Otherwise, None.
    pub fn read_percentage_of_pressure_control_valve(&self, idx: usize) -> Option<f64> {
        self.read_percentage_of_analog_input(
            idx,
            &[AnalogInput::ReadoutPcv1, AnalogInput::ReadoutPcv2],
        )
    }

    /// Read the current percentage opening of the specified control mixing
    /// valve (CMV).
    ///
    /// # Arguments
    /// * `idx` - The index of the control mixing valve (0-2).
    ///
    /// # Returns
    /// Some containing the percentage opening if successful. Otherwise, None.
    pub fn read_percentage_of_control_mixing_valve(&self, idx: usize) -> Option<f64> {
        self.read_percentage_of_analog_input(
            idx,
            &[
                AnalogInput::ReadoutCmv1,
                AnalogInput::ReadoutCmv2,
                AnalogInput::ReadoutCmv20,
            ],
        )
    }

    /// Read the current tank level as a percentage and the distance to full.
    ///
    /// # Arguments
    /// * `idx` - The index of the tank (0-1).
    ///
    /// # Returns
    /// Some containing a tuple of the distance to full (in millimeters) and
    /// the remaining percentage if successful. Otherwise, None.
    pub fn read_percentage_of_tank_level(&self, idx: usize) -> Option<(f64, f64)> {
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

    use ts_control_utils::enums::BitEnum;

    use crate::enums::DigitalInputMod4;

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
        for idx in 0..2 {
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
    fn test_open_control_mixing_valve_and_read_percentage() {
        let mut data_acquisition = DataAcquisition::new(true);

        let percentage = 54.0;
        for idx in 0..3 {
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
    fn test_read_percentage_of_tank_level() {
        let data_acquisition = DataAcquisition::new(true);

        for idx in 0..2 {
            let (distance_to_full, remaining_percentage) =
                data_acquisition.read_percentage_of_tank_level(idx).unwrap();

            assert_eq!(distance_to_full, 263.1773);
            assert_relative_eq!(remaining_percentage, 69.3979884, epsilon = EPSILON);
        }
    }
}
