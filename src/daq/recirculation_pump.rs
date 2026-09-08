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

use crate::constants::{
    HUNDRED, NUM_REGISTER_RECIRCULATION_PUMP_CIM_CONFIGURATION,
    NUM_REGISTER_RECIRCULATION_PUMP_CONTROL, NUM_REGISTER_RECIRCULATION_PUMP_DATA,
    NUM_REGISTER_RECIRCULATION_PUMP_STATUS, TEN,
};
use crate::enums::{
    AlarmWarningRecirculationPump, ControlModeRecirculationPump, OperationModeRecirculationPump,
};
use crate::utility::get_values_from_u8_array;

#[derive(Debug, PartialEq)]
pub struct RecirculationPumpCimConfiguration {
    // The minimum reply delay from the slave in ms.
    // Value range: 0-10000, i.e. up to 10 seconds reply delay. This delay is
    // typically used in conjunction with a modem. The delay value is stored
    // in the device and remains after a power-off. The delay set here is
    // added to the internal delay in the device.
    // Default setting: 0.
    pub slave_minimum_reply_delay: u16,
    // If the Modbus address switch has not been set correctly, meaning
    // outside the 1 to 247 range, the value in this register is used as Modbus
    // address. The value is stored in the device and remains after a
    // power-off.
    // Note that for CIM 200, this value is used only when you have set the
    // transmission speed to "software-defined" on DIP switches SW4 and
    // SW5. Otherwise, CIM 200 ignores it.
    pub software_defined_modbus_address: u16,
    // Modbus software-defined value for transmission speed.
    // The software-defined value for transmission speed is stored in the
    // device and remains after a power-off.
    // 0: 1200 bit/s
    // 1: 2400 bit/s
    // 2: 4800 bit/s
    // 3: 9600 bit/s
    // 4: 19200 bit/s
    // 5: 38400 bit/s.
    // Note that for CIM 200, this value is used only when you have set the
    // transmission speed to "software-defined" on DIP switches SW4 and
    // SW5. Otherwise, CIM 200 ignores it.
    pub software_defined_bit_rate: u16,
    // Used to select the behaviour of control bit acknowledgements from the
    // CIM/CIU.
    // 0: Disabled.
    // Control bits are not automatically lowered when accepted by the
    // device. The user must lower the triggered control bit manually before
    // the control bit can be triggered again.
    // 1: Enabled.
    // Control bits are automatically lowered when accepted by the device.
    // The user does not have to lower it manually (default).
    pub auto_ack_control_bits: u16,
    // Parity setting when using software-defined settings.
    // 0: No parity (default)
    // 1: Even parity
    // 2: Odd parity.
    // Note that for CIM 200, this value is used only when you set the
    // transmission speed to "software-defined" on DIP switches SW4 and
    // SW5. Otherwise, CIM 200 ignores it.
    pub software_defined_parity: u16,
    // Stop bit setting when using software-defined settings.
    // 0: No stop bit
    // 1: 1 stop bit (default)
    // 2: 2 stop bits.
    // Note that for CIM 200, this value is used only when you set the
    // transmission speed to "software-defined" on DIP switches SW4 and
    // SW5. Otherwise, CIM 200 ignores it.
    pub software_defined_stop_bit: u16,
    // Configuration of fieldbus communication watchdog.
    // 0: Watchdog is disabled (default)
    // 1: Watchdog is enabled, timeout 5 s.
    // Any other value disables the watchdog.
    // Watchdog action: The pump will be set to Local mode.
    //
    // CIM 200: Watchdog is fed whenever serial line data appears on the
    // network. It is not a requirement that valid Modbus telegrams are preset
    // nor that CIM 200 is specifically addressed. An interruption of serial
    // data for more than 5 seconds activates the watchdog.
    //
    // CIM 500: Watchdog is only fed if CIM 500 is specifically addressed with
    // Modbus TCP telegrams, matching IP address. If CIM 500 is connected
    // to a Modbus TCP network but never gets addressed, it will activate
    // watchdog after 5 seconds.
    pub watchdog: u16,
    // For disabling the GENIbus LED2.
    // 0: GENIbus diode LED2 has normal function.
    // 1: GENIbus diode LED2 is permanently switched off.
    pub genibus_diode_off: u16,
    // Holds a CRC error counter for the GENIbus connection to the booster
    // system.
    pub genibus_crc_error_cnt: u16,
    // Holds a data error counter for the GENIbus connection to the booster
    // system.
    pub genibus_data_error_cnt: u16,
    //  A Grundfos-specific version number. This is an unsigned integer value.
    pub version_number: u16,
    // Holds the current Modbus slave address of the device.
    // Valid value range: 1...247.
    pub actual_modbus_address: u16,
    // Holds a transmit counter for the total number of telegrams sent to the
    // booster system on the GENIbus connection.
    pub genibus_tx_count: u32,
    // Holds a receive counter for the total number of telegrams received from
    // the booster system on the GENIbus connection.
    pub genibus_rx_count: u32,
    // Grundfos product family.
    pub unit_family: u16,
    // Grundfos product type.
    pub unit_type: u16,
    // Grundfos product version.
    pub unit_version: u16,
    // Product software version (aa.bb.cc.dd).
    pub product_software_version: String,
    //  Product software date (dd/mm/yyyy).
    pub product_software_date: String,
}

