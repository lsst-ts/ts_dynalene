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

use serde_json::{Value, json};
use std::collections::HashMap;

use crate::constants::{
    NUM_CMV, NUM_PCV, NUM_TANK, NUM_TEMPERATURE_CHANNEL, NUM_TEMPERATURE_HUB,
    NUMS_PRESSURE_TRANSDUCER,
};
use crate::daq::{
    chiller::Chiller,
    flowmeter::Flowmeter,
    pier_fan::PierFan,
    power_grid_monitor::PowerGridMonitor,
    recirculation_pump::{
        RecirculationPump, RecirculationPumpCimConfiguration, RecirculationPumpControl,
        RecirculationPumpData, RecirculationPumpStatus,
    },
};

use crate::telemetry::telemetry_default::TelemetryDefault;
use crate::utility::fahrenheit_to_celsius;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TelemetryDataAcquisition {
    // Temperatures in degrees Celsius.
    pub temperatures: Vec<f64>,
    // Pressures in PSI.
    pub pressures: Vec<f64>,
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
        let mut messages = vec![
            self.get_message_temperature(digit),
            self.get_message_pressure(digit),
            self.get_message_valve(digit),
            self.get_message_tank_level(digit),
        ];

        for flowmeter in &self.flowmeters {
            if flowmeter.address != 0 {
                messages.push(self.get_message_flowmeter(flowmeter, digit));
            }
        }

        for power_grid_monitor in &self.power_grid_monitors {
            if power_grid_monitor.address != 0 {
                messages.push(self.get_message_power_grid_monitor(power_grid_monitor, digit));
            }
        }

        for pier_fan in &self.pier_fans {
            if pier_fan.address != 0 {
                messages.push(self.get_message_pier_fan(pier_fan, digit));
            }
        }

        for recirculation_pump in &self.recirculation_pumps {
            let address = recirculation_pump.address;
            if address != 0 {
                messages.push(
                    self.get_message_recirculation_pump_cim_configuration(
                        address,
                        recirculation_pump
                            .cim_configuration
                            .as_ref()
                            .expect("Should have the CIM configuration in recirculation pump."),
                    ),
                );
                messages.push(
                    self.get_message_recirculation_pump_control(
                        address,
                        recirculation_pump
                            .control
                            .as_ref()
                            .expect("Should have the control in recirculation pump."),
                        digit,
                    ),
                );
                messages.push(
                    self.get_message_recirculation_pump_status(
                        address,
                        recirculation_pump
                            .status
                            .as_ref()
                            .expect("Should have the status in recirculation pump."),
                        digit,
                    ),
                );
                messages.push(
                    self.get_message_recirculation_pump_data(
                        address,
                        recirculation_pump
                            .data
                            .as_ref()
                            .expect("Should have the data in recirculation pump."),
                        digit,
                    ),
                );
            }
        }

        for (idx, chiller) in self.chillers.iter().enumerate() {
            if chiller.unit_id != 0 {
                messages.push(self.get_message_chiller(idx, chiller, digit));
            }
        }

        messages
    }
}

impl TelemetryDataAcquisition {
    /// Create a new data acquisition telemetry object.
    ///
    /// # Returns
    /// A new instance of `TelemetryDataAcquisition`.
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

    /// Get the message of the temperature.
    ///
    /// # Arguments
    /// * `digit` - The number of digits after the decimal point.
    ///
    /// # Returns
    /// The message of the temperature.
    fn get_message_temperature(&self, digit: i32) -> Value {
        json!({
            "id": "temperature",
            // Unit is degrees Celsius. The addresses are in the order of
            // ["addresses_temperature_hub_0",
            //  "addresses_temperature_hub_1",
            //  "addresses_temperature_hub_2"] assigned in
            // "parameters_app.yaml".
            "temperatureItem": self.round_vector(&self.temperatures, digit),
        })
    }

    /// Get the message of the pressure.
    ///
    /// # Arguments
    /// * `digit` - The number of digits after the decimal point.
    ///
    /// # Returns
    /// The message of the pressure.
    fn get_message_pressure(&self, digit: i32) -> Value {
        json!({
            "id": "pressure",
            // Unit is PSI. The addresses are in the order of
            // ["addresses_pressure_transducer_bus_0",
            //  "addresses_pressure_transducer_bus_1",
            //  "addresses_pressure_transducer_bus_2"] assigned in
            // "parameters_app.yaml".
            "pressureItem": self.round_vector(&self.pressures, digit),
        })
    }

