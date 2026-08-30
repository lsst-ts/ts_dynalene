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

// Temperature in degree Celsius
pub const PLANT_TEMPERATURE: f32 = 0.51;

// Pressure in PSI
pub const PLANT_PRESSURE: f32 = 14.7;

pub const PLANT_FLOWMETER_SIGNAL_STRENGTH: f32 = 2.4;

// The unit of flow rate is liter/minute.
pub const PLANT_FLOWMETER_FLOW_RATE: f32 = 1.2345678;

// The unit is liter.
pub const PLANT_FLOWMETER_POSITIVE_TOTALIZER: f32 = 3.5;

// The unit is liter.
pub const PLANT_FLOWMETER_NEGATIVE_TOTALIZER: f32 = 1.1;

// The unit is liter.
pub const PLANT_FLOWMETER_NET_TOTALIZER: f32 =
    PLANT_FLOWMETER_POSITIVE_TOTALIZER - PLANT_FLOWMETER_NEGATIVE_TOTALIZER;

// The unit is amperes.
pub const PLANT_POWER_GRID_MONITOR_CURRENT_AVG: f32 = 1.23;

// The unit is volts.
pub const PLANT_POWER_GRID_MONITOR_VOLTAGE_LL_AVG: f32 = 234.53;
pub const PLANT_POWER_GRID_MONITOR_VOLTAGE_LN_AVG: f32 = 120.21;

// The unit is kW.
pub const PLANT_POWER_GRID_MONITOR_ACTIVE_POWER_TOTAL: f32 = 456.73;

// The unit is kVAR.
pub const PLANT_POWER_GRID_MONITOR_REACTIVE_POWER_TOTAL: f32 = 89.13;

// The unit is kVA.
pub const PLANT_POWER_GRID_MONITOR_APPARENT_POWER_TOTAL: f32 = 500.24;

// No unit.
pub const PLANT_POWER_GRID_MONITOR_POWER_FACTOR_TOTAL: f32 = 0.95;

// The unit is Hz.
pub const PLANT_POWER_GRID_MONITOR_FREQUENCY: f32 = 100.0;

// Maximum speed of the pier fan in rpm.
pub const PLANT_PIER_FAN_MAX_SPEED: u16 = 6000;

// Reference DC link voltage of the pier fan in mV.
pub const PLANT_PIER_FAN_REF_DC_LINK_VOLTAGE: f32 = 15000.0;

// Reference DC link current of the pier fan in mA.
pub const PLANT_PIER_FAN_REF_DC_LINK_CURRENT: f32 = 5000.0;

// The unit is rpm.
pub const PLANT_PIER_FAN_ACTUAL_SPEED: f32 = 5600.0;

// The unit is volts.
pub const PLANT_PIER_FAN_DC_LINK_VOLTAGE: f32 = 14.97;

// The unit is amperes.
pub const PLANT_PIER_FAN_DC_LINK_CURRENT: f32 = 4.89;

// The unit is degrees Celsius.
pub const PLANT_PIER_FAN_MODULE_TEMPERATURE: u16 = 12;
pub const PLANT_PIER_FAN_MOTOR_TEMPERATURE: i16 = 15;
pub const PLANT_PIER_FAN_ELECTRONICS_TEMPERATURE: u16 = 13;

// No unit.
pub const PLANT_PIER_FAN_CURRENT_DIRECTION_OF_ROTATION: u16 = 1;

// The unit is percentage.
pub const PLANT_PIER_FAN_CURRENT_MODULATION_LEVEL: f32 = 5.72;

// The unit is rpm.
pub const PLANT_PIER_FAN_CURRENT_SET_VALUE: f32 = 5700.0;

// No unit.
pub const PLANT_PIER_FAN_ENABLE_INPUT_STATUS: u16 = 1;
pub const PLANT_PIER_FAN_CURRENT_PARAMETER_SET: u16 = 1;
pub const PLANT_PIER_FAN_CURRENT_CONTROLLER_FUNCTION: u16 = 1;

// The unit is watts.
pub const PLANT_PIER_FAN_CURRENT_POWER: f32 = 71.23;
