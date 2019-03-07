#![no_std]
#![no_main]
#![feature(lang_items, asm, naked_functions)]

extern crate capsules;
extern crate cc26x2;
extern crate cortexm4;
extern crate cortexm; 
extern crate enum_primitive;

use capsules::uart;
use cc26x2::aon;
use cc26x2::peripheral_interrupts::NVIC_IRQ;
use cc26x2::prcm;
use cc26x2::pwm;
use cortexm::events;

use enum_primitive::cast::FromPrimitive;
#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, static_init};

use kernel::capabilities;
use kernel::common::cells::TakeCell;
use kernel::hil;
use kernel::hil::entropy::Entropy32;
use kernel::hil::gpio::InterruptMode;
use kernel::hil::gpio::Pin;
use kernel::hil::gpio::PinCtl;
use kernel::hil::i2c::I2CMaster;
use kernel::hil::rng::Rng;
use kernel::platform::Platform;

#[macro_use]
pub mod io;

#[allow(dead_code)]
mod ccfg_test;
#[allow(dead_code)]
mod i2c_tests;
#[allow(dead_code)]
mod uart_test;
mod event_priority;


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

pub struct LaunchXlPlatform<'a>{
    gpio: &'static capsules::gpio::GPIO<'static, cc26x2::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, cc26x2::gpio::GPIOPin>,
    uart: &'a capsules::uart::UartDriver<'a>,
    debug_client: &'a debug::DebugClient<'a>,
    button: &'static capsules::button::Button<'static, cc26x2::gpio::GPIOPin>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26x2::rtc::Rtc>,
    >,
    rng: &'static capsules::rng::RngDriver<'static>,
    i2c_master: &'static capsules::i2c_master::I2CMasterDriver<cc26x2::i2c::I2CMaster<'static>>,
}

