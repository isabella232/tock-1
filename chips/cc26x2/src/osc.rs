use kernel::common::registers::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use prcm;
use rom;

pub struct DdiRegisters {
    pub ctl0: ReadWrite<u32, Ctl0::Register>,
    pub ctl1: ReadOnly<u32, Ctl1::Register>,
    pub radc_ext_cfg: ReadWrite<u32, RadCExtCfg::Register>,
    pub amp_comp_ctl: ReadWrite<u32, AmpCompCtl0::Register>,
    pub amp_comp_th1: ReadWrite<u32, AmpCompTh1::Register>,
    pub amp_comp_th2: ReadWrite<u32, AmpCompTh2::Register>,

    pub ana_bypass_val1: ReadWrite<u32, AnaBypassVal1::Register>,
    pub ana_bypass_val2: ReadWrite<u32, AnaBypassVal2::Register>,

    pub analog_test_ctl: ReadWrite<u32, AnalogTestCtl::Register>,
    pub adc_doubler_nanoamp_ctl: ReadWrite<u32, AdcDoublerNanoAmpCtl::Register>,

    pub xosc_hf_ctl: ReadWrite<u32, XOscHfCtl::Register>,
    pub lf_osc_ctl: ReadWrite<u32, LfOscCtl::Register>,
    pub rc_osc_hf_ctl: ReadWrite<u32, RcOscHfCtl::Register>,
    pub rc_osc_mf_ctl: ReadWrite<u32, RcOscMfCtl::Register>,

    _reserved: ReadOnly<u32>,

    pub stat0: ReadOnly<u32, Stat0::Register>,
    pub stat1: ReadOnly<u32, Stat1::Register>,
    pub stat2: ReadOnly<u32, Stat2::Register>,
}

