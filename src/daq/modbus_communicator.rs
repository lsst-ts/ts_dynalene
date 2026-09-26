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
use regex::{Captures, Regex};
use std::str::{FromStr, from_utf8};

use crate::constants::{
    BYTES_HOLDING_REGISTER, BYTES_RESPONSE_PRESSURE, BYTES_RESPONSE_TEMPERATURE,
    CODE_READ_HOLDING_REGISTERS, CODE_WRITE_SINGLE_REGISTER, NUM_REGISTER_CHILLER,
    NUM_REGISTER_FLOWMETER, NUM_REGISTER_PIER_FAN_ACTUAL_SPEED,
    NUM_REGISTER_PIER_FAN_REFERENCE_VALUE_OF_DC_LINK_VOLTAGE, NUM_REGISTER_POWER_GRID_MONITOR,
    NUM_REGISTER_RECIRCULATION_PUMP_CIM_CONFIGURATION, NUM_REGISTER_RECIRCULATION_PUMP_CONTROL,
    NUM_REGISTER_RECIRCULATION_PUMP_DATA, NUM_REGISTER_RECIRCULATION_PUMP_STATUS,
    NUM_TEMPERATURE_CHANNEL, REGISTER_ADDRESS_CHILLER_TEMPERATURE_SETPOINT,
    REGISTER_ADDRESS_FLOWMETER, REGISTER_ADDRESS_PIER_FAN_ACTUAL_SPEED,
    REGISTER_ADDRESS_PIER_FAN_MAXIMUM_SPEED,
    REGISTER_ADDRESS_PIER_FAN_REFERENCE_VALUE_OF_DC_LINK_VOLTAGE, REGISTER_ADDRESS_PIER_FAN_RESET,
    REGISTER_ADDRESS_POWER_GRID_MONITOR, REGISTER_ADDRESS_RECIRCULATION_PUMP_CIM_CONFIGURATION,
    REGISTER_ADDRESS_RECIRCULATION_PUMP_CONTROL, REGISTER_ADDRESS_RECIRCULATION_PUMP_DATA,
    REGISTER_ADDRESS_RECIRCULATION_PUMP_STATUS,
};
use crate::daq::{
    chiller::Chiller, flowmeter::Flowmeter, pier_fan::PierFan,
    power_grid_monitor::PowerGridMonitor, recirculation_pump::RecirculationPump,
};

use crate::utility::{calculate_modbus_crc_and_update_frame, verify_modbus_crc};

pub struct ModbusCommunicator {
    // Cyclic redundancy check (CRC) calculator for the Modbus communication.
    _crc: Crc<u16>,
}

impl Default for ModbusCommunicator {
    fn default() -> Self {
        Self::new()
    }
}

impl ModbusCommunicator {
    /// Create a new Modbus communicator.
    ///
    /// # Returns
    /// A new Modbus communicator instance.
    pub fn new() -> Self {
        // Use the const here because the lookup table for the CRC algorithm
        // is precomputed and stored in the constant.
        const CRC_ALGORITHM: Crc<u16> = Crc::<u16>::new(&CRC_16_MODBUS);

        Self {
            _crc: CRC_ALGORITHM,
        }
    }

    /// Create a frame to read the flowmeter.
    ///
    /// # Arguments
    /// * `address` - The address of the device to communicate with.
    ///
    /// # Returns
    /// A byte array representing the frame to read the flowmeter.
    pub fn create_frame_read_flowmeter(&self, address: u8) -> [u8; BYTES_HOLDING_REGISTER] {
        self.create_frame_read_holding_registers(
            address,
            REGISTER_ADDRESS_FLOWMETER,
            NUM_REGISTER_FLOWMETER,
        )
    }

    /// Create a frame to read the power grid monitor.
    ///
    /// # Arguments
    /// * `address` - The address of the device to communicate with.
    ///
    /// # Returns
    /// A byte array representing the frame to read the power grid monitor.
    pub fn create_frame_read_power_grid_monitor(
        &self,
        address: u8,
    ) -> [u8; BYTES_HOLDING_REGISTER] {
        self.create_frame_read_holding_registers(
            address,
            REGISTER_ADDRESS_POWER_GRID_MONITOR,
            NUM_REGISTER_POWER_GRID_MONITOR,
        )
    }