#[derive(Debug, PartialEq)]
pub struct RecirculationPumpControl {
    // Control status that each bit is defined in
    // `ControlStatusRecirculationPump` in enums.rs.
    pub status: u16,
    // Sets the control mode.
    //
    // MPC: Multi-Pump Controller.
    // DDD: Demand Driven Distribution. A Grundfos system for municipal water
    // supply.
    //
    // Note: If MPC or DDD is set to control source "From the CU 35X", it
    // is still possible to change ControlMode from Modbus if the
    // RemoteAccessReq bit is set to "1".
    //
    // Note: The control mode depends on the primary sensor, and not
    // all booster systems can run with all the control modes.
    //
    // Note: DDD can run with constant pressure, proportional pressure
    // or automatic adaption.
    pub control_mode: ControlModeRecirculationPump,
    // A state to control the operating mode of the booster system.
    //
    // Note that "OnOffReq" has higher priority than the OperationMode,
    // meaning that you must set "OnOffReq" to "On" for the OperationMode to
    // have any effect.
    // The Hydro Multi-B can only run in auto-control mode.
    // The Hydro Multi-E model G can only run in auto-control mode and
    // OpenLoopMax.
    pub operation_mode: OperationModeRecirculationPump,
    // Sets the setpoint of the booster in percent.
    //
    // Closed loop:
    // MPC, Multi-E: Percentage of closed-loop feedback sensor maximum value.
    // TPED, MAGNA3-D: Percentage of the setpoint range.
    //
    // Open loop:
    // MPC, Multi-E: Percentage of the maximum performance.
    // TPED, MAGNA3-D: Percentage of the nominal pump frequency.
    pub setpoint: f32,
    // Forces the control of pump 1.
    // Default is auto-control.
    // 0: Auto-control (controlled by the booster system)
    // 2: Forced stop.
    pub control_pump_1: u16,
}

#[derive(Debug, PartialEq)]
pub struct RecirculationPumpStatus {
    // Status that each bit is defined in `StatusRecirculationPump` in
    // enums.rs.
    pub status: u16,
    // Indicates the actual process feedback from the booster system in
    // percent.
    // This value can be compared with the setpoint value.
    //
    // Closed loop:
    // MPC, Multi-E: Percentage of closed-loop feedback sensor maximum value.
    // TPED, MAGNA3-D: Percentage of the setpoint range.
    //
    // Open loop:
    // MPC, Multi-E: Percentage of the maximum performance.
    // TPED, MAGNA3-D: Percentage of the nominal pump frequency.
    pub process_feedback: f32,
    // Indicates the actual control mode. See
    // `RecirculationPumpControl.control_mode` for more details.
    pub control_mode: ControlModeRecirculationPump,
    // Indicates the actual operating mode.
    pub operation_mode: OperationModeRecirculationPump,
    // The Grundfos-specific alarm code.
    pub alarm_code: AlarmWarningRecirculationPump,
    // The Grundfos-specific warning code.
    pub warning_code: AlarmWarningRecirculationPump,
    // Indicates presence of pumps. Each bit is defined in
    // `PumpIdRecirculationPump` in enums.rs.
    //
    // A bit value of "1" indicates that the pump is present.
    pub pumps_present: u8,
    // Indicates the running status of pumps. Each bit is defined in
    // `PumpIdRecirculationPump` in enums.rs.
    //
    // A bit value of "1" indicates that the pump is running.
    pub pumps_running: u8,
    // Indicates alarm status of pumps. Each bit is defined in
    // `PumpIdRecirculationPump` in enums.rs.
    //
    // A bit value of "1" indicates that the pump has an alarm.
    pub pumps_fault: u8,
    // Indicates communication status of pumps. Each bit is defined in
    // `PumpIdRecirculationPump` in enums.rs.
    //
    // A bit value of "1" indicates that there is no communication with the
    // pump.
    pub pumps_comm_fault: u8,
    // Indicates active system functions. Each bit is defined in
    // `SystemActiveFunctionRecirculationPump` in enums.rs.
    //
    // Hydro MPC supports all bits.
    // Hydro Multi-B only supports bits 7 (RemoteBusActive)
    // and 12 (LowFlowStopActive).
    pub system_active_functions: u16,
}