register_bitfields! [
    u32,
    Ctl0 [
        XTAL_IS_24M              OFFSET(31) NUMBITS(1) [],
        // RESERVED 30
        BYPASS_XOSC_LF_CLK_QUAL  OFFSET(29) NUMBITS(1) [],
        BYPASS_RCOSC_LF_CLK_QUAL OFFSET(28) NUMBITS(1) [],
        DOUBLER_START_DURATION   OFFSET(26) NUMBITS(2) [],
        DOUBLER_RESET_DURATION   OFFSET(25) NUMBITS(1) [],
        CLK_DCDC_SRC_SEL         OFFSET(24) NUMBITS(1) [],
        // RESERVED 15-23
        HPOSC_MODE_ON            OFFSET(14) NUMBITS(1) [],
        // RESERVED 13
        RCOSC_LF_TRIMMED         OFFSET(12) NUMBITS(1) [],
        XOSC_HF_POWER_MODE       OFFSET(11) NUMBITS(1) [],
        XOSC_LF_DIG_BYPASS       OFFSET(10) NUMBITS(1) [],

        CLK_LOSS_EN              OFFSET(9) NUMBITS(1) [],
        ACLK_TDC_SRC_SEL         OFFSET(7) NUMBITS(2) [],
        ACLK_REF_SRC_SEL         OFFSET(4) NUMBITS(3) [],

        SCLK_LF_SRC_SEL          OFFSET(2) NUMBITS(2) [],
        // RESERVED 1
        SCLK_HF_SRC_SEL     OFFSET(0) NUMBITS(1) []
    ],
    Ctl1 [
        // 31-23 RESERVED
        RCOSC_HFCTRIM_FRACT     OFFSET(18) NUMBITS(5) [],
        RFOSC_HFCTRIM_FRACT_EN  OFFSET(17) NUMBITS(1) [],
        SPARE2                  OFFSET(2) NUMBITS(16) [],
        XOSC_HF_FAST_START      OFFSET(0) NUMBITS(2) []
    ],
    RadCExtCfg [
        HPM_IBIAS_WAIT_CNT      OFFSET(22) NUMBITS(9) [],
        LPM_IBIAS_WAIT_CNT      OFFSET(16) NUMBITS(5) [],
        IDAC_STEP               OFFSET(12) NUMBITS(3) [],
        RADC_DAC_TH             OFFSET(6) NUMBITS(6) [],
        RDAC_MODE_IS_SAR        OFFSET(5) NUMBITS(1) []
        // Reserved 4 - 0
    ],
    AmpCompCtl0 [
        SPARE31                 OFFSET(31) NUMBITS(1) [],
        AMPCOMP_REQ_MODE        OFFSET(30) NUMBITS(1) [],
        AMPCOMP_FSM_UPDATE_RATE OFFSET(28) NUMBITS(2) [],
        AMPCOMP_SW_CTRL         OFFSET(27) NUMBITS(1) [],
        AMPCOMP_SW_EN           OFFSET(26) NUMBITS(1) [],
        // Reserved 25 -24
        IBIAS_OFFSET            OFFSET(20) NUMBITS(4) [],
        IBIAS_INIT              OFFSET(16) NUMBITS(4) [],
        LPM_IBIAS_WAIT_CNT_FINAL OFFSET(8) NUMBITS(8) [],
        CAP_STEP                OFFSET(4) NUMBITS(4) [],
        IBIASCAP_HPTOLP_OL_CNT  OFFSET(0) NUMBITS(4) []
    ],
    AmpCompTh1 [
        SPARE24                 OFFSET(24) NUMBITS(8) [],
        HPMRAMP3_LTH            OFFSET(18) NUMBITS(23) [],
        SPARE16                 OFFSET(16) NUMBITS(17) [],
        HPMRAMP3_HTH            OFFSET(10) NUMBITS(6) [],
        IBIASCAP_LPTOHP_OL_CNT  OFFSET(6) NUMBITS(4) [],
        HPMRAMP1_TH             OFFSET(0) NUMBITS(6) []
    ],
    AmpCompTh2 [
        LPMUPDATE_LTH           OFFSET(26) NUMBITS(6) [],
        SPARE16                 OFFSET(16) NUMBITS(2) [],
        ADC_COMP_AMPTH_LPM      OFFSET(10) NUMBITS(6) [],
        SPARE8                  OFFSET(8) NUMBITS(2) [],
        ADC_COMP_AMPTH_HPM      OFFSET(2) NUMBITS(6) [],
        SPARE0                  OFFSET(0) NUMBITS(2) []
    ],
    AnaBypassVal1 [
        // Reserved 31-20
        XOSC_HF_ROW_Q12         OFFSET(16) NUMBITS(4) [],
        XOSC_HF_COLUMN_Q12      OFFSET(0) NUMBITS(16) []
    ],
    AnaBypassVal2 [
        // Reserved 31-14
        XOSC_HF_IBIASTHERM      OFFSET(0) NUMBITS(14) []
    ],
    AnalogTestCtl [
        SCLK_LF_AUX_EN          OFFSET(31) NUMBITS(1) [],
        // Reserved 30-16
        TEST_RCOSCMF            OFFSET(14) NUMBITS(2) [],
        ATEST_RCOSCMF           OFFSET(12) NUMBITS(2) []
        // Reserved 11-0
    ],
    AdcDoublerNanoAmpCtl [
        // Reserved 31-25
        NANOAMP_BIAS_ENABLE     OFFSET(24) NUMBITS(1) [],
        SPARE23                 OFFSET(23) NUMBITS(1) [],
        // Reserved 22-6
        ADC_SH_MODE_EN          OFFSET(5) NUMBITS(1) [],
        ADC_SH_VBUF_EN          OFFSET(4) NUMBITS(1) [],
        // Reserved 3-2
        ADC_IREF_CTRL           OFFSET(0) NUMBITS(2) []
    ],
    XOscHfCtl [
        SPARE14                 OFFSET(14) NUMBITS(18) [],
        TCXO_MODE_XOSC_HF_EN    OFFSET(13) NUMBITS(1) [],
        TCXO_MODE               OFFSET(12) NUMBITS(1) [],
        // Reserved 11-10
        PEAK_DET_ITRIM          OFFSET(8) NUMBITS(2) [],
        // Reserved 7
        BYPASS                  OFFSET(6) NUMBITS(1) [],
        // Reserved 5 
        HP_BUF_ITRIM            OFFSET(2) NUMBITS(3) [],
        LP_BUF_ITRIM            OFFSET(0) NUMBITS(2) []
    ],
    LfOscCtl [
        // Reserved 31-24
        XOSCLF_REGULATOR_TRIM   OFFSET(22) NUMBITS(2) [],
        XOSCLF_CMIRRWR_RATIO    OFFSET(18) NUMBITS(3) [],
        // Reserved 17-10
        RCOSCLF_RTUNE_TRIM      OFFSET(8) NUMBITS(2) [],
        RCOSCLF_CTUNE_TRIM      OFFSET(0) NUMBITS(8) []
    ],
    RcOscHfCtl [
        // Reserved 31 - 16
        RCOSCHF_CTRIM           OFFSET(8) NUMBITS(8) []
        // Reserved 7-0
    ],
    RcOscMfCtl [
        SPARE16                 OFFSET(16) NUMBITS(16) [],
        RCOSC_MF_CAP_ARRAY      OFFSET(9) NUMBITS(7) [],
        RCOSC_MF_REG_SEL        OFFSET(8) NUMBITS(1) [],
        RCOSC_MF_RES_COARSE     OFFSET(6) NUMBITS(2) [],
        RCOSC_MF_RES_FINE       OFFSET(4) NUMBITS(2) [],
        RCOSC_MF_BIAS_ADJ       OFFSET(0) NUMBITS(4) []
    ],
    Stat0 [
        // RESERVED 31
        SCLK_LF_SRC     OFFSET(29) NUMBITS(2) [],
        SCLK_HF_SRC     OFFSET(28) NUMBITS(1) [],
        // RESERVED 23-27
        RCOSC_HF_EN      OFFSET(22) NUMBITS(1) [],
        RCOSC_LF_EN      OFFSET(21) NUMBITS(1) [],
        XOSC_LF_EN       OFFSET(20) NUMBITS(1) [],
        CLK_DCDC_RDY     OFFSET(19) NUMBITS(1) [],
        CLK_DCDC_RDY_ACK OFFSET(18) NUMBITS(1) [],

        SCLK_HF_LOSS     OFFSET(17) NUMBITS(1) [],
        SCLK_LF_LOSS     OFFSET(16) NUMBITS(1) [],
        XOSC_HF_EN       OFFSET(15) NUMBITS(1) [],
        // RESERVED 14
        // Indicates the 48MHz clock from the DOUBLER enabled
        XB_48M_CLK_EN    OFFSET(13) NUMBITS(1) [],
        // RESERVED 12
        XOSC_HF_LP_BUF_EN OFFSET(11) NUMBITS(1) [],
        XOSC_HF_HP_BUF_EN OFFSET(10) NUMBITS(1) [],
        // RESERVED 9
        ADC_THMET       OFFSET(8) NUMBITS(1) [],
        ADC_DATA_READY  OFFSET(7) NUMBITS(1) [],
        ADC_DATA        OFFSET(1) NUMBITS(6) [],
        PENDING_SCLK_HF_SWITCHING OFFSET(0) NUMBITS(1) []
    ],
    Stat1 [
        RAMPSTATE           OFFSET(28) NUMBITS(4) [],
        HPM_UPDATE_AMP      OFFSET(22) NUMBITS(6) [],
        LPM_UPDATE_AMP      OFFSET(16) NUMBITS(6) [],
        FORCE_RCOSC_HF      OFFSET(15) NUMBITS(1) [],
        SCLK_HF_EN          OFFSET(14) NUMBITS(1) [],
        SCLK_MF_EN          OFFSET(13) NUMBITS(1) [],
        ACLK_ADC_EN         OFFSET(12) NUMBITS(1) [],
        ACLK_TDC_EN         OFFSET(11) NUMBITS(1) [],
        ACLK_REF_EN         OFFSET(10) NUMBITS(1) [],
        CLK_CHP_EN          OFFSET(9) NUMBITS(1) [],
        CLK_DCDC_EN         OFFSET(8) NUMBITS(1) [],
        SCLK_HF_GOOD        OFFSET(7) NUMBITS(1) [],
        SCLK_MF_GOOD        OFFSET(6) NUMBITS(1) [],
        SCLK_LF_GOOD        OFFSET(5) NUMBITS(1) [],
        ACLK_ADC_GOOD       OFFSET(4) NUMBITS(1) [],
        ACLK_TDC_GOOD       OFFSET(3) NUMBITS(1) [],
        ACLK_REF_GOOD       OFFSET(2) NUMBITS(1) [],
        CLK_CHP_GOOD        OFFSET(1) NUMBITS(1) [],
        CLK_DCDC_GOOD       OFFSET(0) NUMBITS(1) []
    ],
    Stat2 [
        ADC_DCBIAS          OFFSET(26) NUMBITS(6) [],
        HPM_RAMP1_THMET     OFFSET(25) NUMBITS(1) [],
        HPM_RAMP2_THMET     OFFSET(24) NUMBITS(1) [],
        HPM_RAMP3_THMET     OFFSET(23) NUMBITS(1) [],
        // Reserved 22-16
        RAMPSTATE           OFFSET(12) NUMBITS(4) [],
        // Reserved 11-4
        AMPCOMP_REQ         OFFSET(3) NUMBITS(1) [],
        XOSC_HF_AMPGOOD     OFFSET(2) NUMBITS(1) [],
        XOSC_HF_FREQGOOD    OFFSET(1) NUMBITS(1) [],
        XOSC_HF_RF_FREQGOOD OFFSET(0) NUMBITS(1) []
    ]
];

