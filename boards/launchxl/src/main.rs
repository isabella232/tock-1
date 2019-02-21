#![no_std]
#![no_main]
#![feature(lang_items, asm)]
#![feature(const_slice_len)]
#![feature(type_ascription)] 

extern crate capsules;
extern crate cc26x2;
extern crate cortexm4;
extern crate enum_primitive;

#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, static_init};
use kernel::common::cells::TakeCell;

use cc26x2::aon;
use cc26x2::prcm;
use cc26x2::pwm;

use kernel::capabilities;
use kernel::hil;

use core::mem;


use capsules::uart;

#[macro_use]
pub mod io;

#[allow(dead_code)]
mod ccfg_test;
#[allow(dead_code)]
mod i2c_tests;
#[allow(dead_code)]
mod uart_test;

// High frequency oscillator speed
pub const HFREQ: u32 = 48 * 1_000_000;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 3;
static mut PROCESSES: [Option<&'static kernel::procs::ProcessType>; NUM_PROCS] = [None, None, None];

#[link_section = ".app_memory"]
// Give half of RAM to be dedicated APP memory
static mut APP_MEMORY: [u8; 0x10000] = [0; 0x10000];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

use cc26x2::peripheral_interrupts::NVIC_IRQ;
use enum_primitive::cast::FromPrimitive;



mod cc1312r;
mod cc1352p;

pub struct Pinmap {
    uart0_rx: usize,
    uart0_tx: usize,
    i2c0_scl: usize,
    i2c0_sda: usize,
    red_led: usize,
    green_led: usize,
    button1: usize,
    button2: usize,
    gpio0: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
    pwm0: usize,
    pwm1: usize,
}

unsafe fn configure_pins(pin: &Pinmap) {
    cc26x2::gpio::PORT[pin.uart0_rx].enable_uart0_rx();
    cc26x2::gpio::PORT[pin.uart0_tx].enable_uart0_tx();

    cc26x2::gpio::PORT[pin.i2c0_scl].enable_i2c_scl();
    cc26x2::gpio::PORT[pin.i2c0_sda].enable_i2c_sda();

    cc26x2::gpio::PORT[pin.red_led].enable_gpio();
    cc26x2::gpio::PORT[pin.green_led].enable_gpio();

    cc26x2::gpio::PORT[pin.button1].enable_gpio();
    cc26x2::gpio::PORT[pin.button2].enable_gpio();

    cc26x2::gpio::PORT[pin.gpio0].enable_gpio();

    cc26x2::gpio::PORT[pin.a7].enable_analog_input();
    cc26x2::gpio::PORT[pin.a6].enable_analog_input();
    cc26x2::gpio::PORT[pin.a5].enable_analog_input();
    cc26x2::gpio::PORT[pin.a4].enable_analog_input();
    cc26x2::gpio::PORT[pin.a3].enable_analog_input();
    cc26x2::gpio::PORT[pin.a2].enable_analog_input();
    cc26x2::gpio::PORT[pin.a1].enable_analog_input();
    cc26x2::gpio::PORT[pin.a0].enable_analog_input();

    cc26x2::gpio::PORT[pin.pwm0].enable_pwm(pwm::Timer::GPT0A);
    cc26x2::gpio::PORT[pin.pwm1].enable_pwm(pwm::Timer::GPT0B);
}

use kernel::platform::Platform;

#[no_mangle]
pub unsafe fn reset_handler() {
    cc26x2::init();

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    // Setup AON event defaults
    aon::AON.setup();

    // Power on peripherals (eg. GPIO)
    prcm::Power::enable_domain(prcm::PowerDomain::Peripherals);

    // Wait for it to turn on until we continue
    while !prcm::Power::is_enabled(prcm::PowerDomain::Peripherals) {}

    // Power on Serial domain
    prcm::Power::enable_domain(prcm::PowerDomain::Serial);

    while !prcm::Power::is_enabled(prcm::PowerDomain::Serial) {}

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    // Enable the GPIO clocks
    prcm::Clock::enable_gpio();

    let pinmap: &Pinmap;
    let chip_id = (cc26x2::rom::HAPI.get_chip_id)();

    if chip_id == cc1352p::CHIP_ID {
        pinmap = &cc1352p::PINMAP;
    } else {
        pinmap = &cc1312r::PINMAP;
    }

    configure_pins(pinmap);

    // UART
    let uart0_hil = cc26x2::uart::UART::new(cc26x2::uart::PeripheralNum::_0);
    let uart1_hil = cc26x2::uart::UART::new(cc26x2::uart::PeripheralNum::_1);

    let board_uarts = [
        &uart::Uart::new(
            &uart0_hil,
            board_kernel.create_grant(&memory_allocation_capability)),
        &uart::Uart::new(
            &uart1_hil,
            board_kernel.create_grant(&memory_allocation_capability)),
    ];

    let uart_driver = uart::UartDriver::new(&board_uarts);

    // set up test client
    let mut space = hil::uart::TxRequest::new();
    let test_client = uart_test::TestClient::new(&mut space);
    let mut launchxl = LaunchXlPlatform {
        uart_driver: &uart_driver,
        test_client: &test_client,
    };

    launchxl.handle_irq(NVIC_IRQ::UART0 as usize);

    let chip = static_init!(cc26x2::chip::Cc26X2, cc26x2::chip::Cc26X2::new(HFREQ));

    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }

    kernel::procs::load_processes(
        board_kernel,
        chip,
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    );

    // debug!("alive");

    board_kernel.kernel_loop(&mut launchxl, chip, None, &main_loop_capability);
}

pub struct LaunchXlPlatform<'a> {
    uart_driver: &'a capsules::uart::UartDriver<'a>,
    test_client: &'a uart_test::TestClient<'a>,
}

impl<'a> kernel::Platform for LaunchXlPlatform<'a> {

    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::uart::DRIVER_NUM => f(Some(self.uart_driver)),
            _ => f(None),
        }
    }

    fn handle_irq(&mut self, irq_num: usize)
    {
        let irq = NVIC_IRQ::from_u32(irq_num as u32)
            .expect("Pending IRQ flag not enumerated in NVIQ_IRQ");
        match irq {
            NVIC_IRQ::GPIO => (),//gpio::PORT.handle_interrupt(),
            NVIC_IRQ::AON_RTC => (),//rtc::RTC.handle_interrupt(),
            NVIC_IRQ::UART0 => {
                let clients = [self.test_client as &kernel::hil::uart::Client];
                capsules::uart::handle_irq(0, self.uart_driver, &clients);
            },
            NVIC_IRQ::I2C0 => (),//i2c::I2C0.handle_interrupt(),
            // We need to ignore JTAG events since some debuggers emit these
            NVIC_IRQ::AON_PROG => (),
            _ => ()//panic!("Unhandled interrupt {:?}", irq_num),
        }
    }
}