#[derive(Debug, PartialEq)]
pub struct RecirculationPumpData {
    // Performance relative to maximum performance in percent.
    pub relative_performance: f32,
    // Actual setpoint, according to control mode. The unit is percent.
    pub actual_setpoint: f32,
    // Actual motor current in amperes.
    pub motor_current: f32,
    // Total power consumption of the system in watts.
    pub power: u32,
    // Total operating time of the system in hours.
    pub operation_time: u32,
    // Total energy consumption of the system in kWh.
    pub energy: u32,
    // Setpoint before modifications in percent.
    pub user_setpoint: f32,
}

#[derive(Debug, PartialEq)]
pub struct RecirculationPump {
    // Address of the recirculation pump.
    pub address: u8,
    // CIM configuration.
    pub cim_configuration: Option<RecirculationPumpCimConfiguration>,
    // Control configuration.
    pub control: Option<RecirculationPumpControl>,
    // Status information.
    pub status: Option<RecirculationPumpStatus>,
    // Data measurements.
    pub data: Option<RecirculationPumpData>,
}

impl RecirculationPump {
    /// Creates a new instance of `RecirculationPump` with default values.
    ///
    /// # Arguments
    /// * `address` - The address of the recirculation pump.
    ///
    /// # Returns
    /// A new instance of `RecirculationPump`.
    pub fn new(address: u8) -> Self {
        Self {
            address,

            cim_configuration: None,
            control: None,
            status: None,
            data: None,
        }
    }

    /// Creates a new instance of `RecirculationPump` from the provided
    /// elements.
    ///
    /// # Arguments
    /// * `frame_cim_configuration` - The Modbus frame containing the CIM
    ///   configuration.
    /// * `frame_control` - The Modbus frame containing the control
    ///   configuration.
    /// * `frame_status` - The Modbus frame containing the status information.
    /// * `frame_data` - The Modbus frame containing the data measurements.
    ///
    /// # Returns
    /// A new instance of `RecirculationPump`.
    pub fn from_frame(
        frame_cim_configuration: &[u8],
        frame_control: &[u8],
        frame_status: &[u8],
        frame_data: &[u8],
    ) -> RecirculationPump {
        let mut address = 0;
        let cim_configuration = Self::from_frame_cim_configuration(frame_cim_configuration);
        if let Some(configuration) = &cim_configuration {
            address = configuration.actual_modbus_address as u8;
        }

        RecirculationPump {
            address,

            cim_configuration,
            control: Self::from_frame_control(frame_control),
            status: Self::from_frame_status(frame_status),
            data: Self::from_frame_data(frame_data),
        }
    }