    /// Create a frame to read the pier fan maximum speed.
    ///
    /// # Arguments
    /// * `address` - The address of the device to communicate with.
    ///
    /// # Returns
    /// A byte array representing the frame to read the pier fan maximum speed.
    pub fn create_frame_read_pier_fan_max_speed(
        &self,
        address: u8,
    ) -> [u8; BYTES_HOLDING_REGISTER] {
        self.create_frame_read_holding_registers(
            address,
            REGISTER_ADDRESS_PIER_FAN_MAXIMUM_SPEED,
            1,
        )
    }

    /// Create a frame to read the pier fan reference DC link voltage and
    /// current.
    ///
    /// # Arguments
    /// * `address` - The address of the device to communicate with.
    ///
    /// # Returns
    /// A byte array representing the frame to read the pier fan reference DC
    /// link voltage and current.
    pub fn create_frame_read_pier_fan_ref_dc_link_voltage_and_current(
        &self,
        address: u8,
    ) -> [u8; BYTES_HOLDING_REGISTER] {
        self.create_frame_read_holding_registers(
            address,
            REGISTER_ADDRESS_PIER_FAN_REFERENCE_VALUE_OF_DC_LINK_VOLTAGE,
            NUM_REGISTER_PIER_FAN_REFERENCE_VALUE_OF_DC_LINK_VOLTAGE,
        )
    }

    /// Create a frame to read the pier fan.
    ///
    /// # Arguments
    /// * `address` - The address of the device to communicate with.
    ///
    /// # Returns
    /// A byte array representing the frame to read the pier fan.
    pub fn create_frame_read_pier_fan(&self, address: u8) -> [u8; BYTES_HOLDING_REGISTER] {
        self.create_frame_read_holding_registers(
            address,
            REGISTER_ADDRESS_PIER_FAN_ACTUAL_SPEED,
            NUM_REGISTER_PIER_FAN_ACTUAL_SPEED,
        )
    }

    /// Create a frame to read the recirculation pump.
    ///
    /// # Arguments
    /// * `address` - The address of the device to communicate with.
    ///
    /// # Returns
    /// An array of byte arrays representing the frames to read the
    /// recirculation pump. The order is the CIM configuration, control,
    /// status, and data.
    pub fn create_frame_read_recirculation_pump(
        &self,
        address: u8,
    ) -> [[u8; BYTES_HOLDING_REGISTER]; 4] {
        [
            self.create_frame_read_holding_registers(
                address,
                REGISTER_ADDRESS_RECIRCULATION_PUMP_CIM_CONFIGURATION,
                NUM_REGISTER_RECIRCULATION_PUMP_CIM_CONFIGURATION,
            ),
            self.create_frame_read_holding_registers(
                address,
                REGISTER_ADDRESS_RECIRCULATION_PUMP_CONTROL,
                NUM_REGISTER_RECIRCULATION_PUMP_CONTROL,
            ),
            self.create_frame_read_holding_registers(
                address,
                REGISTER_ADDRESS_RECIRCULATION_PUMP_STATUS,
                NUM_REGISTER_RECIRCULATION_PUMP_STATUS,
            ),
            self.create_frame_read_holding_registers(
                address,
                REGISTER_ADDRESS_RECIRCULATION_PUMP_DATA,
                NUM_REGISTER_RECIRCULATION_PUMP_DATA,
            ),
        ]
    }

    /// Create a frame to read the chiller.
    ///
    /// # Arguments
    /// * `address` - The address of the device to communicate with.
    ///
    /// # Returns
    /// A byte array representing the frame to read the chiller.
    pub fn create_frame_read_chiller(&self, address: u8) -> [u8; BYTES_HOLDING_REGISTER] {
        self.create_frame_read_holding_registers(
            address,
            REGISTER_ADDRESS_CHILLER_TEMPERATURE_SETPOINT,
            NUM_REGISTER_CHILLER,
        )
    }