    /// Get the message of the flowmeter.
    ///
    /// # Arguments
    /// * `flowmeter` - The flowmeter to get the message for.
    /// * `digit` - The number of digits after the decimal point.
    ///
    /// # Returns
    /// The message of the flowmeter.
    fn get_message_flowmeter(&self, flowmeter: &Flowmeter, digit: i32) -> Value {
        json!({
            "id": "flowmeter",
            // See the struct Flowmeter in "flowmeter.rs" for the details.
            "address": flowmeter.address,
            "signalStrength": self.f32_to_f64_and_round(flowmeter.signal_strength, digit),
            "flowRate": self.f32_to_f64_and_round(flowmeter.flow_rate, digit),
            "netTotalizer": self.f32_to_f64_and_round(flowmeter.net_totalizer, digit),
            "positiveTotalizer": self.f32_to_f64_and_round(flowmeter.positive_totalizer, digit),
            "negativeTotalizer": self.f32_to_f64_and_round(flowmeter.negative_totalizer, digit),
        })
    }

    /// Convert a f32 value to f64 and round it to the specified number of
    /// digits.
    ///
    /// # Arguments
    /// * `value` - The f32 value to round.
    /// * `digit` - The number of digits after the decimal point.
    ///
    /// # Returns
    /// The rounded f64 value.
    fn f32_to_f64_and_round(&self, value: f32, digit: i32) -> f64 {
        self.round(value as f64, digit)
    }

    /// Get the message of the power grid monitor.
    ///
    /// # Arguments
    /// * `power_grid_monitor` - The power grid monitor to get the message for.
    /// * `digit` - The number of digits after the decimal point.
    ///
    /// # Returns
    /// The message of the power grid monitor.
    fn get_message_power_grid_monitor(
        &self,
        power_grid_monitor: &PowerGridMonitor,
        digit: i32,
    ) -> Value {
        json!({
            "id": "powerGridMonitor",
            // See the struct PowerGridMonitor in "power_grid_monitor.rs" for
            // the details.
            "address": power_grid_monitor.address,
            "currentAvg": self.f32_to_f64_and_round(power_grid_monitor.current_avg, digit),
            "voltageLlAvg": self.f32_to_f64_and_round(power_grid_monitor.voltage_ll_avg, digit),
            "voltageLnAvg": self.f32_to_f64_and_round(power_grid_monitor.voltage_ln_avg, digit),
            "activePowerTotal": self.f32_to_f64_and_round(power_grid_monitor.active_power_total, digit),
            "reactivePowerTotal": self.f32_to_f64_and_round(power_grid_monitor.reactive_power_total, digit),
            "apparentPowerTotal": self.f32_to_f64_and_round(power_grid_monitor.apparent_power_total, digit),
            "powerFactorTotal": self.f32_to_f64_and_round(power_grid_monitor.power_factor_total, digit),
            "frequency": self.f32_to_f64_and_round(power_grid_monitor.frequency, digit),
        })
    }

    /// Get the message of the pier fan.
    ///
    /// # Arguments
    /// * `pier_fan` - The pier fan to get the message for.
    /// * `digit` - The number of digits after the decimal point.
    ///
    /// # Returns
    /// The message of the pier fan.
    fn get_message_pier_fan(&self, pier_fan: &PierFan, digit: i32) -> Value {
        json!({
            "id": "pierFan",
            // See the struct PierFan in "pier_fan.rs" for the details.
            "address": pier_fan.address,
            "actualSpeed": self.f32_to_f64_and_round(pier_fan.actual_speed, digit),
            "motorStatus": pier_fan.motor_status,
            "warning": pier_fan.warning,
            "dcLinkVoltage": self.f32_to_f64_and_round(pier_fan.dc_link_voltage, digit),
            "dcLinkCurrent": self.f32_to_f64_and_round(pier_fan.dc_link_current, digit),
            "moduleTemperature": pier_fan.module_temperature,
            "motorTemperature": pier_fan.motor_temperature,
            "electronicsTemperature": pier_fan.electronics_temperature,
            "currentDirectionOfRotation": pier_fan.current_direction_of_rotation,
            "currentModulationLevel": self.f32_to_f64_and_round(pier_fan.current_modulation_level, digit),
            "currentSetValue": self.f32_to_f64_and_round(pier_fan.current_set_value, digit),
            "enableInputStatus": pier_fan.enable_input_status,
            "currentParameterSet": pier_fan.current_parameter_set,
            "currentControllerFunction": pier_fan.current_controller_function,
            "currentPower": self.f32_to_f64_and_round(pier_fan.current_power, digit),
        })
    }