    /// Create a `RecirculationPumpCimConfiguration` instance from a Modbus
    /// frame.
    ///
    /// # Arguments
    /// * `frame` - The Modbus frame containing the recirculation pump CIM
    ///   configuration.
    ///
    /// # Returns
    /// An `Option` containing the `RecirculationPumpCimConfiguration` if the
    /// frame is valid, or `None` otherwise.
    fn from_frame_cim_configuration(frame: &[u8]) -> Option<RecirculationPumpCimConfiguration> {
        const DATA_BYTES_RECIRCULATION_PUMP_CIM_CONFIGURATION: usize =
            2 * (NUM_REGISTER_RECIRCULATION_PUMP_CIM_CONFIGURATION as usize);
        const FRAME_LENGTH_RECIRCULATION_PUMP_CIM_CONFIGURATION: usize =
            5 + DATA_BYTES_RECIRCULATION_PUMP_CIM_CONFIGURATION;
        if (frame.len() != FRAME_LENGTH_RECIRCULATION_PUMP_CIM_CONFIGURATION)
            || (frame[2] != (DATA_BYTES_RECIRCULATION_PUMP_CIM_CONFIGURATION as u8))
        {
            return None;
        }

        // Index is: 3 + (address - 1) * 2. See
        // REGISTER_ADDRESS_RECIRCULATION_PUMP_CIM_CONFIGURATION in
        // constants.rs.
        //
        // SlaveMinimumReplyDelay, address: 00001
        // SoftwareDefinedModbusAddress, address: 00003
        // SoftwareDefinedBitRate, address: 00004
        // AutoAckControlBits, address: 00005
        // SoftwareDefinedParity, address: 00009
        // SoftwareDefinedStopBit, address: 00010
        // Watchdog, address: 00012
        // GENIbusDiodeOff, address: 00013
        let group_1 = get_values_from_u8_array::<u16, 13>(&frame[3..29])?;

        // GENIbusCRCErrorCnt, address: 00021
        // GENIbusDataErrorCnt, address: 00022
        // VersionNumber, address: 00023
        // ActualModbusAddress, address: 00024
        let group_2 = get_values_from_u8_array::<u16, 4>(&frame[43..51])?;

        // GENIbusTXcountHI, address: 00025
        // GENIbusTXcountLO, address: 00026
        // GENIbusRXcountHI, address: 00027
        // GENIbusRXcountLO, address: 00028
        let group_3 = get_values_from_u8_array::<u32, 2>(&frame[51..59])?;

        // UnitFamily, address: 00030
        // UnitType, address: 00031
        // UnitVersion, address: 00032
        let group_4 = get_values_from_u8_array::<u16, 3>(&frame[61..67])?;

        // ProductSoftwareVersionHI, address: 00034
        // Product software version - BCD digit 1-4 aa.bb
        //
        // ProductSoftwareVersionLO, address: 00035
        // Product software version - BCD digit 5-8 cc.dd
        let product_software_version = format!(
            "{}{}.{}{}.{}{}.{}{}",
            frame[69] >> 4,
            frame[69] & 0x0F,
            frame[70] >> 4,
            frame[70] & 0x0F,
            frame[71] >> 4,
            frame[71] & 0x0F,
            frame[72] >> 4,
            frame[72] & 0x0F
        );

        // ProductSoftwareDayMonth, address: 00036
        // Product software date - BCD ddmm
        //
        // ProductSoftwareYear, address: 00037
        // Product software date - BCD yyyy
        let product_software_date = format!(
            "{}{}/{}{}/{}{}{}{}",
            frame[73] >> 4,
            frame[73] & 0x0F,
            frame[74] >> 4,
            frame[74] & 0x0F,
            frame[75] >> 4,
            frame[75] & 0x0F,
            frame[76] >> 4,
            frame[76] & 0x0F
        );

        Some(RecirculationPumpCimConfiguration {
            // Group 1
            slave_minimum_reply_delay: group_1[0],
            software_defined_modbus_address: group_1[2],
            software_defined_bit_rate: group_1[3],
            auto_ack_control_bits: group_1[4],
            software_defined_parity: group_1[8],
            software_defined_stop_bit: group_1[9],
            watchdog: group_1[11],
            genibus_diode_off: group_1[12],
            // Group 2
            genibus_crc_error_cnt: group_2[0],
            genibus_data_error_cnt: group_2[1],
            version_number: group_2[2],
            actual_modbus_address: group_2[3],
            // Group 3
            genibus_tx_count: group_3[0],
            genibus_rx_count: group_3[1],
            // Group 4
            unit_family: group_4[0],
            unit_type: group_4[1],
            unit_version: group_4[2],

            product_software_version,
            product_software_date,
        })
    }

