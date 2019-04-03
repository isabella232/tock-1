// pub mod ble;
pub mod multimode;
pub mod patches;
pub mod queue;
pub mod rfc;
use crate::peripheral_interrupts;
use cortexm4::nvic;

pub mod commands;

const RF_ACK_NVIC: nvic::Nvic =
    unsafe { nvic::Nvic::new(peripheral_interrupts::NVIC_IRQ::RF_CMD_ACK as u32) };
const RF_CPE0_NVIC: nvic::Nvic =
    unsafe { nvic::Nvic::new(peripheral_interrupts::NVIC_IRQ::RF_CORE_CPE0 as u32) };
const RF_CPE1_NVIC: nvic::Nvic =
    unsafe { nvic::Nvic::new(peripheral_interrupts::NVIC_IRQ::RF_CORE_CPE1 as u32) };

pub static mut RFC: rfc::RFCore = rfc::RFCore::new(&RF_ACK_NVIC, &RF_CPE0_NVIC, &RF_CPE1_NVIC);
pub static mut MULTIMODE_RADIO: multimode::Radio = unsafe { multimode::Radio::new(&RFC, multimode::CC1352R_POWER::_4) };
// pub static mut BLE: ble::Ble = unsafe { ble::Ble::new(&RFC) };
