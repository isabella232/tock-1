#![feature(const_fn, used, untagged_unions)]
#![no_std]
#![crate_name = "cc26x2"]
#![crate_type = "rlib"]
extern crate cc26xx;
extern crate cortexm4;
#[allow(unused_imports)]
#[macro_use]
extern crate kernel;
#[macro_use]
extern crate bitfield;
extern crate fixedvec;

pub mod aon;
pub mod aux;
pub mod chip;
pub mod commands;
pub mod crt1;
pub mod osc;
pub mod oscfh;
pub mod prcm;
pub mod rat;
pub mod rfc;
pub mod rtc;
pub mod i2c;
// pub mod setup;
#[allow(unused, unused_mut)]
pub use crt1::init;