    /// Create a `RecirculationPumpControl` instance from a Modbus frame.
    ///
    /// # Arguments
    /// * `frame` - The Modbus frame containing the recirculation pump control.
    ///
    /// # Returns
    /// An `Option` containing the `RecirculationPumpControl` if the frame is
    /// valid, or `None` otherwise.
    fn from_frame_control(frame: &[u8]) -> Option<RecirculationPumpControl> {
        const DATA_BYTES_RECIRCULATION_PUMP_CONTROL: usize =
            2 * (NUM_REGISTER_RECIRCULATION_PUMP_CONTROL as usize);
        const FRAME_LENGTH_RECIRCULATION_PUMP_CONTROL: usize =
            5 + DATA_BYTES_RECIRCULATION_PUMP_CONTROL;
        if (frame.len() != FRAME_LENGTH_RECIRCULATION_PUMP_CONTROL)
            || (frame[2] != (DATA_BYTES_RECIRCULATION_PUMP_CONTROL as u8))
        {
            return None;
        }

        // Index is: 3 + (address - 101) * 2. See
        // REGISTER_ADDRESS_RECIRCULATION_PUMP_CONTROL in constants.rs.
        //
        // Status, address: 00101
        // ControlMode, address: 00102
        // OperationMode, address: 00103
        // Setpoint, address: 00104
        // ControlPump1, address: 00105
        let values = get_values_from_u8_array::<
            u16,
            { NUM_REGISTER_RECIRCULATION_PUMP_CONTROL as usize },
        >(&frame[3..(3 + DATA_BYTES_RECIRCULATION_PUMP_CONTROL)])?;

        Some(RecirculationPumpControl {
            status: values[0],
            control_mode: ControlModeRecirculationPump::from_repr(values[1] as u8)?,
            operation_mode: OperationModeRecirculationPump::from_repr(values[2] as u8)?,
            setpoint: (values[3] as f32) / HUNDRED,
            control_pump_1: values[4],
        })
    }

    /// Create a `RecirculationPumpStatus` instance from a Modbus frame.
    ///
    /// # Arguments
    /// * `frame` - The Modbus frame containing the recirculation pump status.
    ///
    /// # Returns
    /// An `Option` containing the `RecirculationPumpStatus` if the frame is
    /// valid, or `None` otherwise.
    fn from_frame_status(frame: &[u8]) -> Option<RecirculationPumpStatus> {
        const DATA_BYTES_RECIRCULATION_PUMP_STATUS: usize =
            2 * (NUM_REGISTER_RECIRCULATION_PUMP_STATUS as usize);
        const FRAME_LENGTH_RECIRCULATION_PUMP_STATUS: usize =
            5 + DATA_BYTES_RECIRCULATION_PUMP_STATUS;
        if (frame.len() != FRAME_LENGTH_RECIRCULATION_PUMP_STATUS)
            || (frame[2] != (DATA_BYTES_RECIRCULATION_PUMP_STATUS as u8))
        {
            return None;
        }

        // Index is: 3 + (address - 201) * 2. See
        // REGISTER_ADDRESS_RECIRCULATION_PUMP_STATUS in constants.rs.
        //
        // Status, address: 00201
        // ProcessFeedback, address: 00202
        // ControlMode, address: 00203
        // OperationMode, address: 00204
        // AlarmCode, address: 00205
        // WarningCode, address: 00206
        // PumpsPresent, address: 00208
        // PumpsRunning, address: 00209
        // PumpsFault, address: 00210
        // PumpsCommFault, address: 00211
        // SystemActiveFunctions, address: 00212
        let values = get_values_from_u8_array::<
            u16,
            { NUM_REGISTER_RECIRCULATION_PUMP_STATUS as usize },
        >(&frame[3..(3 + DATA_BYTES_RECIRCULATION_PUMP_STATUS)])?;

        Some(RecirculationPumpStatus {
            status: values[0],
            process_feedback: (values[1] as f32) / HUNDRED,
            control_mode: ControlModeRecirculationPump::from_repr(values[2] as u8)?,
            operation_mode: OperationModeRecirculationPump::from_repr(values[3] as u8)?,
            alarm_code: AlarmWarningRecirculationPump::from_repr(values[4] as u8)?,
            warning_code: AlarmWarningRecirculationPump::from_repr(values[5] as u8)?,
            pumps_present: values[7] as u8,
            pumps_running: values[8] as u8,
            pumps_fault: values[9] as u8,
            pumps_comm_fault: values[10] as u8,
            system_active_functions: values[11],
        })
    }

