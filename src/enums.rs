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

use ts_control_utils::enums::BitEnum;

impl BitEnum<u16> for MotorStatusPierFan {
    fn value(&self) -> u16 {
        *self as u16
    }
}

impl BitEnum<u16> for WarningPierFan {
    fn value(&self) -> u16 {
        *self as u16
    }
}

// Although the name is motor status, it looks more like the motor error
// status. Keep this name to be consistent with the user manual and existing
// LabVIEW code.
#[derive(Debug, Clone, Copy)]
pub enum MotorStatusPierFan {
    FanBlocked,
    HallFailure,
    MotorOverheated,
    // General error. This is set for every error.
    FanBad,
    // Communication error between master controller and slave controller
    MasterSlaveError,
    PowerModuleOverheated,
    Spare6,
    // 1-phase device
    MainsUndervoltage,
    Spare8,
    Spare9,
    Spare10,
    DcLinkUndervoltage,
    Spare12,
    Spare13,
    Spare14,
    Spare15,
}

#[derive(Debug, Clone, Copy)]
pub enum WarningPierFan {
    // Triggered in instances where an external force causes the motor to run
    // in the wrong direction at high speed for a prolonged period of time so
    // the motor is unable to start properly.
    BrakeOperation,
    DcLinkVoltageLow,
    ElectronicsTemperatureHigh,
    MotorTemperatureHigh,
    PowerModuleTemperatureHigh,
    PowerLimitationCurrentlyEngaged,
    // DC-link voltage unstable -> Line impedance too high.
    LineImpedanceTooHigh,
    CurrentLimitationCurrentlyEngaged,
    SheddingFunctionActive,
    SupplyVoltageHigh,
    Spare10,
    DcLinkVoltageHigh,
    // The motor should not be started when the heating is activated!
    HeatingActivated,
    // Voltage at the analogue input < Limit value for cable break.
    CableBreakAtSetValueAnalogueInput,
    // Actual speed is less than the speed limit set for speed monitoring.
    ActualSpeedBelowLimit,
    Spare15,
}
