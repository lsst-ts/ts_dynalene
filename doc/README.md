# Software Architecture

This is a multi-threaded application.

## Command Factory Pattern

The [command factory pattern](../src/command/command_schema.rs) is applied in the system.

- [data acquisition](#command-of-the-data-acquisition)

### Command of the Data Acquisition

The followings are the commands:

- CommandSwitchDigitalOutput
- CommandOpenPressureControlValve
- CommandOpenControlMixingValve
- CommandResetPierFan
- CommandSetChillerTemperature

See [here](../src/command/command_data_acquisition.rs) for the details.

## Class Diagram

See [here](class_diagram.md) for the class diagram.
