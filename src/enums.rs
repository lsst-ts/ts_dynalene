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

use strum_macros::FromRepr;
use ts_control_utils::enums::BitEnum;

// Generates `impl BitEnum<$t> for $enum_name` bodies that just cast `self` to
// `$t`, since Rust has no built-in way to derive a custom trait like this.
macro_rules! impl_bit_enum {
    ($t:ty => $($enum_name:ty),+ $(,)?) => {
        $(
            impl BitEnum<$t> for $enum_name {
                fn value(&self) -> $t {
                    *self as $t
                }
            }
        )+
    };
}

impl_bit_enum!(u16 =>
    MotorStatusPierFan,
    WarningPierFan,
    ControlStatusRecirculationPump,
    StatusRecirculationPump,
    SystemActiveFunctionRecirculationPump,
);
impl_bit_enum!(u8 => PumpIdRecirculationPump);

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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ControlStatusRecirculationPump {
    // MPC: Multi-Pump Controller.
    // DDD: Demand Driven Distribution. A Grundfos system for municipal water
    // supply.
    //
    // Control bit that sets local or remote control.
    // 0: Local
    // 1: Remote (controlled by Modbus master).
    //
    // Set this bit to 1 if a Modbus master should control the booster system.
    //
    // For Hydro MPC and DDD, it is also necessary to enable bus control via
    // the CU 35X control panel ("Settings" > "Secondary functions" >
    // "Control source", select "From bus").
    // If the RemoteAccessReq bit is set to logical 0, the Hydro MPC or DDD
    // will operate with local mode settings as selected on the CU 35X control
    // panel. If you want local control, select this from the CU 35X control
    // panel ("Settings" > "Secondary functions" > "Control source", select
    // "From the CU 35X"), and set the RemoteAccessReq bit to 0.
    RemoteAccessReq,
    // Control bit that switches the booster system to on or off.
    // 0: Off (stop)
    // 1: On (start).
    OnOffReq,
    // Control bit that resets alarms and warnings from the booster system.
    // 0: No resetting
    // 1: Resetting alarm.
    //
    // This control bit is triggered on rising edge only, i.e. setting logical
    // 0 to 1.
    ResetAlarm,
    Spare3,
    // Copies ControlMode, OperationMode and Setpoint to Local, when
    // changing from Remote to Local.
    CopyToLocal,
    // Resets the accumulation counters (volume and energy).
    // 0: No resetting
    // 1: Resetting.
    ResetAccCounters,
    Spare6,
    Spare7,
    Spare8,
    Spare9,
    Spare10,
    Spare11,
    Spare12,
    Spare13,
    Spare14,
    Spare15,
}

#[derive(FromRepr, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum ControlModeRecirculationPump {
    ConstantSpeed = 0,
    ConstantFrequency = 1,
    ConstantHead = 3,
    ConstantPressure = 4,
    ConstantDifferentialPressure = 5,
    ProportionalPressure = 6,
    ConstantFlow = 7,
    ConstantTemperature = 8,
    ConstantLevel = 10,
    // Automatic adaption for DDD (Demand Driven Distribution. A Grundfos
    // system for municipal water supply.)
    AutoAdapt = 128,
    FlowAdapt = 129,
    ClosedLoopSensor = 130,
}