pub enum ClockType {
    LF,
    HF,
}

pub const HF_RCOSC: u8 = 0x00;
pub const HF_XOSC: u8 = 0x01;

pub const LF_DERIVED_RCOSC: u8 = 0x00;
pub const LF_DERIVED_XOSC: u8 = 0x01;
pub const LF_RCOSC: u8 = 0x02;
pub const LF_XOSC: u8 = 0x03;

pub const DDI_0_R_BASE: StaticRef<DdiRegisters> =
    unsafe { StaticRef::new(0x400C_A000 as *const DdiRegisters) };

pub const OSC: Oscillator = Oscillator::new();

pub struct Oscillator {
    dir_regs: StaticRef<DdiRegisters>,
}

impl Oscillator {
    pub const fn new() -> Oscillator {
        Oscillator {
            dir_regs: DDI_0_R_BASE,
        }
    }

    pub fn switch_to_rc_osc(&self) {
        if self.clock_source_get(ClockType::HF) != HF_RCOSC {
            self.clock_source_set(ClockType::HF, HF_RCOSC);
        }

        self.clock_source_set(ClockType::LF, LF_RCOSC);
        self.disable_lfclk_qualifier();
    }

    // Check if the current clock source is HF_XOSC. If not, set it.
    pub fn request_switch_to_hf_xosc(&self) {
        prcm::disable_osc_interrupt();

        if self.clock_source_get(ClockType::HF) != HF_XOSC {
            self.clock_source_set(ClockType::HF, HF_XOSC);
        }
    }