    /// Create a `RecirculationPumpData` instance from a Modbus frame.
    ///
    /// # Arguments
    /// * `frame` - The Modbus frame containing the recirculation pump data.
    ///
    /// # Returns
    /// An `Option` containing the `RecirculationPumpData` if the frame is
    /// valid, or `None` otherwise.
    fn from_frame_data(frame: &[u8]) -> Option<RecirculationPumpData> {
        const DATA_BYTES_RECIRCULATION_PUMP_DATA: usize =
            2 * (NUM_REGISTER_RECIRCULATION_PUMP_DATA as usize);
        const FRAME_LENGTH_RECIRCULATION_PUMP_DATA: usize = 5 + DATA_BYTES_RECIRCULATION_PUMP_DATA;
        if (frame.len() != FRAME_LENGTH_RECIRCULATION_PUMP_DATA)
            || (frame[2] != (DATA_BYTES_RECIRCULATION_PUMP_DATA as u8))
        {
            return None;
        }

        // Index is: 3 + (address - 301) * 2. See
        // REGISTER_ADDRESS_RECIRCULATION_PUMP_DATA in constants.rs.

        // RelativePerformance, address: 00303
        let relative_performance =
            (get_values_from_u8_array::<u16, 1>(&frame[7..9])?[0] as f32) / HUNDRED;

        // ActualSetpoint, address: 00308
        let actual_setpoint =
            (get_values_from_u8_array::<u16, 1>(&frame[17..19])?[0] as f32) / HUNDRED;

        // MotorCurrent, address: 00309
        let motor_current = (get_values_from_u8_array::<u16, 1>(&frame[19..21])?[0] as f32) / TEN;

        // PowerHI, address: 00312
        // PowerLO, address: 00313
        let power = get_values_from_u8_array::<u32, 1>(&frame[25..29])?[0];

        // OperationTimeHI, address: 00327
        // OperationTimeLO, address: 00328
        let operation_time = get_values_from_u8_array::<u32, 1>(&frame[55..59])?[0];

        // EnergyHI, address: 00332
        // EnergyLO, address: 00333
        let energy = get_values_from_u8_array::<u32, 1>(&frame[65..69])?[0];

        // UserSetpoint, address: 00343
        let user_setpoint =
            (get_values_from_u8_array::<u16, 1>(&frame[87..89])?[0] as f32) / HUNDRED;

        Some(RecirculationPumpData {
            relative_performance,
            actual_setpoint,
            motor_current,
            power,
            operation_time,
            energy,
            user_setpoint,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_frame_invalid() {
        let recirculation_pump = RecirculationPump::from_frame(&[], &[], &[], &[]);
        assert_eq!(recirculation_pump.address, 0);

        assert!(recirculation_pump.cim_configuration.is_none());
        assert!(recirculation_pump.control.is_none());
        assert!(recirculation_pump.status.is_none());
        assert!(recirculation_pump.data.is_none());
    }

    #[test]
    fn test_from_frame_cim_configuration_invalid() {
        // Frame with incorrect length
        let frame_short: [u8; 78] = [0; 78];
        assert!(RecirculationPump::from_frame_cim_configuration(&frame_short).is_none());

        // Frame with incorrect data bytes
        let mut frame_wrong_data_bytes: [u8; 79] = [0; 79];
        frame_wrong_data_bytes[2] = 73;
        assert!(RecirculationPump::from_frame_cim_configuration(&frame_wrong_data_bytes).is_none());
    }

    #[test]
    fn test_from_frame_control_invalid() {
        // Frame with incorrect length
        let frame_short: [u8; 14] = [0; 14];
        assert!(RecirculationPump::from_frame_control(&frame_short).is_none());

        // Frame with incorrect data bytes
        let mut frame_wrong_data_bytes: [u8; 15] = [0; 15];
        frame_wrong_data_bytes[2] = 9;
        assert!(RecirculationPump::from_frame_control(&frame_wrong_data_bytes).is_none());
    }

    #[test]
    fn test_from_frame_status_invalid() {
        // Frame with incorrect length
        let frame_short: [u8; 28] = [0; 28];
        assert!(RecirculationPump::from_frame_status(&frame_short).is_none());

        // Frame with incorrect data bytes
        let mut frame_wrong_data_bytes: [u8; 29] = [0; 29];
        frame_wrong_data_bytes[2] = 23;
        assert!(RecirculationPump::from_frame_status(&frame_wrong_data_bytes).is_none());
    }

    #[test]
    fn test_from_frame_data_invalid() {
        // Frame with incorrect length
        let frame_short: [u8; 90] = [0; 90];
        assert!(RecirculationPump::from_frame_data(&frame_short).is_none());

        // Frame with incorrect data bytes
        let mut frame_wrong_data_bytes: [u8; 91] = [0; 91];
        frame_wrong_data_bytes[2] = 85;
        assert!(RecirculationPump::from_frame_data(&frame_wrong_data_bytes).is_none());
    }
}
