# Version History

0.1.7

- Remove the underscore of fields.
- Change the package name and add the **features** to `Cargo.toml`.
- Add the dependencies of `flexi_logger`, `libc`, `signal-hook`, and `clap`.
- Update the `parameters_app.yaml`, `README.md`, and `class_diagram.md`.
- Add the `logspecification.toml`, `dynalene` script, `deployment.md`, and `communication_diagram.md`.
- Report the instrument status after starting the instrument in `MockPlant.update_digital_inputs()`.
- Fix the `TelemetryServer.process_telemetry()` to detect the close of connection from peer.
- Add the classes of **Status**, **Controller**, **CommandSwitchCommandSource**, and **Model**.
- Add the `application.rs`.
- Update the `event.rs` and `command_format.rs`, and `main.rs`.

0.1.6

- Add the **CommandStatus** and **Commander** to `enums.rs`.
- Update the `constants.rs`.
- Update the `parameters_app.yaml`.
- Update the `utility.rs`.
- Update the **Config**.
- Update the **DataAcquisition**.
- Add the **CommandFormat**, **CommandSchema**, **CommandDataAcquisition**, **DataAcquisitionProcess**, and **Event**.
- Copy and modify the **TelemetryServer**, **CommandServer**, and **CommandTelemetryServer** from the **ts_mtm2_controller**.
- Add the `communication_protocol.md`.
- Update the `class_diagram.md` and `README.md`.

0.1.5

- Add the dependencies of `serde`, `serde_json`, and `log`.
- Derive the **Clone** to DAQ classes.
- Support the **Default** in `recirculation_pump.rs`.
- Update the `constants.rs.`
- Update the `parameters_app.yaml`.
- Add the **EventQueue**, **TelemetryDefault**, **TelemetryDataAcquisition**, and **Telemetry** classes.
- Update the **DataAcquisition** class.
- Update the class diagram.

0.1.4

- Add the dependency of `strum` and `regex`.
- Add the **BYTES_HOLDING_REGISTER** and **MAX_ANALOG_INPUT_OUTPUT** to `constants.rs.`.
- Add the chiller and tank level sensor settings to `parameters_app.yaml` and update the **Config**.
- Add the **Chiller**, **MockChiller**, **ModbusCommunicator**, **TankLevel**, and **DataAcquisition** classes.
- Add the **PLANT_TANK_LEVEL_VOLTAGE** and **PLANT_NUM_CHANNEL_ANALOG_INPUT_OUTPUT** to `mock_constants.rs`.
- Add the digital/analog input/output enums to `enums.rs`.
- Update the **MockPlant** class and **PierFan** class.
- Update the class diagram.
- Update the **README.md**.

0.1.3

- Use the `rubincr.lsst.org` for docker image in `Jenkinsfile`.

0.1.2

- Add the dependency of `strum_macros`.
- Add the **addresses_recirculation_pump** to `parameters_app.yaml` and update the **Config**.
- Add the **RecirculationPump** class and **MockRecirculationPump** class.
- Update the **MockPlant** class.
- Update the class diagram.
- Update the **README.md**.

0.1.1

- Add the dependencies of `crc`, `approx`, and `ts_control_utils`.
- Add the daq classes: **Flowmeter**, **PowerGridMonitor**, and **PierFan**.
- Add the mock classes: **MockFlowmeterGroup**, **MockPressureTransducerGroup**, **MockPowerGridMonitor**, and **MockPierFan**.
- Add the `utility.rs` and `enums.rs`.
- Update the `constants.rs` and `mock_constants.rs`.
- Remove the **Default** in **MockPressureTransducer**.
- Use the `f32` instead of `f64` in **MockPressureTransducer** and **MockTemperatureHub**.
- Add the `parameters_app.yaml`.
- Add the **Config**.
- Update the **MockPlant**.
- Update the class diagram.
- Update the **README.md**.

0.1.0

- Initial version to setup the repository.
- Add the prototype of plant model.
