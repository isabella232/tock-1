use adi;
use adi::AuxAdi4Registers;
use aux;
use kernel::common::StaticRef;

use memory_map::AUX_ADI4_BASE;

// Redeclaration of bitfield enums s.t. client only needs adc.rs dependency
#[allow(non_camel_case_types)]
pub enum SAMPLE_CYCLE {
    _2p7_us,  // 2.7  uS
    _5p3_us,  // 5.3  uS
    _10p6_us, // 10.6 uS
    _21p3_us, // 21.3 uS
    _42p6_us, // 42.6 uS
    _85p3_us, // 85.3.uS
    _170_us,  // 170  uS
    _341_us,  // 341  uS
    _682_us,  // 682  uS
    _1p37_us, // 1.37 mS
    _2p73_us, // 2.73 mS
    _5p46_ms, // 5.46 mS
    _10p9_ms, // 10.9 mS
}

pub enum SOURCE {
    Fixed4P5V,
    NominalVdds,
}

const AUX_ADI4: StaticRef<AuxAdi4Registers> =
    unsafe { StaticRef::new(AUX_ADI4_BASE as *const AuxAdi4Registers) };

pub struct Adc {
    aux_adi4: StaticRef<AuxAdi4Registers>,
}

pub static mut ADC: Adc = Adc::new();

impl Adc {
    const fn new() -> Adc {
        Adc { aux_adi4: AUX_ADI4 }
    }

    pub fn flush(&self) {
        aux::anaif::REG
            .adc_ctl
            .write(aux::anaif::AdcCtl::CMD::FlushFifo);
        aux::anaif::REG
            .adc_ctl
            .write(aux::anaif::AdcCtl::CMD::Enable);
    }

    // todo: recurring event mode
    pub fn enable(&self, source: SOURCE, sample_time: SAMPLE_CYCLE) {
        // Enable ADC reference
        let source_value;
        match source {
            SOURCE::Fixed4P5V => source_value = adi::Reference0::SRC::FIXED_4P5V,
            SOURCE::NominalVdds => source_value = adi::Reference0::SRC::NOMINAL_VDDS,
        }

        self.aux_adi4
            .reference0
            .write(source_value + adi::Reference0::EN::SET);

        // Enable ADC Clock
        let adc_clk_ctl = &aux::sysif::REGISTERS.adc_clk_ctl;
        adc_clk_ctl.req().write(aux::sysif::Req::CLOCK::Enable);
        // Wait for it to start
        while !adc_clk_ctl
            .ack()
            .matches_all(aux::sysif::Ack::CLOCK::Enabled)
        {}

        // Enable the ADC data interface
        // assume manual for now
        aux::anaif::REG
            .adc_ctl
            .write(aux::anaif::AdcCtl::START_SRC::NO_EVENT + aux::anaif::AdcCtl::CMD::Enable);
        // Notes on how to do it with special events
        // GPT trigger: Configure event routing via MCU_EV to the AUX domain
        // HWREG(EVENT_BASE + EVENT_O_AUXSEL0) = trigger;
        // HWREG(AUX_ANAIF_BASE + AUX_ANAIF_O_ADCCTL) = AUX_ANAIF_ADCCTL_START_SRC_MCU_EV | AUX_ANAIF_ADCCTL_CMD_EN;

        let sample_time_value;
        match sample_time {
            SAMPLE_CYCLE::_2p7_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_2P7_US,
            SAMPLE_CYCLE::_5p3_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_5P3_US,
            SAMPLE_CYCLE::_10p6_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_10P6_US,
            SAMPLE_CYCLE::_21p3_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_21P3_US,
            SAMPLE_CYCLE::_42p6_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_42P6_US,
            SAMPLE_CYCLE::_85p3_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_85P3_US,
            SAMPLE_CYCLE::_170_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_170_US,
            SAMPLE_CYCLE::_341_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_341_US,
            SAMPLE_CYCLE::_682_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_682_US,
            SAMPLE_CYCLE::_1p37_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_1P37_MS,
            SAMPLE_CYCLE::_2p73_us => sample_time_value = adi::Control0::SAMPLE_CYCLE::_2P73_MS,
            SAMPLE_CYCLE::_5p46_ms => sample_time_value = adi::Control0::SAMPLE_CYCLE::_5P46_US,
            SAMPLE_CYCLE::_10p9_ms => sample_time_value = adi::Control0::SAMPLE_CYCLE::_10P9_US,
        }

        self.aux_adi4
            .control0
            .write(sample_time_value + adi::Control0::RESET_N::SET + adi::Control0::EN::SET);
    }

    //     // Enable the ADC clock
    //     HWREG(AUX_SYSIF_BASE + AUX_SYSIF_O_ADCCLKCTL) = AUX_SYSIF_ADCCLKCTL_REQ_M;
    //     while (!(HWREG(AUX_SYSIF_BASE + AUX_SYSIF_O_ADCCLKCTL) & AUX_SYSIF_ADCCLKCTL_ACK_M));

    //     // Enable the ADC data interface
    //     if (trigger == AUXADC_TRIGGER_MANUAL) {
    //         // Manual trigger: No need to configure event routing from GPT
    //         HWREG(AUX_ANAIF_BASE + AUX_ANAIF_O_ADCCTL) = AUX_ANAIF_ADCCTL_START_SRC_NO_EVENT | AUX_ANAIF_ADCCTL_CMD_EN;
    //     } else {
    //         // GPT trigger: Configure event routing via MCU_EV to the AUX domain
    //         HWREG(EVENT_BASE + EVENT_O_AUXSEL0) = trigger;
    //         HWREG(AUX_ANAIF_BASE + AUX_ANAIF_O_ADCCTL) = AUX_ANAIF_ADCCTL_START_SRC_MCU_EV | AUX_ANAIF_ADCCTL_CMD_EN;
    //     }

    //     // Configure the ADC
    //     ADI8BitsSet(AUX_ADI4_BASE, ADI_4_AUX_O_ADC0, sampleTime << ADI_4_AUX_ADC0_SMPL_CYCLE_EXP_S);

    //     // Release reset and enable the ADC
    //     ADI8BitsSet(AUX_ADI4_BASE, ADI_4_AUX_O_ADC0, ADI_4_AUX_ADC0_EN_M | ADI_4_AUX_ADC0_RESET_N_M);
    // }
}