#[derive(FromRepr, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum OperationModeRecirculationPump {
    // Setpoint control according to selected control mode
    AutoControl = 0,
    // Running at minimum speed
    OpenLoopMin = 4,
    // Running at maximum speed
    OpenLoopMax = 6,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum StatusRecirculationPump {
    Spare0,
    // Copies remote settings to local settings. 1: Enable, 0: Disable.
    CopyToLocal,
    // Acknowledgement of ResetAccCounters (volume and energy).
    // 0: No acknowledgement
    // 1: Command acknowledged.
    //
    // This functionality is only used when AutoAcknowledgeEvents is disabled.
    ResetAccCountersAck,
    // Indicates if a ResetAlarm command was acknowledged by the device.
    // This bit is set when the module or unit have accepted a ResetAlarm
    // command, and the programmer can clear the ResetAlarm bit. The CIM/CIU
    // will automatically clear the ResetAlarmAck bit to 0 when the master
    // device clears the ResetAlarm bit, and you can attempt a new ResetAlarm
    // command by raising the ResetAlarm bit again.
    // 0: No acknowledgement
    // 1: Command acknowledged.
    //
    // This functionality is only used when AutoAcknowledgeEvents is disabled.
    ResetAlarmAck,
    // Setpoint influence status
    // 0: No setpoint influence
    // 1: Setpoint influence active
    SetpointInfluence,
    // Running at power limit
    // 0: Not at power limit
    // 1: At power limit
    AtMaxPower,
    // Indicates if any pumps are rotating (running) or not.
    // 0: No rotation
    // 1: Rotation.
    Rotation,
    Spare7,
    // Indicates if the booster system is locally or remotely controlled.
    // 0: Local (a local control source with higher priority controls the
    //    system)
    // 1: Remote (controlled by Modbus master).
    AccessMode,
    // Indicates if the booster system is on or off.
    // 0: Off (stopped, the green LED on the booster system flashes)
    // 1: On (started, the green LED on the booster system is on).
    //
    // "Started" does not necessarily indicate rotation, for instance in case
    // of low-flow stop.
    OnOff,
    // Indicates if there is an alarm or not.
    // 0: No alarm
    // 1: Alarm (red LED on the booster system is on).
    Alarm,
    // Indicates if there is a warning or not.
    // 0: No warning
    // 1: Warning (red LED on the booster system is on).
    //
    // The system will continue running even if there is a warning.
    Warning,
    Spare12,
    // Indicates if the system is running at maximum speed.
    // 0: No
    // 1: Yes.
    AtMaxSpeed,
    Spare14,
    // Indicates if the system is running at minimum speed.
    // 0: No
    // 1: Yes.
    AtMinSpeed,
}

