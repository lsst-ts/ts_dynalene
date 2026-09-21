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

namespace main {
  class Config
}
namespace mock {
  class MockPlant
}

RecirculationPump *-- RecirculationPumpCimConfiguration
RecirculationPump *-- RecirculationPumpControl
RecirculationPump *-- RecirculationPumpStatus
RecirculationPump *-- RecirculationPumpData

ModbusCommunicator ..> Flowmeter
ModbusCommunicator ..> PowerGridMonitor
ModbusCommunicator ..> PierFan
ModbusCommunicator ..> RecirculationPump
ModbusCommunicator ..> Chiller

DataAcquisition *-- Config
DataAcquisition *-- ModbusCommunicator
DataAcquisition o-- MockPlant
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