    /// Create a frame to read holding registers.
    ///
    /// # Arguments
    /// * `address` - The address of the device to communicate with.
    /// * `register_address` - The starting address of the holding registers to
    ///   read.
    /// * `num_register` - The number of holding registers to read.
    ///
    /// # Returns
    /// A byte array representing the frame to read holding registers.
    fn create_frame_read_holding_registers(
        &self,
        address: u8,
        register_address: u16,
        num_register: u16,
    ) -> [u8; BYTES_HOLDING_REGISTER] {
        let mut frame = [0; BYTES_HOLDING_REGISTER];
        frame[0] = address;
        frame[1] = CODE_READ_HOLDING_REGISTERS;
        frame[2..4].copy_from_slice(&register_address.to_be_bytes());
        frame[4..6].copy_from_slice(&num_register.to_be_bytes());

        calculate_modbus_crc_and_update_frame(&self._crc, &mut frame);

        frame
    }

    /// Read the flowmeter value from a received frame.
    ///
    /// # Arguments
    /// * `frame` - The received frame containing the flowmeter data.
    ///
    /// # Returns
    /// A `Flowmeter` instance if the frame is valid. Otherwise, `None`.
    pub fn read_flowmeter_from_frame(&self, frame: &[u8]) -> Option<Flowmeter> {
        if !verify_modbus_crc(&self._crc, frame) {
            return None;
        }

        Flowmeter::from_frame(frame)
    }

    /// Read the power grid monitor value from a received frame.
    ///
    /// # Arguments
    /// * `frame` - The received frame containing the power grid monitor data.
    ///
    /// # Returns
    /// A `PowerGridMonitor` instance if the frame is valid. Otherwise, `None`.
    pub fn read_power_grid_monitor_from_frame(&self, frame: &[u8]) -> Option<PowerGridMonitor> {
        if !verify_modbus_crc(&self._crc, frame) {
            return None;
        }

        PowerGridMonitor::from_frame(frame)
    }

    /// Read the maximum speed of the pier fan from a received frame.
    ///
    /// # Arguments
    /// * `frame` - The received frame containing the pier fan maximum speed
    ///   data.
    ///
    /// # Returns
    /// The maximum speed of the pier fan in rpm if the frame is valid.
    /// Otherwise, `None`.
    pub fn read_pier_fan_max_speed_from_frame(&self, frame: &[u8]) -> Option<f32> {
        if !verify_modbus_crc(&self._crc, frame) {
            return None;
        }

        PierFan::get_max_speed_from_frame(frame)
    }

    /// Read the DC link voltage and current of the pier fan from a received
    /// frame.
    ///
    /// # Arguments
    /// * `frame` - The received frame containing the pier fan DC link voltage
    ///   and current data.
    ///
    /// # Returns
    /// A tuple containing the DC link voltage in volts and current in amperes
    /// if the frame is valid. Otherwise, `None`.
    pub fn read_pier_fan_dc_link_voltage_and_current_from_frame(
        &self,
        frame: &[u8],
    ) -> Option<(f32, f32)> {
        if !verify_modbus_crc(&self._crc, frame) {
            return None;
        }

        PierFan::get_ref_dc_link_voltage_current_from_frame(frame)
    }

    /// Read the pier fan data from a received frame.
    ///
    /// # Arguments
    /// * `max_speed` - The maximum speed of the pier fan in rpm.
    /// * `ref_dc_link_voltage` - The reference DC link voltage in volts. To
    ///   keep the resolution variable, all values for the DC-link voltage are
    ///   based on this reference value. Note the raw register value is in mV.
    /// * `ref_dc_link_current` - The reference DC link current in amperes. To
    ///   keep the resolution variable, all values for the DC-link current are
    ///   based on this reference value. Note the raw register value is in mA.
    /// * `frame` - The received frame containing the pier fan data.
    ///
    /// # Returns
    /// A `PierFan` instance if the frame is valid. Otherwise, `None`.
    pub fn read_pier_fan_from_frame(
        &self,
        max_speed: f32,
        ref_dc_link_voltage: f32,
        ref_dc_link_current: f32,
        frame: &[u8],
    ) -> Option<PierFan> {
        if !verify_modbus_crc(&self._crc, frame) {
            return None;
        }

        PierFan::from_frame(max_speed, ref_dc_link_voltage, ref_dc_link_current, frame)
    }

