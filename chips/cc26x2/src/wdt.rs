// Table 17-1. WDTIMERV2_0_NOSYNC_WRAPPER_MAP1 Registers
//
// Offset 	Acronym 	Register Name 					Section
//
// 0h		LOAD		Configuration 					Section 17.4.1.1
// 4h		VALUE		Current Count Value 			Section 17.4.1.2
// 8h		CTL		 	Control 						Section 17.4.1.3
// Ch		ICR		 	Interrupt Clear 				Section 17.4.1.4
// 10h		RIS		 	Raw Interrupt Status 			Section 17.4.1.5
// 14h		MIS		 	Masked Interrupt Status 		Section 17.4.1.6
// 418h		TEST		Test Mode 						Section 17.4.1.7
// 41Ch		INT_CAUS	Interrupt Cause Test Mode 		Section 17.4.1.8
// C00h		LOCK		Lock 							Section 17.4.1.9

use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;

use crate::memory_map::WDT_BASE;

pub const WDT: StaticRef<Registers> =
    unsafe { StaticRef::new(WDT_BASE as *const Registers) };

#[repr(C)]
pub struct Registers {
    load: ReadWrite<u32>,
	read: ReadOnly<u32>,
	ctl: ReadWrite<u32, Ctl::Register>,
}

register_bitfields![
    u32,
    Ctl [
        INTTYPE  OFFSET(2) NUMBITS(1) [
            MASKABLE = 1,
            NONMASKABLE= 0
        ],
        RESET OFFSET(1) NUMBITS(1) [
            ENABLE = 1,
            DISABLE = 0
        ],
        INT OFFSET(0) NUMBITS(1) [
            ENABLE = 1,
            DISABLE = 0
        ]
    ]
];

pub fn enable() {
	WDT.load.set(0xFF);
    WDT.ctl.write(Ctl::RESET::ENABLE + Ctl::INT::ENABLE);
}

pub fn disable() {
    WDT.ctl.write(Ctl::INT::DISABLE);
}

