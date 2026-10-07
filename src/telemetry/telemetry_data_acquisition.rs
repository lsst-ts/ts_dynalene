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

use serde_json::Value;
use std::collections::HashMap;

use crate::constants::{
    NUM_CMV, NUM_PCV, NUM_TANK, NUM_TEMPERATURE_CHANNEL, NUM_TEMPERATURE_HUB,
    NUMS_PRESSURE_TRANSDUCER,
};
use crate::daq::{
    chiller::Chiller, flowmeter::Flowmeter, pier_fan::PierFan,
    power_grid_monitor::PowerGridMonitor, recirculation_pump::RecirculationPump,
};
use crate::telemetry::telemetry_default::TelemetryDefault;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TelemetryDataAcquisition {
    // Temperatures in degrees Celsius.
    pub temperatures: Vec<f32>,
    // Pressures in PSI.
    pub pressures: Vec<f32>,
    // Power grid monitors.
    pub power_grid_monitors: Vec<PowerGridMonitor>,
    // Flowmeters.
    pub flowmeters: Vec<Flowmeter>,
    // Pier fans.
    pub pier_fans: Vec<PierFan>,
    // Recirculation pumps.
    pub recirculation_pumps: Vec<RecirculationPump>,
    // Chillers.
    pub chillers: Vec<Chiller>,
    // Percentages of the pressure control valve (PCV).
    pub percentages_pcv: Vec<f64>,
    // Percentages of the control mixing valve (CMV).
    pub percentages_cmv: Vec<f64>,
    // Tank levels.
    pub tank_levels: HashMap<String, Vec<f64>>,
    // Digital inputs (Mod4, NI-9425).
    pub digital_inputs_mod4: u32,
    // Digital inputs (Mod7, NI-9425).
    pub digital_inputs_mod7: u32,
    // Digital outputs (Mod6, NI-9476).
    pub digital_outputs: u32,
}

impl TelemetryDefault for TelemetryDataAcquisition {
    fn get_messages(&self, digit: i32) -> Vec<Value> {
        vec![]
    }
}

impl TelemetryDataAcquisition {
    /// Create a new data acquisition telemetry object.
    pub fn new() -> Self {
        Self {
            temperatures: vec![0.0; NUM_TEMPERATURE_HUB * NUM_TEMPERATURE_CHANNEL],
            pressures: vec![0.0; NUMS_PRESSURE_TRANSDUCER.iter().sum()],
            flowmeters: Vec::new(),
            power_grid_monitors: Vec::new(),
            pier_fans: Vec::new(),
            recirculation_pumps: Vec::new(),
            chillers: Vec::new(),
            percentages_pcv: vec![0.0; NUM_PCV],
            percentages_cmv: vec![0.0; NUM_CMV],
            tank_levels: Self::initialize_dict_vector(&["distance", "percentage"], 0.0, NUM_TANK),

            digital_inputs_mod4: 0,
            digital_inputs_mod7: 0,

            digital_outputs: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_messages() {
        let telemetry = TelemetryDataAcquisition::new();

        let messages_off = telemetry.get_messages(2);

        assert_eq!(messages_off.len(), 0);
    }
}