    /// Read the recirculation pump data from received frames.
    ///
    /// # Arguments
    /// * `frame_cim_configuration` - The received frame containing the CIM
    ///   configuration data.
    /// * `frame_control` - The received frame containing the control data.
    /// * `frame_status` - The received frame containing the status data.
    /// * `frame_data` - The received frame containing the recirculation pump
    ///   data.
    ///
    /// # Returns
    /// A `RecirculationPump` instance if all frames are valid. Otherwise, `None`.
    pub fn read_recirculation_pump_from_frame(
        &self,
        frame_cim_configuration: &[u8],
        frame_control: &[u8],
        frame_status: &[u8],
        frame_data: &[u8],
    ) -> Option<RecirculationPump> {
        if (!verify_modbus_crc(&self._crc, frame_cim_configuration))
            || (!verify_modbus_crc(&self._crc, frame_control))
            || (!verify_modbus_crc(&self._crc, frame_status))
            || (!verify_modbus_crc(&self._crc, frame_data))
        {
            return None;
        }

        Some(RecirculationPump::from_frame(
            frame_cim_configuration,
            frame_control,
            frame_status,
            frame_data,
        ))
    }

    /// Read the chiller data from a received frame.
    ///
    /// # Arguments
    /// * `frame` - The received frame containing the chiller data.
    ///
    /// # Returns
    /// A `Chiller` instance if the frame is valid. Otherwise, `None`.
    pub fn read_chiller_from_frame(&self, frame: &[u8]) -> Option<Chiller> {
        if !verify_modbus_crc(&self._crc, frame) {
            return None;
        }

        Chiller::from_frame(frame)
    }

    /// Create a frame to reset the pier fan.
    ///
    /// # Arguments
    /// * `address` - The address of the device to communicate with.
    ///
    /// # Returns
    /// A byte array representing the frame to reset the pier fan.
    pub fn create_frame_pier_fan_reset(&self, address: u8) -> [u8; BYTES_HOLDING_REGISTER] {
        self.create_frame_write_single_register(address, REGISTER_ADDRESS_PIER_FAN_RESET, 1)
    }

    /// Create a frame to set the chiller temperature.
    ///
    /// # Arguments
    /// * `address` - The address of the device to communicate with.
    /// * `temperature` - The temperature to set for the chiller. The unit is
    ///   the degree Fahrenheit.
    ///
    /// # Returns
    /// A byte array representing the frame to set the chiller temperature.
    pub fn create_frame_chiller_set_temperature(
        &self,
        address: u8,
        temperature: u16,
    ) -> [u8; BYTES_HOLDING_REGISTER] {
        self.create_frame_write_single_register(
            address,
            REGISTER_ADDRESS_CHILLER_TEMPERATURE_SETPOINT,
            temperature,
        )
    }

    /// Create a frame to write a single holding register.
    ///
    /// # Arguments
    /// * `address` - The address of the device to communicate with.
    /// * `register_address` - The address of the holding register to write.
    /// * `value` - The value to write to the holding register.
    ///
    /// # Returns
    /// A byte array representing the frame to write a single holding register.
    fn create_frame_write_single_register(
        &self,
        address: u8,
        register_address: u16,
        value: u16,
    ) -> [u8; BYTES_HOLDING_REGISTER] {
        let mut frame = [0; BYTES_HOLDING_REGISTER];
        frame[0] = address;
        frame[1] = CODE_WRITE_SINGLE_REGISTER;
        frame[2..4].copy_from_slice(&register_address.to_be_bytes());
        frame[4..6].copy_from_slice(&value.to_be_bytes());

        calculate_modbus_crc_and_update_frame(&self._crc, &mut frame);

        frame
    }

    /// Create a frame to read the pressure.
    ///
    /// # Arguments
    /// * `address` - The address of the device to communicate with.
    ///
    /// # Returns
    /// A byte array representing the frame to read the pressure.
    pub fn create_frame_read_pressure(&self, address: u8) -> Vec<u8> {
        format!("#{:03}P\r\n", address).as_bytes().to_vec()
    }