    /// Get the message of the CIM configuration of the recirculation pump.
    ///
    /// # Arguments
    /// * `address` - The address of the recirculation pump.
    /// * `cim_configuration` - The CIM configuration to get the message for.
    ///
    /// # Returns
    /// The message of the CIM configuration of the recirculation pump.
    fn get_message_recirculation_pump_cim_configuration(
        &self,
        address: u8,
        cim_configuration: &RecirculationPumpCimConfiguration,
    ) -> Value {
        json!({
            "id": "recirculationPumpCimConfiguration",
            "address": address,
            // See the struct RecirculationPumpCimConfiguration in
            // "recirculation_pump.rs" for the details.
            "slaveMinimumReplyDelay": cim_configuration.slave_minimum_reply_delay,
            "softwareDefinedModbusAddress": cim_configuration.software_defined_modbus_address,
            "softwareDefinedBitRate": cim_configuration.software_defined_bit_rate,
            "autoAckControlBits": cim_configuration.auto_ack_control_bits,
            "softwareDefinedParity": cim_configuration.software_defined_parity,
            "softwareDefinedStopBit": cim_configuration.software_defined_stop_bit,
            "watchdog": cim_configuration.watchdog,
            "genibusDiodeOff": cim_configuration.genibus_diode_off,
            "genibusCrcErrorCnt": cim_configuration.genibus_crc_error_cnt,
            "genibusDataErrorCnt": cim_configuration.genibus_data_error_cnt,
            "versionNumber": cim_configuration.version_number,
            "actualModbusAddress": cim_configuration.actual_modbus_address,
            "genibusTxCount": cim_configuration.genibus_tx_count,
            "genibusRxCount": cim_configuration.genibus_rx_count,
            "unitFamily": cim_configuration.unit_family,
            "unitType": cim_configuration.unit_type,
            "unitVersion": cim_configuration.unit_version,
            "productSoftwareVersion": cim_configuration.product_software_version,
            "productSoftwareDate": cim_configuration.product_software_date,
        })
    }

    /// Get the message of the control of the recirculation pump.
    ///
    /// # Arguments
    /// * `address` - The address of the recirculation pump.
    /// * `control` - The control to get the message for.
    /// * `digit` - The number of digits after the decimal point.
    ///
    /// # Returns
    /// The message of the control of the recirculation pump.
    fn get_message_recirculation_pump_control(
        &self,
        address: u8,
        control: &RecirculationPumpControl,
        digit: i32,
    ) -> Value {
        json!({
            "id": "recirculationPumpControl",
            "address": address,
            // See the struct RecirculationPumpControl in
            // "recirculation_pump.rs" for the details.
            "status": control.status,
            "controlMode": control.control_mode as u8,
            "operationMode": control.operation_mode as u8,
            "setpoint": self.f32_to_f64_and_round(control.setpoint, digit),
            "controlPump1": control.control_pump_1,
        })
    }

    /// Get the message of the status of the recirculation pump.
    ///
    /// # Arguments
    /// * `address` - The address of the recirculation pump.
    /// * `status` - The status to get the message for.
    /// * `digit` - The number of digits after the decimal point.
    ///
    /// # Returns
    /// The message of the status of the recirculation pump.
    fn get_message_recirculation_pump_status(
        &self,
        address: u8,
        status: &RecirculationPumpStatus,
        digit: i32,
    ) -> Value {
        json!({
            "id": "recirculationPumpStatus",
            "address": address,
            // See the struct RecirculationPumpStatus in
            // "recirculation_pump.rs" for the details.
            "status": status.status,
            "processFeedback": self.f32_to_f64_and_round(status.process_feedback, digit),
            "controlMode": status.control_mode as u8,
            "operationMode": status.operation_mode as u8,
            "alarmCode": status.alarm_code as u8,
            "warningCode": status.warning_code as u8,
            "pumpsPresent": status.pumps_present,
            "pumpsRunning": status.pumps_running,
            "pumpsFault": status.pumps_fault,
            "pumpsCommFault": status.pumps_comm_fault,
            "systemActiveFunctions": status.system_active_functions,
        })
    }

