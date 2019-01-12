//! Implementation of the ARM Floating Point Unit

use core::cmp;
use kernel;
use kernel::common::math;
use kernel::common::registers::{FieldValue, ReadOnly, ReadWrite};
use kernel::common::StaticRef;


/// FPU Registers for Cortex-M4F family
///
/// Described in section 2.5 of 
/// https://static.docs.arm.com/ddi0403/e/DDI0403E_d_armv7m_arm.pdf?_ga=2.22762491.1113668228.1547240419-646502690.1547240419
#[repr(C)]
pub struct FpuRegisters {
    /// FP context cotrol register. Holds data for the FPU. 
    /// Accessable only by privlidged software. Is reserved if FP extension is not implemented. 
    pub fpccr: ReadWrite<u32, ContextControl::Register>,

    /// FP context address register. Holds default values for the FP status control data that the
    /// processor assign to the FPSCR when a new floating point context is created
    pub fpcar: ReadWrite<u32, ContextAddress::Register>,

    /// Holds the default values for the floating-point status control data that the processor assigns to the FPSCR when it 
    /// creates a new floating-point context.
    pub fpdscr: ReadWrite<u32, StatusControl::Register>,

    /// Describes features provided by the floating point extension. Must be interpreted together.
    pub mvfr0: ReadOnly<u32>,
    pub mvfr1: ReadOnly<u32>,
    pub mvfr2: ReadOnly<u32>,
}

register_bitfields![
    u32,
    ContextControl [
        // Enabling bit sets CONTROL.FPCA on execution of a floating-point instruction
        // Hardware automatically preserves FP context on exception entry, restores on exception
        // return.
        ASPEN OFFSET(31) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        // Enables lazy context save of FP state
        LSPEN OFFSET(30) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        // Reserved 9-29
        // Indicates whether the software executing when the processor allocated the FP stack frame
        // was able to set the DebugMonitor exception to pending. 
        // "0" Unable to set DM to pending, 
        // "1" Able to set DM to pending.
        MONRDY OFFSET(8) NUMBITS(1) [],
        // Reserved 7
        // ... set BusFault exception to pending
        // "0" Unable to set BF to pending.
        // "1" Able to set BF to pending.
        BFRDY OFFSET(6) NUMBITS(1) [],
        // ... set MemManage exception to pending
        // "0" Unable to set MM to pending
        // "1" Able to set MM to pending
        MMRDY OFFSET(5) NUMBITS(1) [],
        // ... set HardFault exception to pending
        // "0" Unable to set HF to pending
        // "1" Able to set HF to pending
        HFRDY OFFSET(4) NUMBITS(1) [],
        // Indicates processor mode when it allocated the FP stack frame.
        // "0" Handler mode
        // "1" Thread mode
        THREAD OFFSET(3) NUMBITS(1) [],
        // Reserved 2
        // Indicates privlidge level of the software executing when processor allocated the FP
        // stack frame
        // "0" Privlidged
        // "1" Unprivlidged
        USER OFFSET(1) NUMBITS(1) [],
        // Indicates whether Lazy preservation of the FP state is active
        // "0" Lazy not active
        // "1" Lazy active
        LSPACT OFFSET(0) NUMBITS(1) [],
    ],
    ContextAddress [
        // Location of unpopulated FP register space allocated on exception stack
        ADDRESS OFFSET(3) NUMBITS(28) [],
        // Reserved 0-2
    ],
    StatusControl [
        // Reserved 27-31
        // Default value for FPSCR.AHP
        AHP OFFSET(26) NUMBITS(1) [],
        // ... DN
        DN OFFSET(25) NUMBITS(1) [],
        // ... FZ
        FZ OFFSET(24) NUMBITS(1) [],
        // ... RMODE
        RMODE OFFSET(22) NUMBITS(2) [],
        // Reserved 0-21
    ]
];

const FPU_BASE_ADDRESS: StaticRef<FpuRegisters> = unsafe { StaticRef::new(0xE000EF34 as *const FpuRegisters) };

/// Private constructor for FPU registers 
pub struct FPU(StaticRef<FpuRegisters>);

impl FPU {
    pub const unsafe fn new() -> FPU {
        FPU(FPU_BASE_ADDRESS)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Fpca {
    Active,
    Inactive,
}

impl Fpca {
    pub fn is_active(&self) -> bool {
        *self == Fpca::Active
    }

    pub fn is_inactive(&self) -> bool {
        *self == Fpca::Inactive
    }
}


