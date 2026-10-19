# Class Diagram

There are the following modules in the control system:

- [daq](#daq)
- [interface](#interface)
- [telemetry](#telemetry)
- [command](#command)
- [mock](#mock)

Show the main class diagram below:

```mermaid
classDiagram

class Config
class EventQueue
```

## Daq

The [daq](../src/daq/) module implements the data acquisition process:

```mermaid
classDiagram

namespace main {
  class Config
  class EventQueue
}

namespace telemetry {
  class Telemetry
  class TelemetryDataAcquisition
}

Telemetry o-- TelemetryDataAcquisition

namespace mock {
  class MockPlant
}

namespace command {
  class CommandSchema
  class CommandDataAcquisition
}

RecirculationPump o-- RecirculationPumpCimConfiguration
RecirculationPump o-- RecirculationPumpControl
RecirculationPump o-- RecirculationPumpStatus
RecirculationPump o-- RecirculationPumpData

ModbusCommunicator ..> Flowmeter
ModbusCommunicator ..> PowerGridMonitor
ModbusCommunicator ..> PierFan
ModbusCommunicator ..> RecirculationPump
ModbusCommunicator ..> Chiller

DataAcquisition *-- Config
DataAcquisition *-- EventQueue
DataAcquisition *-- TelemetryDataAcquisition
DataAcquisition "1" *-- "2" TankLevel
DataAcquisition *-- ModbusCommunicator
DataAcquisition o-- MockPlant

CommandSchema "1" *-- "n" CommandDataAcquisition

CommandDataAcquisition --> DataAcquisition

DataAcquisitionProcess *-- DataAcquisition
DataAcquisitionProcess *-- CommandSchema
DataAcquisitionProcess ..> Telemetry
```

## Interface

The [interface](../src/interface/) module supports the TCP/IP communication:

```mermaid
classDiagram

namespace telemetry {
  class Event
}

CommandTelemetryServer --> CommandServer
CommandTelemetryServer --> TelemetryServer
CommandTelemetryServer ..> Event
```

## Telemetry

The [telemetry](../src/telemetry/) module supports the telemetry and event:

```mermaid
classDiagram

namespace daq {
  class Chiller
  class Flowmeter
  class PierFan
  class PowerGridMonitor
  class RecirculationPump
  class RecirculationPumpCimConfiguration
  class RecirculationPumpControl
  class RecirculationPumpStatus
  class RecirculationPumpData
}

RecirculationPump o-- RecirculationPumpCimConfiguration
RecirculationPump o-- RecirculationPumpControl
RecirculationPump o-- RecirculationPumpStatus
RecirculationPump o-- RecirculationPumpData

TelemetryDefault <|-- TelemetryDataAcquisition

TelemetryDataAcquisition ..> Chiller
TelemetryDataAcquisition ..> Flowmeter
TelemetryDataAcquisition ..> PierFan
TelemetryDataAcquisition ..> PowerGridMonitor
TelemetryDataAcquisition ..> RecirculationPump

Telemetry o-- TelemetryDataAcquisition

class Event
```

## Command

The [command](../src/command/) module implements the command factory:

```mermaid
classDiagram

namespace daq {
  class DataAcquisition
}

CommandSchema "1" *-- "n" Command

Command --> DataAcquisition
Command <|-- CommandDataAcquisition
```

## Mock

The [mock](../src/mock/) module supports the simulation mode:

```mermaid
classDiagram

namespace main {
  class Config
}

namespace daq {
  class Flowmeter
  class PowerGridMonitor
  class PierFan
  class RecirculationPump
  class Chiller
}

MockPressureTransducerGroup "1" *-- "n" MockPressureTransducer
MockFlowmeterGroup "1" *-- "n" Flowmeter
MockPowerGridMonitor *-- PowerGridMonitor
MockPierFan *-- PierFan
MockRecirculationPump *-- RecirculationPump
MockChiller *-- Chiller

MockPlant ..> Config
MockPlant "1" *-- "3" MockTemperatureHub
MockPlant "1" *-- "3" MockPressureTransducerGroup
MockPlant "1" *-- "3" MockFlowmeterGroup
MockPlant "1" *-- "n" MockPowerGridMonitor
MockPlant "1" *-- "n" MockPierFan
MockPlant "1" *-- "2" MockRecirculationPump
MockPlant "1" *-- "2" MockChiller
```