    /// Get the message of the data of the recirculation pump.
    ///
    /// # Arguments
    /// * `address` - The address of the recirculation pump.
    /// * `data` - The data of the recirculation pump to get the message for.
    /// * `digit` - The number of digits after the decimal point.
    ///
    /// # Returns
    /// The message of the data of the recirculation pump.
    fn get_message_recirculation_pump_data(
        &self,
        address: u8,
        data: &RecirculationPumpData,
        digit: i32,
    ) -> Value {
        json!({
            "id": "recirculationPumpData",
            "address": address,
            // See the struct RecirculationPumpData in
            // "recirculation_pump.rs" for the details.
            "relativePerformance": self.f32_to_f64_and_round(data.relative_performance, digit),
            "actualSetpoint": self.f32_to_f64_and_round(data.actual_setpoint, digit),
            "motorCurrent": self.f32_to_f64_and_round(data.motor_current, digit),
            "power": data.power,
            "operationTime": data.operation_time,
            "energy": data.energy,
            "userSetpoint": self.f32_to_f64_and_round(data.user_setpoint, digit),
        })
    }

    /// Get the message of the chiller.
    ///
    /// # Arguments
    /// * `idx` - The index of the chiller (0-1). This is the same order of
    ///   "ips_chiller" in the "parameters_app.yaml".
    /// * `chiller` - The chiller to get the message for.
    /// * `digit` - The number of digits after the decimal point.
    ///
    /// # Returns
    /// The message of the chiller.
    fn get_message_chiller(&self, idx: usize, chiller: &Chiller, digit: i32) -> Value {
        json!({
            "id": "chiller",
            "index": idx,
            // See the struct Chiller in "chiller.rs" for the details.
            // All temperatures are converted from Fahrenheit to Celsius to be
            // consistent with the rest of the system.
            "unitId": chiller.unit_id,
            "temperatureSetpoint": self.fahrenheit_to_celsius_and_round(chiller.temperature_setpoint as f32, digit),
            "temperatureHighAlarm": self.fahrenheit_to_celsius_and_round(chiller.temperature_high_alarm as f32, digit),
            "temperatureLowAlarm": self.fahrenheit_to_celsius_and_round(chiller.temperature_low_alarm as f32, digit),
            "toProcessTemperature": self.fahrenheit_to_celsius_and_round(chiller.to_process_temperature as f32, digit),
            "fromProcessTemperature": self.fahrenheit_to_celsius_and_round(chiller.from_process_temperature as f32, digit),
            "processStatus": chiller.process_status,
            "machineStatus1": chiller.machine_status_1,
            "machineStatus2": chiller.machine_status_2,
            "machineStatus3": chiller.machine_status_3,
            "heartbeat": chiller.heartbeat,
            "zone1TemperatureEvaporatorIn": self.fahrenheit_to_celsius_and_round(chiller.zone_1_temperature_evaporator_in as f32, digit),
            "zone1TemperatureEvaporatorOut": self.fahrenheit_to_celsius_and_round(chiller.zone_1_temperature_evaporator_out as f32, digit),
            "zone2TemperatureEvaporatorIn": self.fahrenheit_to_celsius_and_round(chiller.zone_2_temperature_evaporator_in as f32, digit),
            "zone2TemperatureEvaporatorOut": self.fahrenheit_to_celsius_and_round(chiller.zone_2_temperature_evaporator_out as f32, digit),
            "zone1Status": chiller.zone_1_status,
            "zone2Status": chiller.zone_2_status,
        })
    }

    /// Convert Fahrenheit to Celsius and round to the specified number of
    /// digits.
    ///
    /// # Arguments
    /// * `fahrenheit` - The temperature in Fahrenheit.
    /// * `digit` - The number of digits after the decimal point.
    ///
    /// # Returns
    /// The temperature in Celsius, rounded to the specified number of digits.
    fn fahrenheit_to_celsius_and_round(&self, fahrenheit: f32, digit: i32) -> f64 {
        self.f32_to_f64_and_round(fahrenheit_to_celsius(fahrenheit), digit)
    }

