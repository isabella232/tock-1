pub enum RfcOpStatus {
    Idle,           // Operation has not started
    Pending,        // Waiting for start trigger
    Active,         // Running operation
    Skipped,        // Operation skipped due to condition in another command
    DoneOk,         // Operation ended normally
    DoneCountdown,  // Counter reached zero
    DoneRxErr,      // Operation ended with CRC error
    DoneTimeout,    // Operation ended with time-out
    DoneStopped,    // Operation stopped after CMD_STOP command
    DoneAbort,      // Operation aborted by CMD_ABORT command
    ErrorPastStart, // The start trigger occurred in the past
    ErrorStartTrig, // Illegal start trigger parameter
    ErrorCondition, // Illegal condition for next operation
    ErrorPar,       // Error in a command specific parameter
    ErrorPointer,   // Invalid pointer to next operation
    ErrorCmdId, // Next operation has a command ID that is undefined or not a radio operation command
    ErrorNoSetup, // Operation using RX, TX or synthesizer attempted without CMD_RADIO_SETUP
    ErrorNoFs, // Operation using RX or TX attempted without the synthesizer being programmed or powered on
    ErrorSynthProg, // Synthesizer programming failed
    ErrorTxUnf, // Modem TX underflow observed
    ErrorRxOvf, // Modem RX overflow observed
    ErrorNoRx, // Data requested from last RX when no such data exists
}

pub enum RfcPropStatus {
    Idle,                // Operation not started
    Pending,             // Waiting for trigger
    Active,              // Running operation
    PropDoneOk,          // Operation ended normally
    PropDoneRxTimeout,   // Operation stopped after end trigger while waiting for sync
    PropDoneBreak,       // RX stopped due to timeout in the middle of a packet
    PropDoneEnded,       // Operation stopped after end trigger during reception
    PropDoneStopped,     // Operation stopped after stop command
    PropDoneAbort,       // Operation aborted by abort command
    PropDoneRxErr,       // Operation ended after receiving packet with CRC error
    PropDoneIdle,        // Carrier sense operation ended because of idle channel
    PropDoneBusy,        // Carrier sense operation ended because of busy channel
    PropDoneIdleTimeout, // Carrier sense operation ended because of time-out with csConf.timeoutRes = 1
    PropDoneBusyTimeout, // Carrier sense operation ended because of time out with csConf.timeoutRes = 0
    PropErrorPar,        // Illegal parameter
    PropErrorRxBuf, // No RX buffer large enough for the received data available at the start of a packet
    PropErrorRxFull, // Out of RX buffer during reception in a partial read buffer
    PropErrorNoSetup, // Radio was not set up in proprietary mode
    PropErrorNoFs,  // Synthesizer was not programmed when running RX or TX
    PropErrorRxOvf, // TX overflow observed during operation
    PropErrorTxUnf, // TX underflow observed during operation
}