    // Check if current clock source is HF_XOSC. If not, wait until request is done, then set it in
    // ddi
    pub fn switch_to_hf_xosc(&self) {
        if self.clock_source_get(ClockType::HF) != HF_XOSC {
            // Wait for source ready to switch
            let regs = &*self.dir_regs;

            while !regs.stat0.is_set(Stat0::PENDING_SCLK_HF_SWITCHING) {}

            self.switch_osc();
        }
    }

    pub fn switch_to_hf_rcosc(&self) {
        let regs = self.dir_regs;

        self.clock_source_set(ClockType::HF, HF_RCOSC);
        while !regs.stat0.is_set(Stat0::PENDING_SCLK_HF_SWITCHING) {}
        if self.clock_source_get(ClockType::HF) != HF_RCOSC {
            self.switch_osc();
        }
    }

    pub fn disable_lfclk_qualifier(&self) {
        let regs = self.dir_regs;

        while self.clock_source_get(ClockType::LF) != LF_RCOSC {}

        regs.ctl0
            .modify(Ctl0::BYPASS_XOSC_LF_CLK_QUAL::SET + Ctl0::BYPASS_RCOSC_LF_CLK_QUAL::SET);
    }

    // Get the current clock source of either LF or HF sources
    pub fn clock_source_get(&self, source: ClockType) -> u8 {
        let regs = self.dir_regs;
        match source {
            ClockType::LF => regs.stat0.read(Stat0::SCLK_LF_SRC) as u8,
            ClockType::HF => regs.stat0.read(Stat0::SCLK_HF_SRC) as u8,
        }
    }