    /// Get the message of the percentage of valve.
    ///
    /// # Arguments
    /// * `digit` - The number of digits after the decimal point.
    ///
    /// # Returns
    /// The message of the valve.
    fn get_message_valve(&self, digit: i32) -> Value {
        json!({
            "id": "valve",
            // The percentage to open the valve (0.0 to 100.0). The order is
            // [ReadoutPcv1. ReadoutPcv2]. See the AnalogInput in enums.rs.
            "pcv": self.round_vector(&self.percentages_pcv, digit),
            // The percentage to open the valve (0.0 to 100.0). The order is
            // [ReadoutCmv1. ReadoutCmv2, ReadoutCmv20]. See the AnalogInput in
            // enums.rs.
            "cmv": self.round_vector(&self.percentages_cmv, digit),
        })
    }

    /// Get the message of the tank level.
    ///
    /// # Arguments
    /// * `digit` - The number of digits after the decimal point.
    ///
    /// # Returns
    /// The message of the tank level.
    fn get_message_tank_level(&self, digit: i32) -> Value {
        json!({
            "id": "tankLevel",
            // Distance to full in millimeters.
            "distance": self.round_vector(&self.tank_levels["distance"], digit),
            // Remaining percentage (0-100).
            "percentage": self.round_vector(&self.tank_levels["percentage"], digit)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_value;

    #[test]
    fn test_get_messages() {
        let mut telemetry = TelemetryDataAcquisition::new();

        // No special devices
        let messages_no_device = telemetry.get_messages(2);

        assert_eq!(messages_no_device.len(), 4);

        // Add the special device with zero addresses
        telemetry.flowmeters.push(Flowmeter::new(0));
        telemetry.power_grid_monitors.push(PowerGridMonitor::new(0));
        telemetry.pier_fans.push(PierFan::new(0));
        telemetry
            .recirculation_pumps
            .push(RecirculationPump::new(0));
        telemetry.chillers.push(Chiller::new(0));

        let messages_with_device_address_zero = telemetry.get_messages(2);

        assert_eq!(messages_with_device_address_zero.len(), 4);

        // Add the special device with non-zero addresses
        telemetry.flowmeters.push(Flowmeter::new(1));
        telemetry.power_grid_monitors.push(PowerGridMonitor::new(1));
        telemetry.pier_fans.push(PierFan::new(1));
        telemetry.chillers.push(Chiller::new(1));

        let recirculation_pump = RecirculationPump {
            address: 1,
            cim_configuration: Some(RecirculationPumpCimConfiguration::default()),
            control: Some(RecirculationPumpControl::default()),
            status: Some(RecirculationPumpStatus::default()),
            data: Some(RecirculationPumpData::default()),
        };
        telemetry.recirculation_pumps.push(recirculation_pump);

        let messages_with_device_address_non_zero = telemetry.get_messages(2);

        assert_eq!(messages_with_device_address_non_zero.len(), 12);
    }

    #[test]
    fn test_get_message_temperature() {
        let mut telemetry = TelemetryDataAcquisition::new();

        let num = telemetry.temperatures.len();
        telemetry.temperatures = vec![25.1234; num];

        let message = telemetry.get_message_temperature(2);

        assert_eq!(message["id"], "temperature");
        assert_eq!(
            from_value::<Vec<f64>>(message["temperatureItem"].clone()).unwrap(),
            vec![25.12; num]
        );
    }

    #[test]
    fn test_get_message_pressure() {
        let mut telemetry = TelemetryDataAcquisition::new();

        let num = telemetry.pressures.len();
        telemetry.pressures = vec![14.7123; num];

        let message = telemetry.get_message_pressure(2);

        assert_eq!(message["id"], "pressure");
        assert_eq!(
            from_value::<Vec<f64>>(message["pressureItem"].clone()).unwrap(),
            vec![14.71; num]
        );
    }

    #[test]
    fn test_get_message_flowmeter() {
        let telemetry = TelemetryDataAcquisition::new();

        let mut flowmeter = Flowmeter::new(1);
        flowmeter.signal_strength = 12.345;
        flowmeter.flow_rate = 7.891;
        flowmeter.net_totalizer = 123.456;
        flowmeter.positive_totalizer = 78.914;
        flowmeter.negative_totalizer = 12.345;

        let message = telemetry.get_message_flowmeter(&flowmeter, 2);

        assert_eq!(message["id"], "flowmeter");
        assert_eq!(message["address"], flowmeter.address);
        assert_eq!(message["signalStrength"], 12.35);
        assert_eq!(message["flowRate"], 7.89);
        assert_eq!(message["netTotalizer"], 123.46);
        assert_eq!(message["positiveTotalizer"], 78.91);
        assert_eq!(message["negativeTotalizer"], 12.35);
    }

    #[test]
    fn test_f32_to_f64_and_round() {
        let telemetry = TelemetryDataAcquisition::new();

        assert_eq!(telemetry.f32_to_f64_and_round(3.14159, 2), 3.14);
    }

    #[test]
    fn test_get_message_power_grid_monitor() {
        let telemetry = TelemetryDataAcquisition::new();

        let mut power_grid_monitor = PowerGridMonitor::new(1);
        power_grid_monitor.current_avg = 12.345;
        power_grid_monitor.voltage_ll_avg = 123.456;
        power_grid_monitor.voltage_ln_avg = 78.914;
        power_grid_monitor.active_power_total = 45.678;
        power_grid_monitor.reactive_power_total = 34.567;
        power_grid_monitor.apparent_power_total = 56.789;
        power_grid_monitor.power_factor_total = 0.987;
        power_grid_monitor.frequency = 50.123;

        let message = telemetry.get_message_power_grid_monitor(&power_grid_monitor, 2);

        assert_eq!(message["id"], "powerGridMonitor");
        assert_eq!(message["currentAvg"], 12.35);
        assert_eq!(message["voltageLlAvg"], 123.46);
        assert_eq!(message["voltageLnAvg"], 78.91);
        assert_eq!(message["activePowerTotal"], 45.68);
        assert_eq!(message["reactivePowerTotal"], 34.57);
        assert_eq!(message["apparentPowerTotal"], 56.79);
        assert_eq!(message["powerFactorTotal"], 0.99);
        assert_eq!(message["frequency"], 50.12);
    }

    #[test]
    fn test_get_message_pier_fan() {
        let telemetry = TelemetryDataAcquisition::new();

        let mut pier_fan = PierFan::new(1);
        pier_fan.actual_speed = 123.456;
        pier_fan.dc_link_voltage = 456.789;
        pier_fan.dc_link_current = 12.345;
        pier_fan.current_modulation_level = 0.123;
        pier_fan.current_set_value = 0.456;
        pier_fan.current_power = 7.891;

        let message = telemetry.get_message_pier_fan(&pier_fan, 2);

        assert_eq!(message["id"], "pierFan");
        assert_eq!(message["address"], pier_fan.address);
        assert_eq!(message["actualSpeed"], 123.46);
        assert_eq!(message["motorStatus"], pier_fan.motor_status);
        assert_eq!(message["warning"], pier_fan.warning);
        assert_eq!(message["dcLinkVoltage"], 456.79);
        assert_eq!(message["dcLinkCurrent"], 12.35);
        assert_eq!(message["moduleTemperature"], pier_fan.module_temperature);
        assert_eq!(message["motorTemperature"], pier_fan.motor_temperature);
        assert_eq!(
            message["electronicsTemperature"],
            pier_fan.electronics_temperature
        );
        assert_eq!(
            message["currentDirectionOfRotation"],
            pier_fan.current_direction_of_rotation
        );
        assert_eq!(message["currentModulationLevel"], 0.12);
        assert_eq!(message["currentSetValue"], 0.46);
        assert_eq!(message["enableInputStatus"], pier_fan.enable_input_status);
        assert_eq!(
            message["currentParameterSet"],
            pier_fan.current_parameter_set
        );
        assert_eq!(
            message["currentControllerFunction"],
            pier_fan.current_controller_function
        );
        assert_eq!(message["currentPower"], 7.89);
    }

    #[test]
    fn test_get_message_recirculation_pump_cim_configuration() {
        let telemetry = TelemetryDataAcquisition::new();

        let cim_configuration = RecirculationPumpCimConfiguration::default();
        let message =
            telemetry.get_message_recirculation_pump_cim_configuration(1, &cim_configuration);

        assert_eq!(message["id"], "recirculationPumpCimConfiguration");
        assert_eq!(message["address"], 1);
        assert_eq!(
            message["slaveMinimumReplyDelay"],
            cim_configuration.slave_minimum_reply_delay
        );
        assert_eq!(
            message["softwareDefinedModbusAddress"],
            cim_configuration.software_defined_modbus_address
        );
        assert_eq!(
            message["softwareDefinedBitRate"],
            cim_configuration.software_defined_bit_rate
        );
        assert_eq!(
            message["autoAckControlBits"],
            cim_configuration.auto_ack_control_bits
        );
        assert_eq!(
            message["softwareDefinedParity"],
            cim_configuration.software_defined_parity
        );
        assert_eq!(
            message["softwareDefinedStopBit"],
            cim_configuration.software_defined_stop_bit
        );
        assert_eq!(message["watchdog"], cim_configuration.watchdog);
        assert_eq!(
            message["genibusDiodeOff"],
            cim_configuration.genibus_diode_off
        );
        assert_eq!(
            message["genibusCrcErrorCnt"],
            cim_configuration.genibus_crc_error_cnt
        );
        assert_eq!(
            message["genibusDataErrorCnt"],
            cim_configuration.genibus_data_error_cnt
        );
        assert_eq!(message["versionNumber"], cim_configuration.version_number);
        assert_eq!(
            message["actualModbusAddress"],
            cim_configuration.actual_modbus_address
        );
        assert_eq!(
            message["genibusTxCount"],
            cim_configuration.genibus_tx_count
        );
        assert_eq!(
            message["genibusRxCount"],
            cim_configuration.genibus_rx_count
        );
        assert_eq!(message["unitFamily"], cim_configuration.unit_family);
        assert_eq!(message["unitType"], cim_configuration.unit_type);
        assert_eq!(message["unitVersion"], cim_configuration.unit_version);
        assert_eq!(
            message["productSoftwareVersion"],
            cim_configuration.product_software_version
        );
        assert_eq!(
            message["productSoftwareDate"],
            cim_configuration.product_software_date
        );
    }

    #[test]
    fn test_get_message_recirculation_pump_control() {
        let telemetry = TelemetryDataAcquisition::new();

        let mut control = RecirculationPumpControl::default();
        control.setpoint = 12.456;

        let message = telemetry.get_message_recirculation_pump_control(1, &control, 2);

        assert_eq!(message["id"], "recirculationPumpControl");
        assert_eq!(message["address"], 1);
        assert_eq!(message["status"], control.status);
        assert_eq!(message["controlMode"], control.control_mode as u8);
        assert_eq!(message["operationMode"], control.operation_mode as u8);
        assert_eq!(message["setpoint"], 12.46);
        assert_eq!(message["controlPump1"], control.control_pump_1);
    }

    #[test]
    fn test_get_message_recirculation_pump_status() {
        let telemetry = TelemetryDataAcquisition::new();

        let mut status = RecirculationPumpStatus::default();
        status.process_feedback = 12.345;

        let message = telemetry.get_message_recirculation_pump_status(1, &status, 2);

        assert_eq!(message["id"], "recirculationPumpStatus");
        assert_eq!(message["address"], 1);
        assert_eq!(message["status"], status.status);
        assert_eq!(message["processFeedback"], 12.35);
        assert_eq!(message["controlMode"], status.control_mode as u8);
        assert_eq!(message["operationMode"], status.operation_mode as u8);
        assert_eq!(message["alarmCode"], status.alarm_code as u8);
        assert_eq!(message["warningCode"], status.warning_code as u8);
        assert_eq!(message["pumpsPresent"], status.pumps_present);
        assert_eq!(message["pumpsRunning"], status.pumps_running);
        assert_eq!(message["pumpsFault"], status.pumps_fault);
        assert_eq!(message["pumpsCommFault"], status.pumps_comm_fault);
        assert_eq!(
            message["systemActiveFunctions"],
            status.system_active_functions
        );
    }

    #[test]
    fn test_get_message_recirculation_pump_data() {
        let telemetry = TelemetryDataAcquisition::new();

        let mut data = RecirculationPumpData::default();
        data.relative_performance = 12.345;
        data.actual_setpoint = 67.345;
        data.motor_current = 8.678;
        data.user_setpoint = 45.123;

        let message = telemetry.get_message_recirculation_pump_data(1, &data, 2);

        assert_eq!(message["id"], "recirculationPumpData");
        assert_eq!(message["address"], 1);
        assert_eq!(message["relativePerformance"], 12.35);
        assert_eq!(message["actualSetpoint"], 67.35);
        assert_eq!(message["motorCurrent"], 8.68);
        assert_eq!(message["power"], data.power);
        assert_eq!(message["operationTime"], data.operation_time);
        assert_eq!(message["energy"], data.energy);
        assert_eq!(message["userSetpoint"], 45.12);
    }

    #[test]
    fn test_get_message_chiller() {
        let telemetry = TelemetryDataAcquisition::new();

        let chiller = Chiller::new(1);
        let message = telemetry.get_message_chiller(1, &chiller, 2);

        assert_eq!(message["id"], "chiller");
        assert_eq!(message["index"], 1);
        assert_eq!(message["unitId"], chiller.unit_id);

        let temperature = telemetry.fahrenheit_to_celsius_and_round(0.0, 2);
        assert_eq!(message["temperatureSetpoint"], temperature);
        assert_eq!(message["temperatureHighAlarm"], temperature);
        assert_eq!(message["temperatureLowAlarm"], temperature);
        assert_eq!(message["toProcessTemperature"], temperature,);
        assert_eq!(message["fromProcessTemperature"], temperature,);
        assert_eq!(message["processStatus"], chiller.process_status);
        assert_eq!(message["machineStatus1"], chiller.machine_status_1);
        assert_eq!(message["machineStatus2"], chiller.machine_status_2);
        assert_eq!(message["machineStatus3"], chiller.machine_status_3);
        assert_eq!(message["heartbeat"], chiller.heartbeat);
        assert_eq!(message["zone1TemperatureEvaporatorIn"], temperature,);
        assert_eq!(message["zone1TemperatureEvaporatorOut"], temperature,);
        assert_eq!(message["zone2TemperatureEvaporatorIn"], temperature,);
        assert_eq!(message["zone2TemperatureEvaporatorOut"], temperature,);
        assert_eq!(message["zone1Status"], chiller.zone_1_status);
        assert_eq!(message["zone2Status"], chiller.zone_2_status);
    }

    #[test]
    fn test_fahrenheit_to_celsius_and_round() {
        let telemetry = TelemetryDataAcquisition::new();

        assert_eq!(telemetry.fahrenheit_to_celsius_and_round(17.8, 2), -7.89);
    }

    #[test]
    fn test_get_message_valve() {
        let mut telemetry = TelemetryDataAcquisition::new();
        telemetry.percentages_pcv = vec![50.1234; NUM_PCV];
        telemetry.percentages_cmv = vec![75.5678; NUM_CMV];

        let message = telemetry.get_message_valve(2);

        assert_eq!(message["id"], "valve");
        assert_eq!(
            from_value::<Vec<f64>>(message["pcv"].clone()).unwrap(),
            vec![50.12; NUM_PCV]
        );
        assert_eq!(
            from_value::<Vec<f64>>(message["cmv"].clone()).unwrap(),
            vec![75.57; NUM_CMV]
        );
    }

    #[test]
    fn test_get_message_tank_level() {
        let mut telemetry = TelemetryDataAcquisition::new();
        telemetry
            .tank_levels
            .insert("distance".to_string(), vec![100.1234; NUM_TANK]);
        telemetry
            .tank_levels
            .insert("percentage".to_string(), vec![50.5678; NUM_TANK]);

        let message = telemetry.get_message_tank_level(2);

        assert_eq!(message["id"], "tankLevel");
        assert_eq!(
            from_value::<Vec<f64>>(message["distance"].clone()).unwrap(),
            vec![100.12; NUM_TANK]
        );
        assert_eq!(
            from_value::<Vec<f64>>(message["percentage"].clone()).unwrap(),
            vec![50.57; NUM_TANK]
        );
    }
}