    /// Read the pressure value from a received frame.
    ///
    /// # Arguments
    /// * `frame` - The received frame containing the pressure data.
    ///
    /// # Returns
    /// A tuple containing the address and the pressure in PSI if the frame is
    /// valid. Otherwise, `None`.
    pub fn read_pressure_from_frame(&self, frame: &[u8]) -> Option<(u8, f32)> {
        if frame.len() != BYTES_RESPONSE_PRESSURE {
            return None;
        }

        let response = from_utf8(frame).ok()?;
        let re = Regex::new(r"@(\d+)\s(\d+\.\d+)\s").ok()?;
        let captures = re.captures(response)?;

        let address = self.parse_capture(&captures, 1)?;
        let pressure = self.parse_capture(&captures, 2)?;

        Some((address, pressure))
    }

    /// Parse a value of type `T` from a regex capture group.
    ///
    /// # Arguments
    /// * `captures` - The regex captures to extract the value from.
    /// * `index` - The index of the capture group to parse.
    ///
    /// # Returns
    /// The parsed value if the capture group exists and parses successfully.
    /// Otherwise, `None`.
    fn parse_capture<T: FromStr>(&self, captures: &Captures, index: usize) -> Option<T> {
        captures.get(index)?.as_str().parse().ok()
    }

    /// Read the temperature values from a received frame.
    ///
    /// # Arguments
    /// * `frame` - The received frame containing the temperature data.
    ///
    /// # Returns
    /// An array of temperatures in degree Celsius if the frame is valid.
    /// Otherwise, `None`.
    pub fn read_temperature_from_frame(
        &self,
        frame: &[u8],
    ) -> Option<[f32; NUM_TEMPERATURE_CHANNEL]> {
        if frame.len() != BYTES_RESPONSE_TEMPERATURE {
            return None;
        }

        let response = from_utf8(frame).ok()?;
        let re = Regex::new(r"C\d+=(\d+\.\d+)").ok()?;

        let mut temperatures = [0.0; NUM_TEMPERATURE_CHANNEL];
        for (idx, captures) in re
            .captures_iter(response)
            .enumerate()
            .take(NUM_TEMPERATURE_CHANNEL)
        {
            temperatures[idx] = self.parse_capture(&captures, 1)?;
        }

        Some(temperatures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::mock::mock_constants::{
        PLANT_PIER_FAN_MAX_SPEED, PLANT_PIER_FAN_REF_DC_LINK_CURRENT,
        PLANT_PIER_FAN_REF_DC_LINK_VOLTAGE, PLANT_PRESSURE, PLANT_TEMPERATURE,
    };
    use crate::mock::mock_plant::MockPlant;

    fn create_communicator_and_plant() -> (ModbusCommunicator, MockPlant) {
        (ModbusCommunicator::new(), MockPlant::new())
    }

    #[test]
    fn test_create_frame_read_flowmeter_and_read() {
        let (communicator, plant) = create_communicator_and_plant();

        let frame_request = communicator.create_frame_read_flowmeter(7);
        let frame_response = plant.request_sensor_flowmeter(1, &frame_request);

        let flowmeter = communicator.read_flowmeter_from_frame(&frame_response.unwrap());

        assert!(flowmeter.is_some());
    }

    #[test]
    fn test_create_frame_read_power_grid_monitor_and_read() {
        let (communicator, plant) = create_communicator_and_plant();

        let frame_request = communicator.create_frame_read_power_grid_monitor(2);
        let frame_response = plant.request_power_grid_monitor(&frame_request);

        let power_grid_monitor =
            communicator.read_power_grid_monitor_from_frame(&frame_response.unwrap());

        assert!(power_grid_monitor.is_some());
    }

    #[test]
    fn test_create_frame_read_pier_fan_max_speed_and_read() {
        let (communicator, mut plant) = create_communicator_and_plant();

        let frame_request = communicator.create_frame_read_pier_fan_max_speed(12);
        let frame_response = plant.request_pier_fan(&frame_request).unwrap();

        let max_speed = communicator.read_pier_fan_max_speed_from_frame(&frame_response);

        assert_eq!(max_speed.unwrap(), PLANT_PIER_FAN_MAX_SPEED as f32);
    }

    #[test]
    fn test_create_frame_read_pier_fan_ref_dc_link_voltage_current_and_read() {
        let (communicator, mut plant) = create_communicator_and_plant();

        let frame_request =
            communicator.create_frame_read_pier_fan_ref_dc_link_voltage_and_current(12);
        let frame_response = plant.request_pier_fan(&frame_request).unwrap();

        let (voltage, current) = communicator
            .read_pier_fan_dc_link_voltage_and_current_from_frame(&frame_response)
            .unwrap();

        // Note the unit change from mV to V and from mA to A
        assert_eq!(voltage, PLANT_PIER_FAN_REF_DC_LINK_VOLTAGE * 0.001);
        assert_eq!(current, PLANT_PIER_FAN_REF_DC_LINK_CURRENT * 0.001);
    }

    #[test]
    fn test_create_frame_read_pier_fan_from_frame_and_read() {
        let (communicator, mut plant) = create_communicator_and_plant();

        let frame_request = communicator.create_frame_read_pier_fan(12);
        let frame_response = plant.request_pier_fan(&frame_request).unwrap();

        let pier_fan = communicator.read_pier_fan_from_frame(
            PLANT_PIER_FAN_MAX_SPEED as f32,
            PLANT_PIER_FAN_REF_DC_LINK_VOLTAGE * 0.001,
            PLANT_PIER_FAN_REF_DC_LINK_CURRENT * 0.001,
            &frame_response,
        );

        assert!(pier_fan.is_some());
    }

    #[test]
    fn test_create_frame_read_recirculation_pump_and_read() {
        let (communicator, mut plant) = create_communicator_and_plant();

        let frame_request = communicator.create_frame_read_recirculation_pump(1);
        let frame_response_cim_configuration =
            plant.request_recirculation_pump(&frame_request[0]).unwrap();
        let frame_response_control = plant.request_recirculation_pump(&frame_request[1]).unwrap();
        let frame_response_status = plant.request_recirculation_pump(&frame_request[2]).unwrap();
        let frame_response_data = plant.request_recirculation_pump(&frame_request[3]).unwrap();

        let recirculation_pump = communicator.read_recirculation_pump_from_frame(
            &frame_response_cim_configuration,
            &frame_response_control,
            &frame_response_status,
            &frame_response_data,
        );

        assert!(recirculation_pump.is_some());
    }

    #[test]
    fn test_create_frame_read_chiller_and_read() {
        let (communicator, mut plant) = create_communicator_and_plant();

        let frame_request = communicator.create_frame_read_chiller(1);
        let frame_response = plant.request_chiller(0, &frame_request).unwrap();

        let chiller = communicator.read_chiller_from_frame(&frame_response);

        assert!(chiller.is_some());
    }

    #[test]
    fn test_create_frame_pier_fan_reset() {
        let (communicator, mut plant) = create_communicator_and_plant();

        let frame_request = communicator.create_frame_pier_fan_reset(12);
        let frame_response = plant.request_pier_fan(&frame_request).unwrap();

        assert_eq!(frame_response, frame_request);
    }

    #[test]
    fn test_create_frame_chiller_set_temperature() {
        let (communicator, mut plant) = create_communicator_and_plant();

        let temperature = 75;
        let frame_request = communicator.create_frame_chiller_set_temperature(0, temperature);
        let frame_response = plant.request_chiller(0, &frame_request).unwrap();

        assert_eq!(frame_response, frame_request);
    }

    #[test]
    fn test_create_frame_read_pressure() {
        let communicator = ModbusCommunicator::new();

        assert_eq!(
            communicator.create_frame_read_pressure(5),
            b"#005P\r\n".to_vec()
        );
        assert_eq!(
            communicator.create_frame_read_pressure(20),
            b"#020P\r\n".to_vec()
        );
    }

    #[test]
    fn test_read_pressure_from_frame() {
        let (communicator, plant) = create_communicator_and_plant();

        let address = 20;
        let frame_request = communicator.create_frame_read_pressure(address);
        let frame_response = plant.request_sensor_pressure(0, &frame_request);

        let pressure = communicator.read_pressure_from_frame(&frame_response.unwrap());

        assert_eq!(pressure.unwrap(), (address, PLANT_PRESSURE));
    }

    #[test]
    fn test_read_temperature_from_frame() {
        let (communicator, plant) = create_communicator_and_plant();

        let frame_response = plant.request_sensor_temperatures(0);
        let temperatures = communicator.read_temperature_from_frame(&frame_response.unwrap());

        assert_eq!(
            temperatures.unwrap(),
            [PLANT_TEMPERATURE; NUM_TEMPERATURE_CHANNEL]
        );
    }
}