impl<'a> kernel::Platform for LaunchXlPlatform<'a> {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::uart::DRIVER_NUM => f(Some(self.uart)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules::i2c_master::DRIVER_NUM => f(Some(self.i2c_master)),
            _ => f(None),
        }
    }

    fn has_pending_events(&mut self) -> bool {
        events::has_event()
    }

    fn service_pending_events(&mut self) {
        let pending_event: Option<event_priority::EVENT_PRIORITY>  = events::next_pending();
        while let Some(event) = pending_event {
            events::clear_event_flag(event);
            match event {
                event_priority::EVENT_PRIORITY::GPIO => {}, //unsafe {cc26x2::gpio::PORT.handle_events()},
                event_priority::EVENT_PRIORITY::AON_RTC => {}, //unsafe {cc26x2::rtc::RTC.handle_events()},
                event_priority::EVENT_PRIORITY::I2C0 => {}, //unsafe {cc26x2::i2c::I2C0.handle_events()},
                event_priority::EVENT_PRIORITY::UART0 => {

                    // pass data from static debug writer to the stack allocated debug uart client
                    unsafe {
                        self.debug_client.with_buffer( |buf| debug::get_debug_writer().publish_str(buf));
                    }
                    let clients = [
                        self.debug_client as &kernel::hil::uart::Client,
                    ];
                    capsules::uart::handle_irq(0, self.uart, Some(&clients));
                },
                event_priority::EVENT_PRIORITY::UART1 => {
                    //capsules::uart::handle_irq(1, self.uart, None);
                },
                //event_priority::EVENT_PRIORITY::RF_CMD_ACK => cc26x2::radio::RFC.handle_ack_event(),
                //event_priority::EVENT_PRIORITY::RF_CORE_CPE0 => cc26x2::radio::RFC.handle_cpe0_event(),
                //event_priority::EVENT_PRIORITY::RF_CORE_CPE1 => cc26x2::radio::RFC.handle_cpe1_event(),
                //event_priority::EVENT_PRIORITY::RF_CORE_HW => panic!("Unhandled RFC interupt event!"),
                //event_priority::EVENT_PRIORITY::AUX_ADC => cc26x2::adc::ADC.handle_events(),
                //event_priority::EVENT_PRIORITY::OSC => cc26x2::prcm::handle_osc_interrupt(),
                event_priority::EVENT_PRIORITY::AON_PROG => (),
                _ => panic!("unhandled event {:?} ", event),
            }
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

    // BUTTONS
    let button_pins = static_init!(
        [(&'static cc26x2::gpio::GPIOPin, capsules::button::GpioMode); 2],
        [
            (
                &cc26x2::gpio::PORT[pinmap.button1],
                capsules::button::GpioMode::LowWhenPressed
            ), // Button 1
            (
                &cc26x2::gpio::PORT[pinmap.button2],
                capsules::button::GpioMode::LowWhenPressed
            ), // Button 2
        ]
    );
    let button = static_init!(
        capsules::button::Button<'static, cc26x2::gpio::GPIOPin>,
        capsules::button::Button::new(
            button_pins,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );

    let mut count = 0;
    for &(btn, _) in button_pins.iter() {
        btn.set_input_mode(hil::gpio::InputMode::PullUp);
        btn.enable_interrupt(count, InterruptMode::FallingEdge);
        btn.set_client(button);
        count += 1;
    }

    // UARTU
    // setup static debug writer
    let debug_writer = static_init!(
        kernel::debug::DebugWriter,
        kernel::debug::DebugWriter::new(&mut kernel::debug::BUF)
    );
    kernel::debug::set_debug_writer(debug_writer);
    // setup uart client for debug on stack
    let mut debug_client_space = debug::DebugClient::space();
    let debug_client = debug::DebugClient::new_with_default_space(&mut debug_client_space);

    // UART
    let uart0_hil = cc26x2::uart::UART::new(cc26x2::uart::PeripheralNum::_0);
    let mut uart0_driver_app_space = uart::AppRequestsInProgress::space();

    // for each client for the driver, provide an empty TakeCell
    let uart0_clients: [TakeCell<hil::uart::RxRequest>; 3] = [TakeCell::empty(), TakeCell::empty(), TakeCell::empty()];

    let uart1_hil = cc26x2::uart::UART::new(cc26x2::uart::PeripheralNum::_1);
    let mut uart1_driver_app_space = uart::AppRequestsInProgress::space();


    let board_uarts = [
        &uart::Uart::new(
            &uart0_hil,
            Some(&uart0_clients),
            uart::AppRequestsInProgress::new_with_default_space(&mut uart0_driver_app_space),
            board_kernel.create_grant(&memory_allocation_capability),
        ),
        &uart::Uart::new(
            &uart1_hil,
            None,
            uart::AppRequestsInProgress::new_with_default_space(&mut uart1_driver_app_space),
            board_kernel.create_grant(&memory_allocation_capability),
        ),
    ];

    let uart_driver = uart::UartDriver::new(&board_uarts);

    cc26x2::i2c::I2C0.initialize();

    let i2c_master = static_init!(
        capsules::i2c_master::I2CMasterDriver<cc26x2::i2c::I2CMaster<'static>>,
        capsules::i2c_master::I2CMasterDriver::new(
            &cc26x2::i2c::I2C0,
            &mut capsules::i2c_master::BUF,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );

    cc26x2::i2c::I2C0.set_client(i2c_master);
    cc26x2::i2c::I2C0.enable();

    // Setup for remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static cc26x2::gpio::GPIOPin; 1],
        [
            // This is the order they appear on the launchxl headers.
            // Pins 5, 8, 11, 29, 30
            &cc26x2::gpio::PORT[pinmap.gpio0],
        ]
    );
    let gpio = static_init!(
        capsules::gpio::GPIO<'static, cc26x2::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(
            gpio_pins,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    let rtc = &cc26x2::rtc::RTC;
    rtc.start();

    let mux_alarm = static_init!(
        capsules::virtual_alarm::MuxAlarm<'static, cc26x2::rtc::Rtc>,
        capsules::virtual_alarm::MuxAlarm::new(&cc26x2::rtc::RTC)
    );
    rtc.set_client(mux_alarm);

    let virtual_alarm1 = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26x2::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<
            'static,
            capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26x2::rtc::Rtc>,
        >,
        capsules::alarm::AlarmDriver::new(
            virtual_alarm1,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    virtual_alarm1.set_client(alarm);

    let entropy_to_random = static_init!(
        capsules::rng::Entropy32ToRandom<'static>,
        capsules::rng::Entropy32ToRandom::new(&cc26x2::trng::TRNG)
    );
    let rng = static_init!(
        capsules::rng::RngDriver<'static>,
        capsules::rng::RngDriver::new(
            entropy_to_random,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    cc26x2::trng::TRNG.set_client(entropy_to_random);
    entropy_to_random.set_client(rng);

    let pwm_channels = [
        pwm::Signal::new(pwm::Timer::GPT0A),
        pwm::Signal::new(pwm::Timer::GPT0B),
        pwm::Signal::new(pwm::Timer::GPT1A),
        pwm::Signal::new(pwm::Timer::GPT1B),
        pwm::Signal::new(pwm::Timer::GPT2A),
        pwm::Signal::new(pwm::Timer::GPT2B),
        pwm::Signal::new(pwm::Timer::GPT3A),
        pwm::Signal::new(pwm::Timer::GPT3B),
    ];

    // all PWM channels are enabled
    for pwm_channel in pwm_channels.iter() {
        pwm_channel.enable();
    }

    let ipc = kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability);

    let mut launchxl = LaunchXlPlatform {
        uart: &uart_driver,
        debug_client: &debug_client,
        gpio,
        led,
        button,
        alarm,
        rng,
        i2c_master,
    };

    events::set_event_flag(event_priority::EVENT_PRIORITY::UART0);

    // prime the pump with this interaction
    //launchxl.handle_irq(NVIC_IRQ::UART0 as usize);

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

    debug!("!");

    board_kernel.kernel_loop(&mut launchxl, chip, Some(&ipc), &main_loop_capability);
}


use cortexm4::{
    disable_specific_nvic, generic_isr, hard_fault_handler, nvic, set_privileged_thread,
    stash_process_state, svc_handler, systick_handler
};

macro_rules! generic_isr {
    ($label:tt, $priority:expr) => {
        #[cfg(target_os = "none")]
        #[naked]
        unsafe extern "C" fn $label() {
            stash_process_state();
            events::set_event_flag_from_isr($priority);
            disable_specific_nvic();
            set_privileged_thread();
        }
    };
}

macro_rules! custom_isr {
    ($label:tt, $priority:expr, $isr:ident) => {
        #[cfg(target_os = "none")]
        #[naked]
        unsafe extern "C" fn $label() {
            stash_process_state();
            events::set_event_flag_from_isr($priority);
            $isr();
            set_privileged_thread();
        }
    };
}

unsafe extern "C" fn unhandled_interrupt() {
    'loop0: loop {}
}

generic_isr!(uart0_nvic, event_priority::EVENT_PRIORITY::UART0);

#[link_section = ".vectors"]
// used Ensures that the symbol is kept until the final binary
#[used]
pub static BASE_VECTORS: [unsafe extern "C" fn(); 54] = [
    cc26x2::crt1::_estack,
    cc26x2::crt1::reset_handler,
    unhandled_interrupt, // NMI
    hard_fault_handler,  // Hard Fault
    unhandled_interrupt, // MPU fault
    unhandled_interrupt, // Bus fault
    unhandled_interrupt, // Usage fault
    unhandled_interrupt, // Reserved
    unhandled_interrupt, // Reserved
    unhandled_interrupt, // Reserved
    unhandled_interrupt, // Reserved
    svc_handler,         // SVC
    unhandled_interrupt, // Debug monitor,
    unhandled_interrupt, // Reserved
    unhandled_interrupt, // PendSV
    systick_handler,     // Systick
    generic_isr,         // GPIO Int handler
    generic_isr,         // I2C
    generic_isr,         // RF Core Command & Packet Engine 1
    generic_isr,         // AON SpiSplave Rx, Tx and CS
    generic_isr,         // AON RTC
    uart0_nvic,         // UART0 Rx and Tx
    generic_isr,         // AUX software event 0
    generic_isr,         // SSI0 Rx and Tx
    generic_isr,         // SSI1 Rx and Tx
    generic_isr,         // RF Core Command & Packet Engine 0
    generic_isr,         // RF Core Hardware
    generic_isr,         // RF Core Command Acknowledge
    generic_isr,         // I2S
    generic_isr,         // AUX software event 1
    generic_isr,         // Watchdog timer
    generic_isr,         // Timer 0 subtimer A
    generic_isr,         // Timer 0 subtimer B
    generic_isr,         // Timer 1 subtimer A
    generic_isr,         // Timer 1 subtimer B
    generic_isr,         // Timer 2 subtimer A
    generic_isr,         // Timer 2 subtimer B
    generic_isr,         // Timer 3 subtimer A
    generic_isr,         // Timer 3 subtimer B
    generic_isr,         // Crypto Core Result available
    generic_isr,         // uDMA Software
    generic_isr,         // uDMA Error
    generic_isr,         // Flash controller
    generic_isr,         // Software Event 0
    generic_isr,         // AUX combined event
    generic_isr,         // AON programmable 0
    generic_isr,         // Dynamic Programmable interrupt
    // source (Default: PRCM)
    generic_isr, // AUX Comparator A
    generic_isr, // AUX ADC new sample or ADC DMA
    // done, ADC underflow, ADC overflow
    generic_isr, // TRNG event
    generic_isr,
    generic_isr,
    generic_isr,
    generic_isr,
];