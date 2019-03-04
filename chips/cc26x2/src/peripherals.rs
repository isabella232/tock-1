use crate::peripheral_manager::{Peripheral, PeripheralManager};
use crate::radio;
use crate::uart;

pub static mut M: PeripheralManager = PeripheralManager::new();

static mut UART_PERIPHERAL: Peripheral<'static> = unsafe { Peripheral::new(&uart::UART0) };

static mut RADIO_PERIPHERAL: Peripheral<'static> =
    unsafe { Peripheral::new(&radio::MULTIMODE_RADIO) };

pub unsafe fn init() {
    let peripherals = [&UART_PERIPHERAL, &RADIO_PERIPHERAL];

    for peripheral in peripherals.iter() {
        M.register_peripheral(peripheral);
    }
}
