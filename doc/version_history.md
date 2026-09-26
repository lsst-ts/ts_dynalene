# Version History

0.1.4

- Add the dependency of `strum` and `regex`.
- Add the **BYTES_HOLDING_REGISTER** and **MAX_ANALOG_INPUT_OUTPUT** to `constants.rs.`.
- Add the chiller settings to `parameters_app.yaml` and update the **Config**.
- Add the **Chiller**, **MockChiller**, **ModbusCommunicator**, and **DataAcquisition** classes.
- Add the **PLANT_NUM_CHANNEL_ANALOG_INPUT_OUTPUT** to `mock_constants.rs`.
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
