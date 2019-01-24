#[allow(unused)]
use enum_primitive::cast::FromPrimitive;

pub type PropReturnCode = Result<(), RfcPropStatus>;
pub type CmdrReturnCode = Result<(), CmdrStatus>;

enum_from_primitive!{
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CmdrStatus {
    Pending = 0x00,           // Command has not been parsed
    Done = 0x01, // Immediate command: The command finished successfully. Radio operation command: The command was successfully submitted for execution.
    Failure = 0x80, // This is not an actual CMDR return code, but is a failure for the command dispatching fuction to check if RFC power is enabled.
    IllegalPointer = 0x81, // The pointer signaled in CMDR is not valid
    UnknownCommand = 0x82, // The command ID number in the command structure is unknown
    UnknownDirCommand = 0x83, // The command number for a direct command is unknown, or the command is not a direct command
    ContextError = 0x85, // The immediate or direct command was issued in a context where it was not supported
    SchedulingError = 0x86, // A radio operation command was attempted to be scheduled while another operation was already running in the RF core. The new command is rejected, while the command already running is not impacted.
    ParError = 0x87, // There were errors in the command parameters that are parsed on submission. For radio operation commands, errors in parameters parsed after start of the command are signaled by the command ending, and an error is indicated in the status field of that command structure.
    QueueError = 0x88, // An operation on a data entry queue was attempted, but the operation was not supported by the queue in its current state.
    QueueBusy = 0x89,  // An operation on a data entry was attempted while that entry was busy
}
}

enum_from_primitive!{
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RfcOpStatus {
    Idle = 0x0000,           // Operation has not started
    Pending = 0x0001,        // Waiting for start trigger
    Active = 0x0002,         // Running operation
    Skipped = 0x0003,        // Operation skipped due to condition in another command
    DoneOk = 0x0400,         // Operation ended normally
    DoneCountdown = 0x0401,  // Counter reached zero
    DoneRxErr = 0x0402,      // Operation ended with CRC error
    DoneTimeout = 0x0403,    // Operation ended with time-out
    DoneStopped = 0x0404,    // Operation stopped after CMD_STOP command
    DoneAbort = 0x0405,      // Operation aborted by CMD_ABORT command
    ErrorPastStart = 0x0800, // The start trigger occurred in the past
    ErrorStartTrig = 0x0801, // Illegal start trigger parameter
    ErrorCondition = 0x0802, // Illegal condition for next operation
    ErrorPar = 0x0803,       // Error in a command specific parameter
    ErrorPointer = 0x0804,   // Invalid pointer to next operation
    ErrorCmdId = 0x0805, // Next operation has a command ID that is undefined or not a radio operation command
    ErrorNoSetup = 0x0807, // Operation using RX, TX or synthesizer attempted without CMD_RADIO_SETUP
    ErrorNoFs = 0x0808, // Operation using RX or TX attempted without the synthesizer being programmed or powered on
    ErrorSynthProg = 0x0809, // Synthesizer programming failed
    ErrorTxUnf = 0x080A, // Modem TX underflow observed
    ErrorRxOvf = 0x080B, // Modem RX overflow observed
    ErrorNoRx = 0x080C, // Data requested from last RX when no such data exists
}
}

enum_from_primitive!{
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RfcPropStatus {
    Idle = 0x0000,                // Operation not started
    Pending = 0x0001,             // Waiting for trigger
    Active = 0x0002,              // Running operation
    PropDoneOk = 0x3400,          // Operation ended normally
    PropDoneRxTimeout = 0x3401,   // Operation stopped after end trigger while waiting for sync
    PropDoneBreak = 0x3402,       // RX stopped due to timeout in the middle of a packet
    PropDoneEnded = 0x3403,       // Operation stopped after end trigger during reception
    PropDoneStopped = 0x3404,     // Operation stopped after stop command
    PropDoneAbort = 0x3405,       // Operation aborted by abort command
    PropDoneRxErr = 0x3406,       // Operation ended after receiving packet with CRC error
    PropDoneIdle = 0x3407,        // Carrier sense operation ended because of idle channel
    PropDoneBusy = 0x3408,        // Carrier sense operation ended because of busy channel
    PropDoneIdleTimeout = 0x3409, // Carrier sense operation ended because of time-out with csConf.timeoutRes = 1
    PropDoneBusyTimeout = 0x340A, // Carrier sense operation ended because of time out with csConf.timeoutRes = 0
    PropErrorPar = 0x3800,        // Illegal parameter
    PropErrorRxBuf = 0x3801, // No RX buffer large enough for the received data available at the start of a packet
    PropErrorRxFull = 0x3802, // Out of RX buffer during reception in a partial read buffer
    PropErrorNoSetup = 0x3803, // Radio was not set up in proprietary mode
    PropErrorNoFs = 0x3804,  // Synthesizer was not programmed when running RX or TX
    PropErrorRxOvf = 0x3805, // TX overflow observed during operation
    PropErrorTxUnf = 0x3806, // TX underflow observed during operation
}
}
