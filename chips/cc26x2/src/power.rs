use crate::power_manager::{PowerManager, Resource, ResourceManager};
use crate::prcm::{Power, PowerDomain};
use cortexm4::scb;
use kernel::common::cells::VolatileCell;

use crate::aon;
use crate::aux;
use crate::osc;
use crate::prcm;
use crate::rtc;

pub static mut PM: PowerManager<RegionManager> = PowerManager::new(RegionManager);

pub static mut POWER_REGIONS: [Resource; 2] = [
    Resource::new(PowerDomain::Serial as u32),
    Resource::new(PowerDomain::Peripherals as u32),
];

pub struct RegionManager;

impl ResourceManager for RegionManager {
    fn enable_resource(&self, resource_id: u32) {
        let domain = PowerDomain::from(resource_id);
        Power::enable_domain(domain);
    }

    fn disable_resource(&self, resource_id: u32) {
        let domain = PowerDomain::from(resource_id);
        Power::disable_domain(domain);
    }
}

/// Initialise the power management,dependencies and resources.
pub unsafe fn init() {
    for pwr_region in POWER_REGIONS.iter() {
        PM.register_resource(&pwr_region);
    }
}

fn vims_disable() {
    const VIMS_BASE: u32 = 0x4003_4000;
    const VIMS_O_CTL: u32 = 0x00000004;

    let vims_ctl: &VolatileCell<u32> =
        unsafe { &*((VIMS_BASE + VIMS_O_CTL) as *const VolatileCell<u32>) };
    vims_ctl.set(0x00000003); // disable VIMS
}

pub fn switch_to_rc_oscillator() {
    if osc::OSC.clock_source_get(osc::ClockType::HF) != osc::HF_RCOSC {
        osc::OSC.switch_to_hf_rcosc();
    }
    osc::OSC.clock_source_set(osc::ClockType::LF, 0x2);
    osc::OSC.disable_lfclk_qualifier();
}

/// Transition into deep sleep
pub fn prepare_deep_sleep() {
    // Need to set pins to some default configuration here but still unimplemented
    //gpio::set_pins_to_default_conf();

    switch_to_rc_oscillator();

    prcm::Power::disable_domain(prcm::PowerDomain::CPU);
    prcm::Power::disable_domain(prcm::PowerDomain::RFC);
    prcm::Power::disable_domain(prcm::PowerDomain::Serial);
    prcm::Power::disable_domain(prcm::PowerDomain::Peripherals);
    prcm::Power::disable_domain(prcm::PowerDomain::VIMS);

    prcm::acquire_uldo();
    prcm::force_disable_dma_and_crypto();

    aon::AON.set_dcdc_enabled(true);
    aon::AON.jtag_set_enabled(false);
    aon::AON.aux_disable_power_down_clock();
    aon::AON.aux_set_ram_retention(false);
    aon::AON.mcu_set_ram_retention(true);
    aon::AON.lock_io_pins(true);

    // AUX domain may need to sleep here
    aux::sysif::AUX.operation_mode_request(aux::sysif::WUMODE_PDA);
    while aux::sysif::AUX.operation_mode_ack() != aux::sysif::WUMODE_PDA {}

    // TODO: if we power off the aux completely we prevent the second wakeup,
    //       and cause a hard-fault during the next access to the AUX domain/bus (eg. osc control)
    //       Investigate this further, as the AUX domain draws ~70uA in sleep

    while !aon::AON.aux_reset_done() {}

    // Configure power cycling (used to keep state in low power modes)
    //recharge::before_power_down(0);

    vims_disable();
    unsafe {
        rtc::RTC.sync();
        scb::set_sleepdeep();
    }
}

/// Perform necessary setup once we've woken up from deep sleep
pub fn prepare_wakeup() {
    // Once we've woken up we need to sync with the RTC to be able
    // to read values which has changed in the AON region during sleep.
    unsafe {
        rtc::RTC.sync();
    }

    // We're ready to allow the auxilliary domain to wake up once it's needed.
    aux::sysif::AUX.operation_mode_request(aux::sysif::WUMODE_A);

    // If we were using the uLDO power to supply the peripherals, we can safely disable it now
    prcm::release_uldo();

    prcm::Power::enable_domain(prcm::PowerDomain::CPU);
    prcm::Power::enable_domain(prcm::PowerDomain::Peripherals);
    prcm::Power::enable_domain(prcm::PowerDomain::Serial);

    // Unlock IO pins and let them be controlled by GPIO
    aon::AON.lock_io_pins(false);

    unsafe {
        rtc::RTC.sync();
        scb::unset_sleepdeep();
    }
}
