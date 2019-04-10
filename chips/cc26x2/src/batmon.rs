// Table 20-1. CC26_AON_BATMON_REGMAP Registers
// Offset Acronym Register Name Section
// 0h	CTL				Internal Section 20.3.1.1
// 4h	MEASCFG			Internal Section 20.3.1.2
// Ch	TEMPP0			Internal Section 20.3.1.3
// 10h	TEMPP1			Internal Section 20.3.1.4
// 14h	TEMPP2			Internal Section 20.3.1.5
// 18h	BATMONP0		Internal Section 20.3.1.6
// 1Ch	BATMONP1		Internal Section 20.3.1.7
// 20h	IOSTRP0			Internal Section 20.3.1.8
// 24h	FLASHPUMPP0		Internal Section 20.3.1.9
// 28h	BAT				Last Measured Battery Voltage Section 20.3.1.10
// 2Ch	BATUPD			Battery Update Section 20.3.1.11
// 30h	TEMP			Temperature Section 20.3.1.12
// 34h	TEMPUPD			Temperature Update Section 20.3.1.13
// 48h	EVENTMASK		Event Mask Section 20.3.1.14
// 4Ch	EVENT			Event Section 20.3.1.15
// 50h	BATTUL			Battery Upper Limit Section 20.3.1.16
// 54h	BATTLL			Battery Lower Limit Section 20.3.1.17
// 58h	TEMPUL			Temperature Upper Limit Section 20.3.1.18
// 5Ch	TEMPLL			Temperature Lower Limit Section 20.3.1.19

use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;

use crate::memory_map::AON_BATMON_BASE;

pub const BATMON: StaticRef<Registers> = unsafe { StaticRef::new(AON_BATMON_BASE as *const Registers) };

#[repr(C)]
pub struct Registers {
    ctl: ReadWrite<u32, Ctl::Register>,
    meascfg: ReadWrite<u32, MeasCfg::Register>,
    _reserved0: ReadOnly<u32>,
    tempp0: ReadOnly<u32>,
    tempp1: ReadOnly<u32>,
    tempp2: ReadOnly<u32>,
    batmonp0: ReadOnly<u32>,
    batmonp1: ReadOnly<u32>,
    iostrp0: ReadOnly<u32>,
    pub flashpumpp0: ReadWrite<u32, FlashPumpP0::Register>,
    bat: ReadOnly<u32, Battery::Register>,
    batupd: ReadOnly<u32, BatteryUpdates::Register>,
}

register_bitfields![
    u32,
    Ctl [
        CALC  OFFSET(0) NUMBITS(1) [
            ENABLE = 1,
            DISABLE = 0
        ],
        MEAS OFFSET(1) NUMBITS(1) [
            ENABLE = 1,
            DISABLE = 0
        ]
    ],
    MeasCfg [
        PER  OFFSET(0) NUMBITS(2) [
            _32 = 0x03,
            _16 = 0x02,
            _8 = 0x01,
            CONTINUOUS = 0x0
        ],
        MEAS OFFSET(1) NUMBITS(1) [
            ENABLE = 1,
            DISABLE = 0
        ]
    ],
    FlashPumpP0 [
    	LOWLIM OFFSET(5) NUMBITS(1) [
            INTERNAL_REGULATOR_MODE = 0,
            EXTERNAL_REGULATOR_MODE  = 1
        ]
    ],
    Battery [
    	INT OFFSET(8) NUMBITS(3) [],
    	FRAC OFFSET(0) NUMBITS(8) [] //binary fractional encoding 
    	// eg: (0x20: 1/2 = 0.125V, 0x40: 1/4 = 0.25V ... 0xA0 = 1/2 + 1/8 = 0.625V)
    ],
    BatteryUpdates [
    	SINCE_LAST_CLEAR OFFSET(0) NUMBITS(1) []
    ]

];

pub fn enable() {
    BATMON.ctl.write(Ctl::CALC::ENABLE + Ctl::MEAS::ENABLE);
    BATMON.meascfg.write(MeasCfg::PER::CONTINUOUS + MeasCfg::MEAS::ENABLE);
}

pub fn has_new_measurement() -> bool {
	BATMON.batupd.read(BatteryUpdates::SINCE_LAST_CLEAR) == 1
}

pub fn get_mv() -> u32 {
	// read in the integer part of the voltage
	// and initialize the return value with it
	let mut ret = 1000 * BATMON.bat.read(Battery::INT);
	// read in the factional part of the voltage
	let frac = BATMON.bat.read(Battery::FRAC);
	// create a bitmask on the highest bit
	let mut bm = 0b10000000;
	// initialize a multiplier coefficient
	let mut mult = 1.0/2.0;

	// for every bit, if it's set, multiply by current mutiplier
	for _i in 0..7 {
		if (frac&bm) != 0 {
			ret += (mult*1000.0) as u32;
		}
		// shift the bitmask
		bm >>= 1;
		// keep multiplying out the multiplier
		mult *= 1.0/2.0;
	}
	ret
}

/*

to add to setup

// 1.
// Do not allow DCDC to be enabled if in external regulator mode.
// Preventing this by setting both the RECHARGE and the ACTIVE bits bit in the CCFG_MODE_CONF copy register (ccfg_ModeConfReg).
//
// 2.
// Adjusted battery monitor low limit in internal regulator mode.
// This is done by setting AON_BATMON_FLASHPUMPP0_LOWLIM=0 in internal regulator mode.
if ( HWREG( AON_PMCTL_BASE + AON_PMCTL_O_PWRCTL ) & AON_PMCTL_PWRCTL_EXT_REG_MODE ) {
    ccfg_ModeConfReg |= ( CCFG_MODE_CONF_DCDC_RECHARGE_M | CCFG_MODE_CONF_DCDC_ACTIVE_M );
} else {
    HWREGBITW( AON_BATMON_BASE + AON_BATMON_O_FLASHPUMPP0, AON_BATMON_FLASHPUMPP0_LOWLIM_BITN ) = 0;

	batmon::BATMON::FlashPumpP0.write(batmon::FlashPumpP0::LOWLIM::INTERNAL_REGULATOR_MODE)
    gpt::GPT[0].cfg.write(gpt::Cfg::BITS::_16);
}

*/