# Class Diagram

There are the following modules in the control system:

- [daq](#daq)
- [mock](#mock)

Show the main class diagram below:

```mermaid
classDiagram

class Config
```

## Daq

The [daq](../src/daq/) module implements the data acquisition process:

```mermaid
classDiagram

class Flowmeter
class PowerGridMonitor
class PierFan

RecirculationPump *-- RecirculationPumpCimConfiguration
RecirculationPump *-- RecirculationPumpControl
RecirculationPump *-- RecirculationPumpStatus
RecirculationPump *-- RecirculationPumpData
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
}

MockPressureTransducerGroup "1" *-- "n" MockPressureTransducer
MockFlowmeterGroup "1" *-- "n" Flowmeter
MockPowerGridMonitor *-- PowerGridMonitor
MockPierFan *-- PierFan
MockRecirculationPump *-- RecirculationPump

MockPlant ..> Config
MockPlant "1" *-- "3" MockTemperatureHub
MockPlant "1" *-- "3" MockPressureTransducerGroup
MockPlant "1" *-- "3" MockFlowmeterGroup
MockPlant "1" *-- "n" MockPowerGridMonitor
MockPlant "1" *-- "n" MockPierFan
MockPlant "1" *-- "2" MockRecirculationPump
```
