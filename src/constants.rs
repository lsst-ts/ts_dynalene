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

// Function code to read the contents of a contiguous block of 16-bit
// configuration or analog output registers from a remote slave device.
// See: https://simplymodbus.ca/learn-rtu-fc03.html
///
/// The format of the command is:
/// <address> <function_code> <register_address> <number> <CRC verify code>
/// register_address, number, and CRC verify code have 2 bytes each.
///
/// The format of response to read the holding registers is:
/// <address> <function_code> <data_bytes> <data> <CRC verify code>
pub const CODE_READ_HOLDING_REGISTERS: u8 = 0x03;

// Function code to write a single 16-bit configuration or analog output
// register to a remote slave device.
// See: https://simplymodbus.ca/learn-rtu-fc06.html
///
/// The format of the command is:
/// <address> <function_code> <register_address> <value> <CRC verify code>
/// register_address, value, and CRC verify code have 2 bytes each.
///
/// The response from the slave device echoes the command back if the write is
/// successful.
pub const CODE_WRITE_SINGLE_REGISTER: u8 = 0x06;

pub const NUM_TEMPERATURE_HUB: usize = 3;
pub const NUM_TEMPERATURE_CHANNEL: usize = 8;

pub const NUM_BUS_PRESSURE_TRANSDUCER: usize = 3;

pub const OFFSET_READ_HOLDING_REGISTERS_FLOWMETER: u16 = 40001;

// Register address of the flowmeter relative to the offset of holding
// registers. 40700 (Signal Strength) is used for Big-endian word order master
// devices.
// See svi_FS_BusReadoutNoOpenNoClose.vi in the dynalene_system LabVIEW
// project.
pub const REGISTER_ADDRESS_FLOWMETER: u16 = 40700 - OFFSET_READ_HOLDING_REGISTERS_FLOWMETER;

// Number of the registers to read from the flowmeter.
pub const NUM_REGISTER_FLOWMETER: u16 = 10;

pub const OFFSET_READ_HOLDING_REGISTERS_POWER_GRID_MONITOR: u16 = 1;

// Register address of the power grid monitor relative to the offset of holding
// registers. 3000 (Current A) is used for Big-endian word order master
// devices.
// See svi_SingleMonitorReadout.vi in the dynalene_system LabVIEW
// project.
pub const REGISTER_ADDRESS_POWER_GRID_MONITOR: u16 =
    3000 - OFFSET_READ_HOLDING_REGISTERS_POWER_GRID_MONITOR;

// Number of the registers to read from the power grid monitor.
pub const NUM_REGISTER_POWER_GRID_MONITOR: u16 = 112;

// This value is defined in svi_PowerGrid_DataClassification.vi in the
// dynalene_system LabVIEW project.
pub const UNEXPECTED_VALUE_POWER_GRID_MONITOR_POWER_FACTOR: f32 = 999.0;

// Register addresses of the pier fan. Note that the offset of reading holding
// registers is 0 here.
pub const REGISTER_ADDRESS_PIER_FAN_RESET: u16 = 0xD000;
pub const REGISTER_ADDRESS_PIER_FAN_MAXIMUM_SPEED: u16 = 0xD119;
pub const REGISTER_ADDRESS_PIER_FAN_REFERENCE_VALUE_OF_DC_LINK_VOLTAGE: u16 = 0xD1A0;

// Input register (only read access)
pub const REGISTER_ADDRESS_PIER_FAN_ACTUAL_SPEED: u16 = 0xD010;

// Number of the registers to read from the pier fan.
pub const NUM_REGISTER_PIER_FAN_REFERENCE_VALUE_OF_DC_LINK_VOLTAGE: u16 = 2;
pub const NUM_REGISTER_PIER_FAN_ACTUAL_SPEED: u16 = 18;

pub const MAX_VALUE_PIER_FAN_ACTUAL_SPEED: u16 = 64000;

// I suspect this value should be 255 (= u8::MAX) instead of 256. But the user
// manual specifies 256. Use the value as is.
pub const MAX_VALUE_PIER_FAN_DC_LINK_VOLTAGE_CURRENT: u16 = 256;

// Conversion factor for the register value of the pier fan's DC link reference
// voltage. The register value needs to multiply by this factor to get the
// actual reference voltage in mV.
pub const FACTOR_PIER_FAN_DC_LINK_REFERENCE_VOLTAGE: f32 = 20.0;

// Conversion factor for the register value of the pier fan's DC link reference
// current. The register value needs to multiply by this factor to get the
// actual reference current in mA.
pub const FACTOR_PIER_FAN_DC_LINK_REFERENCE_CURRENT: f32 = 2.0;

// The number of bytes in the response of temperature hub.
pub const BYTES_RESPONSE_TEMPERATURE: usize = 112;

// The number of bytes in the response of pressure transducer.
pub const BYTES_RESPONSE_PRESSURE: usize = 20;
