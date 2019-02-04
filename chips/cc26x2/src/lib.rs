#![feature(const_fn, untagged_unions, asm, naked_functions)]
#![no_std]
#![crate_name = "cc26x2"]
#![crate_type = "rlib"]
extern crate cortexm;
extern crate cortexm4;
#[allow(unused_imports)]
#[macro_use]
extern crate kernel;
#[macro_use]
extern crate enum_primitive;

pub mod aon;
pub mod ccfg;
pub mod chip;
pub mod crt1;
pub mod event_priority;
pub mod event;
pub mod events;
pub mod gpio;
pub mod gpt;
pub mod i2c;
pub mod ioc;
pub mod memory_map;
pub mod peripheral_interrupts;
pub mod prcm;
pub mod pwm;
pub mod rom;
pub mod rtc;
pub mod trng;
pub mod uart;

pub use crate::crt1::init;
