use crate::adc;
use crate::event_priority::EVENT_PRIORITY;
use crate::events;
use crate::gpio;
use crate::i2c;
use crate::prcm;
use crate::radio;
use crate::rtc;
use crate::uart;
use crate::wdt;

use cortexm4;
use kernel;

#[repr(C)]
#[derive(Clone, Copy)]
pub enum SleepMode {
    DeepSleep = 0,
    Sleep = 1,
    Active = 2,
}

impl From<u32> for SleepMode {
    fn from(n: u32) -> Self {
        match n {
            0 => SleepMode::DeepSleep,
            1 => SleepMode::Sleep,
            2 => SleepMode::Active,
            _ => unimplemented!(),
        }
    }
}

pub struct Cc26X2 {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    systick: cortexm4::systick::SysTick,
}

impl Cc26X2 {
    // internal HFREQ is 40_000_000 Hz
    // but if you are using an external HFREQ to derive systick, you will want to input value here (in Hz)
    pub unsafe fn new(hfreq: u32) -> Cc26X2 {
        Cc26X2 {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            // The systick clocks with 48MHz by default
            systick: cortexm4::systick::SysTick::new_with_calibration(hfreq),
        }
    }
}

impl kernel::Chip for Cc26X2 {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SysTick = cortexm4::systick::SysTick;

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn systick(&self) -> &Self::SysTick {
        &self.systick
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        unsafe {
            while let Some(event) = events::next_pending() {
                self.wdt_enable();
                events::clear_event_flag(event);
                match event {
                    EVENT_PRIORITY::GPIO => gpio::PORT.handle_events(),
                    EVENT_PRIORITY::AON_RTC => rtc::RTC.handle_events(),
                    EVENT_PRIORITY::I2C0 => i2c::I2C0.handle_events(),
                    EVENT_PRIORITY::UART0 => uart::UART0.handle_events(),
                    EVENT_PRIORITY::UART1 => uart::UART1.handle_events(),
                    EVENT_PRIORITY::RF_CMD_ACK => radio::RFC.handle_ack_event(),
                    EVENT_PRIORITY::RF_CORE_CPE0 => radio::RFC.handle_cpe0_event(),
                    EVENT_PRIORITY::RF_CORE_CPE1 => radio::RFC.handle_cpe1_event(),
                    EVENT_PRIORITY::RF_CORE_HW => panic!("Unhandled RFC interupt event!"),
                    EVENT_PRIORITY::AUX_ADC => adc::ADC.handle_events(),
                    EVENT_PRIORITY::OSC => prcm::handle_osc_interrupt(),
                    EVENT_PRIORITY::AON_PROG => (),
                    _ => panic!("unhandled event {:?} ", event),
                }
                self.wdt_disable();
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        events::has_event()
    }

    fn wdt_enable(&self) {
        wdt::enable();
    }

    fn wdt_disable(&self) {
        wdt::disable();
    }

    fn sleep(&self) {
        unsafe {
            cortexm4::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm4::support::atomic(f)
    }
}
