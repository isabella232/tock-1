#![no_std]
#![no_main]
#![feature(lang_items, asm)]

extern crate capsules;
extern crate cc26x2;
extern crate cortexm4;
extern crate enum_primitive;

#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, static_init};

use capsules::virtual_uart::{MuxUart, UartDevice};
use cc26x2::aon;
use cc26x2::prcm;
use cc26x2::pwm;
use kernel::capabilities;
use kernel::hil;

#[macro_use]
pub mod io;

#[allow(dead_code)]
mod ccfg_test;
#[allow(dead_code)]
mod i2c_tests;
#[allow(dead_code)]
mod uart_echo;

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

pub struct Platform {
    led: &'static capsules::led::LED<'static, cc26x2::gpio::GPIOPin>,
    console: &'static capsules::console::Console<'static>,
    ipc: kernel::ipc::IPC,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

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

    // LEDs
    let led_pins = static_init!(
        [(
            &'static cc26x2::gpio::GPIOPin,
            capsules::led::ActivationMode
        ); 2],
        [
            (
                &cc26x2::gpio::PORT[pinmap.red_led],
                capsules::led::ActivationMode::ActiveHigh
            ), // Red
            (
                &cc26x2::gpio::PORT[pinmap.green_led],
                capsules::led::ActivationMode::ActiveHigh
            ), // Green
        ]
    );
    let led = static_init!(
        capsules::led::LED<'static, cc26x2::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins)
    );

    // UART
    cc26x2::uart::UART0.initialize();

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = static_init!(
        MuxUart<'static>,
        MuxUart::new(
            &cc26x2::uart::UART0,
            &mut capsules::virtual_uart::RX_BUF,
            115200
        )
    );
    uart_mux.initialize();
    hil::uart::Receive::set_receive_client(&cc26x2::uart::UART0, uart_mux);
    hil::uart::Transmit::set_transmit_client(&cc26x2::uart::UART0, uart_mux);

    // Create a UartDevice for the console.
    let console_uart = static_init!(UartDevice, UartDevice::new(uart_mux, true));
    console_uart.setup();

    let console = static_init!(
        capsules::console::Console<'static>,
        capsules::console::Console::new(
            console_uart,
            &mut capsules::console::WRITE_BUF,
            &mut capsules::console::READ_BUF,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    kernel::hil::uart::Transmit::set_transmit_client(console_uart, console);
    kernel::hil::uart::Receive::set_receive_client(console_uart, console);

    // Create virtual device for kernel debug.
    let debugger_uart = static_init!(UartDevice, UartDevice::new(uart_mux, false));
    debugger_uart.setup();
    let debugger = static_init!(
        kernel::debug::DebugWriter,
        kernel::debug::DebugWriter::new(
            debugger_uart,
            &mut kernel::debug::OUTPUT_BUF,
            &mut kernel::debug::INTERNAL_BUF,
        )
    );
    hil::uart::Transmit::set_transmit_client(debugger_uart, debugger);

    let debug_wrapper = static_init!(
        kernel::debug::DebugWriterWrapper,
        kernel::debug::DebugWriterWrapper::new(debugger)
    );
    kernel::debug::set_debug_writer_wrapper(debug_wrapper);

    let ipc = kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability);

    let launchxl = Platform {
        console,
        led,
        ipc,
    };

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

    debug!("alive");

    board_kernel.kernel_loop(&launchxl, chip, Some(&launchxl.ipc), &main_loop_capability);
}