    // Set the clock source in DDI_0_OSC
    pub fn clock_source_set(&self, clock: ClockType, src: u8) {
        let regs = self.dir_regs;
        match clock {
            ClockType::LF => {
                regs.ctl0.modify(Ctl0::SCLK_LF_SRC_SEL.val(src as u32));
            }
            ClockType::HF => {
                regs.ctl0.modify(Ctl0::SCLK_HF_SRC_SEL.val(src as u32));
                match src {
                    0 => regs.ctl0.modify(Ctl0::ACLK_REF_SRC_SEL.val(0b000)),
                    1 => regs.ctl0.modify(Ctl0::ACLK_REF_SRC_SEL.val(0b001)),
                    _ => (),
                }
            }
        }
    }

    pub fn rcosc_hf_trim_get(&self) -> u32 {
        let regs = self.dir_regs;
        regs.rc_osc_hf_ctl.read(RcOscHfCtl::RCOSCHF_CTRIM)
    }

    pub fn rcosc_hf_trim_set(&self, val: u32) {
        let regs = self.dir_regs;
        regs.rc_osc_hf_ctl
            .modify(RcOscHfCtl::RCOSCHF_CTRIM.val(val ^ 0xc0));
    }

    pub fn xosc_hf_set_ana_bypass_1(&self, val: u32) {
        let regs = self.dir_regs;
        regs.ana_bypass_val1.set(val);
    }

    pub fn set_hposc(&self) {
        let dir_regs = self.dir_regs;
        dir_regs.ctl0.modify(Ctl0::HPOSC_MODE_ON::SET);
    }

    pub fn set_xosc_bypass(&self) {
        let dir_regs = self.dir_regs;
        dir_regs.xosc_hf_ctl.modify(XOscHfCtl::BYPASS::SET);
    }

    pub fn set_clock_loss_en(&self) {
        let dir_regs = self.dir_regs;
        dir_regs.ctl0.modify(Ctl0::CLK_LOSS_EN::SET);
    }

    pub fn set_digital_bypass(&self) {
        let dir_regs = self.dir_regs;
        dir_regs.ctl0.modify(Ctl0::XOSC_LF_DIG_BYPASS::SET);
    }

    pub fn set_xtal_24mhz(&self) {
        let dir_regs = self.dir_regs;
        dir_regs.ctl0.modify(Ctl0::XTAL_IS_24M::SET);
    }
    // Switch the source OSC in DDI0
    pub fn switch_osc(&self) {
        unsafe {
            (rom::HAPI.hf_source_safe_switch)();
        }
    }
}