#[derive(FromRepr, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum AlarmWarningRecirculationPump {
    None = 0,
    LeakageCurrent = 1,
    MissingPhase = 2,
    ExternalFaultSignal = 3,
    TooManyRestarts = 4,
    RegenerativeBraking = 5,
    MainsFault = 6,
    TooManyHardwareShutdowns = 7,
    PwmSwitchingFrequencyReduced = 8,
    PhaseSequenceReversal = 9,
    CommunicationFaultPump = 10,
    // Motor oil
    WaterInOilFault = 11,
    // General service information
    TimeForService = 12,
    MoistureAlarmAnalog = 13,
    // ERP
    ElectronicDcLinkProtectionActivated = 14,
    // SCADA
    CommunicationFaultMainSystem = 15,
    Other = 16,
    PerformanceRequirementCanNotBeMet = 17,
    // Trip
    CommandedAlarmStandby = 18,
    // Dosing pump
    DiaphragmBreak = 19,
    InsulationResistanceLow = 20,
    TooManyStartsPerHour = 21,
    MoistureSwitchAlarmDigital = 22,
    SmartTrimGapAlarm = 23,
    Vibration = 24,
    SetupConflict = 25,
    LoadContinuesEvenIfMotorSwitchedOff = 26,
    ExternalMotorProtectorActivated = 27,
    BatteryLow = 28,
    // Impellers forced backwards
    TurbineOperation = 29,
    // Specific service information
    ChangeBearings = 30,
    // Specific service information
    ChangeVaristors = 31,
    Overvoltage = 32,
    // Specific service information
    SoonTimeForService = 33,
    NoPrimingWater = 34,
    GasInPumpHeadDeaeratingProblem = 35,
    OutletValveLeakage = 36,
    InletValveLeakage = 37,
    VentValveDefective = 38,
    ValveStuckOrDefective = 39,
    Undervoltage = 40,
    UndervoltageTransient = 41,
    // dV/dt
    CutInFault = 42,
    Spare43 = 43,
    Spare44 = 44,
    VoltageAsymmetry = 45,
    Spare46 = 46,
    Spare47 = 47,
    Overload = 48,
    // i_line, i_dc, i_mo
    Overcurrent = 49,
    MotorProtectionFunctionGeneralShutdown = 50,
    BlockedMotorOrPump = 51,
    MotorSlipHigh = 52,
    StalledMotor = 53,
    // 3 sec limit
    MotorProtectionFunction = 54,
    MotorCurrentProtectionActivated = 55,
    Underload = 56,
    DryRunning = 57,
    LowFlow = 58,
    NoFlow = 59,
    LowInputPower = 60,
    Spare61 = 61,
    Spare62 = 62,
    Spare63 = 63,
    Spare64 = 64,
    // t_m or t_mo or t_mo1
    MotorTemperature1 = 65,
    // t_e
    TemperatureControlElectronics = 66,
    // t_m
    TemperatureTooHighInternalFrequencyConverterModule = 67,
    // t_w
    ExternalTemperatureOrWaterTemperature = 68,
    // For example, Klixon.
    ThermalRelay1InMotor = 69,
    // For example, thermistor.
    ThermalRelay2InMotor = 70,
    // Pt100, t_mo2
    MotorTemperature2 = 71,
    HardwareFaultType1 = 72,
    HardwareShutdown = 73,
    InternalSupplyVoltageTooHigh = 74,
    InternalSupplyVoltageTooLow = 75,
    InternalCommunicationFault = 76,
    CommunicationFaultTwinHeadPump = 77,
    FaultSpeedPlug = 78,
    FunctionalFaultAddonModule = 79,
    HardwareFaultType2 = 80,
    // RAM
    VerificationErrorDataArea = 81,
    // ROM, FLASH
    VerificationErrorCodeArea = 82,
    // EEPROM
    VerificationErrorFeParameterArea = 83,
    MemoryAccessError = 84,
    // EEPROM
    VerificationErrorBeParameterArea = 85,
    // Fault (add-on) I/O module
    FaultIoModule = 86,
    Spare87 = 87,
    SensorFault = 88,
    SignalFaultFeedbackSensor1 = 89,
    SignalFaultSpeedSensor = 90,
    SignalFaultTemperatureSensor1 = 91,
    CalibrationFaultFeedbackSensor = 92,
    SignalFaultSensor2 = 93,
    LimitExceededSensor1 = 94,
    LimitExceededSensor2 = 95,
    SetpointSignalOutsideRange = 96,
    SignalFaultSetpointInput = 97,
    SignalFaultSetpointInfluence = 98,
    SignalFaultAnalogSetpoint = 99,
    RtcTimeSynchronisationWithCellularNetworkOccurred = 100,
    Spare101 = 101,
    DosingPumpNotReady = 102,
    EmergencyStop = 103,
    SoftwareShutdown = 104,
    // ERP
    ElectronicRectifierProtectionActivated = 105,
    // EIP
    ElectronicInverterProtectionActivated = 106,
    Spare107 = 107,
    Spare108 = 108,
    Spare109 = 109,
    SkewLoadElectricalAsymmetry = 110,
    CurrentAsymmetry = 111,
    CosPhiTooHigh = 112,
    CosPhiTooLow = 113,
    // Frost protection
    MotorHeaterFunctionActivated = 114,
    // Too many grinder reversals or grinder reversal attempt failed
    GrinderReversalAttemptFailed = 115,
    GrinderMotorOvertemperature = 116,
    IntrusionDoorOpened = 117,
    SignalFaultHydrogenSulfideH2SSensor = 118,
    SignalFaultAnalogInputAI4 = 119,
    // Single phase motors
    AuxiliaryWindingFault = 120,
    // Single phase motors
    AuxiliaryWindingCurrentTooHigh = 121,
    // Single phase motors
    AuxiliaryWindingCurrentTooLow = 122,
    // Single phase motors
    StartCapacitorLow = 123,
    // Single phase motors
    RunCapacitorLow = 124,
    SignalFaultOutdoorTemperatureSensor = 125,
    SignalFaultAirTemperatureSensor = 126,
    SignalFaultShuntRelativePressureSensor = 127,
    StrainerClogged = 128,
    Spare129 = 129,
    Spare130 = 130,
    Spare131 = 131,
    Spare132 = 132,
    Spare133 = 133,
    Spare134 = 134,
    Spare135 = 135,
    Spare136 = 136,
    Spare137 = 137,
    Spare138 = 138,
    Spare139 = 139,
    Spare140 = 140,
    Spare141 = 141,
    Spare142 = 142,
    Spare143 = 143,
    // Pt100, t_mo3
    MotorTemperature3 = 144,
    // Pt100
    BearingTemperatureHighGeneralOrTopBearing = 145,
    // Pt100
    BearingTemperatureHighMiddleBearing = 146,
    // Pt100
    BearingTemperatureHighBottomBearing = 147,
    // Pt100
    MotorTemperatureHighDriveEnd = 148,
    // Pt100
    MotorTemperatureHighNonDriveEnd = 149,
    // Fault (add-on) pump module
    FaultPumpModule = 150,
    FaultDisplayHmi = 151,
    CommunicationFaultAddonModule = 152,
    FaultAnalogOutput = 153,
    CommunicationFaultDisplay = 154,
    InrushFault = 155,
    CommunicationFaultInternalFrequencyConverterModule = 156,
    RealTimeClockOutOfOrder = 157,
    HardwareCircuitMeasurementFault = 158,
    CommunicationInterfaceModuleFault = 159,
    CellularModemSimCardFault = 160,
    SensorSupplyFault5V = 161,
    SensorSupplyFault24V = 162,
    MeasurementFaultMotorProtection = 163,
    SignalFaultLiqTecSensor = 164,
    SignalFaultAnalogInput1 = 165,
    SignalFaultAnalogInput2 = 166,
    SignalFaultAnalogInput3 = 167,
    SignalFaultPressureSensor = 168,
    SignalFaultFlowSensor = 169,
    SignalFaultWaterInOilSensor = 170,
    SignalFaultMoistureSensor = 171,
    SignalFaultAtmosphericPressureSensor = 172,
    // Hall sensor
    SignalFaultRotorPositionSensor = 173,
    SignalFaultRotorOriginSensor = 174,
    // t_mo2
    SignalFaultTemperatureSensor2 = 175,
    // t_mo3
    SignalFaultTemperatureSensor3 = 176,
    SignalFaultSmartTrimGapSensor = 177,
    SignalFaultVibrationSensor = 178,
    // Pt100
    SignalFaultBearingTemperatureSensorGeneralOrTopBearing = 179,
    // Pt100
    SignalFaultBearingTemperatureSensorMiddleBearing = 180,
    // PTC sensor (short-circuited)
    SignalFaultPtcSensor = 181,
    // Pt100
    SignalFaultBearingTemperatureSensorBottomBearing = 182,
    SignalFaultExtraTemperatureSensor = 183,
    SignalFaultGeneralPurposeSensor = 184,
    UnknownSensorType = 185,
    SignalFaultPowerMeterSensor = 186,
    SignalFaultEnergyMeter = 187,
    SignalFaultUserDefinedSensor = 188,
    SignalFaultLevelSensor = 189,
    // For example, alarm level in WW application
    LimitExceededSensor1InApplication = 190,
    // For example, high level in WW application
    LimitExceededSensor2InApplication = 191,
    // For example, overflow level in WW application
    LimitExceededSensor3InApplication = 192,
    // For example, low level in WW/tank filling application
    LimitExceededSensor4InApplication = 193,
    LimitExceededSensor5InApplication = 194,
    LimitExceededSensor6InApplication = 195,
    OperationWithReducedEfficiency = 196,
    OperationWithReducedPressure = 197,
    OperationWithIncreasedPowerConsumption = 198,
    // Monitoring, estimation, calculation, control
    ProcessOutOfRange = 199,
    ApplicationAlarm = 200,
    ExternalSensorInputHigh = 201,
    ExternalSensorInputLow = 202,
    AlarmOnAllPumps = 203,
    InconsistencyBetweenSensors = 204,
    LevelFloatSwitchSequenceInconsistency = 205,
    WaterShortageLevel1 = 206,
    WaterLeakage = 207,
    Cavitation = 208,
    NonReturnValveFault = 209,
    HighPressure = 210,
    LowPressure = 211,
    DiaphragmTankPrechargePressureOutOfRange = 212,
    // VFD: variable frequency drive
    VfdNotReady = 213,
    WaterShortageLevel2 = 214,
    SoftPressureBuildupTimeout = 215,
    PilotPumpAlarm = 216,
    AlarmGeneralPurposeSensorHigh = 217,
    AlarmGeneralPurposeSensorLow = 218,
    PressureReliefNotAdequate = 219,
    FaultMotorContactorFeedback = 220,
    FaultMixerContactorFeedback = 221,
    TimeForServiceMixer = 222,
    TimeForServiceMixerDuplicate = 223,
    PumpFaultDueToAuxiliaryComponentOrGeneralFault = 224,
    CommunicationFaultPumpModule = 225,
    CommunicationFaultIoModule = 226,
    CombiEvent = 227,
    NightFlowMaxLimitExceeded = 228,
    WaterOnFloor = 229,
    NetworkAlarm = 230,
    EthernetNoIpAddressFromDhcpServer = 231,
    EthernetAutoDisabledDueToMisuse = 232,
    EthernetIpAddressConflict = 233,
    BackupPumpAlarm = 234,
    GasDetected = 235,
    Pump1Fault = 236,
    Pump2Fault = 237,
    Pump3Fault = 238,
    Pump4Fault = 239,
    // Specific service information
    LubricateBearings = 240,
    MotorPhaseFailure = 241,
    AutomaticMotorModelRecognitionFailed = 242,
    // Manually operated or commanded
    MotorRelayHasBeenForced = 243,
    // Fault, On/Off/Auto switch
    FaultOnOffAutoSwitch = 244,
    PumpContinuousRuntimeTooLong = 245,
    // Manually operated or commanded
    UserDefinedRelayHasBeenForced = 246,
    // Device or system has been switched off
    PowerOnNotice = 247,
    FaultBatteryUps = 248,
    UserDefinedEvent1 = 249,
    UserDefinedEvent2 = 250,
    UserDefinedEvent3 = 251,
    UserDefinedEvent4 = 252,
    // DDD: Demand Driven Distribution. A Grundfos system for municipal water
    // supply.
    SmsDataFromDddSensorNotReceivedWithinTimeLimit = 253,
    InconsistentDataModel = 254,
    Spare255 = 255,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PumpIdRecirculationPump {
    Pump1,
    Pump2,
    Pump3,
    Pump4,
    Pump5,
    Pump6,
    PilotPump,
    BackupPump,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SystemActiveFunctionRecirculationPump {
    None,
    EmergencyRunActive,
    StandbyPumpsActive,
    PumpTestRunActive,
    AlternativeSetpointActive,
    ClockProgramActive,
    // VNC: Virtual network connection
    RemoteVncActive,
    RemoteBusActive,
    RemoteServicePortActive,
    PressureReliefActive,
    SoftPressureActive,
    LowFlowBoostActive,
    LowFlowStopActive,
    ProportionalPressureActive,
    Spare14,
    Spare15,
}
