use kernel::common::registers::ReadOnly;

// Radio setup configuration overrides
//
pub static mut LR_RFPARAMS: [u32; 28] = [
    // override_use_patch_simplelink_long_range.xml
    0x00000847, // PHY: Use MCE RAM patch, RFE RAM patch MCE_RFE_OVERRIDE(1,0,0,1,0,0),
    0x006E88E3, // PHY: Use MCE RAM patch only for Rx (0xE), use MCE ROM bank 6 for Tx (0x6)
    // override_synth_prop_863_930_div5.xml
    0x02400403, // Synth: Use 48 MHz crystal as synth clock, enable extra PLL filtering
    0x00068793, // Synth: Set minimum RTRIM to 6
    0x001C8473, // Synth: Configure extra PLL filtering
    0x00088433, // Synth: Configure extra PLL filtering
    0x000684A3, // Synth: Set Fref to 4 MHz
    0x40014005, // Synth: Configure faster calibration HW32_ARRAY_OVERRIDE(0x4004,1),
    0x180C0618, // Synth: Configure faster calibration
    0xC00401A1, // Synth: Configure faster calibration
    0x00010101, // Synth: Configure faster calibration
    0xC0040141, // Synth: Configure faster calibration
    0x00214AD3, // Synth: Configure faster calibration
    0x02980243, // Synth: Decrease synth programming time-out by 90 us from default (0x0298 RAT ticks = 166 us)
    0x0A480583, // Synth: Set loop bandwidth after lock to 20 kHz
    0x7AB80603, // Synth: Set loop bandwidth after lock to 20 kHz
    0x00000623, // Synth: Set loop bandwidth after lock to 20 kHz
    // override_phy_simplelink_long_range_dsss4.xml
    0x030c5068, // PHY: Configure DSSS SF=4 for payload data HW_REG_OVERRIDE(0x5068,0x030C),
    0x146f5128, // PHY: Set SimpleLink Long Range bit-inverted sync word pattern (uncoded, before spreading to fixed-size 64-bit pattern): 0x146F HW_REG_OVERRIDE(0x5128,0x146F),
    0xeb90512c, // PHY: Set SimpleLink Long Range sync word pattern (uncoded, before spreading to fixed-size 64-bit pattern): 0xEB90 HW_REG_OVERRIDE(0x512C,0xEB90),
    0x362e5124, // PHY: Reduce demodulator correlator threshold for improved Rx sensitivity HW_REG_OVERRIDE(0x5124,0x362E),
    0x004c5118, // PHY: Reduce demodulator correlator threshold for improved Rx sensitivity HW_REG_OVERRIDE(0x5118,0x004C),
    0x3e055140, // PHY: Configure limit on frequency offset compensation tracker HW_REG_OVERRIDE(0x5140,0x3E05),
    // override_phy_rx_frontend_simplelink_long_range.xml
    0x000288A3, // Rx: Set RSSI offset to adjust reported RSSI by -2 dB (default: 0)
    // override_phy_rx_aaf_bw_0xd.xml
    0x7ddf0002, // Rx: Set anti-aliasing filter bandwidth to 0xD (in ADI0, set IFAMPCTL3[7:4]=0xD) ADI_HALFREG_OVERRIDE(0,61,0xF,0xD),
    0xFCFC08C3, // DC/DC regulator: In Tx, use DCDCCTL5[3:0]=0xC (DITHER_EN=1 and IPEAK=4). In Rx, use DCDCCTL5[3:0]=0xC (DITHER_EN=1 and IPEAK=4).
    0x82A86C2B, // txHighPA=0x20AA1B
    0xFFFFFFFF,
];

pub static mut GFSK_RFPARAMS: [u32; 26] = [
    // override_use_patch_prop_genfsk.xml
    0x00000847, // PHY: Use MCE RAM patch, RFE RAM patch MCE_RFE_OVERRIDE(1,0,0,1,0,0),
    // override_synth_prop_863_930_div5.xml
    0x02400403, // Synth: Use 48 MHz crystal as synth clock, enable extra PLL filtering
    0x00068793, // Synth: Set minimum RTRIM to 6
    0x001C8473, // Synth: Configure extra PLL filtering
    0x00088433, // Synth: Configure extra PLL filtering
    0x000684A3, // Synth: Set Fref to 4 MHz
    0x40014005, // Synth: Configure faster calibration HW32_ARRAY_OVERRIDE(0x4004,1),
    0x180C0618, // Synth: Configure faster calibration
    0xC00401A1, // Synth: Configure faster calibration
    0x00010101, // Synth: Configure faster calibration
    0xC0040141, // Synth: Configure faster calibration
    0x00214AD3, // Synth: Configure faster calibration
    0x02980243, // Synth: Decrease synth programming time-out by 90 us from default (0x0298 RAT ticks = 166 us) Synth: Set loop bandwidth after lock to 20 kHz
    0x0A480583, // Synth: Set loop bandwidth after lock to 20 kHz
    0x7AB80603, // Synth: Set loop bandwidth after lock to 20 kHz
    0x00000623, 0x002F6028, //
    // override_phy_tx_pa_ramp_genfsk.xml
    0x50880002, // Tx: Configure PA ramp time, PACTL2.RC=0x3 (in ADI0, set PACTL2[3]=1) ADI_HALFREG_OVERRIDE(0,16,0x8,0x8),
    0x51110002, // Tx: Configure PA ramp time, PACTL2.RC=0x3 (in ADI0, set PACTL2[4]=1) ADI_HALFREG_OVERRIDE(0,17,0x1,0x1),
    // override_phy_rx_frontend_genfsk.xml
    0x001a609c, // Rx: Set AGC reference level to 0x1A (default: 0x2E) HW_REG_OVERRIDE(0x609C,0x001A),
    0x00018883, // Rx: Set LNA bias current offset to adjust +1 (default: 0)
    0x000288A3, // Rx: Set RSSI offset to adjust reported RSSI by -2 dB (default: 0)
    // override_phy_rx_aaf_bw_0xd.xml
    0x7ddf0002, // Rx: Set anti-aliasing filter bandwidth to 0xD (in ADI0, set IFAMPCTL3[7:4]=0xD) ADI_HALFREG_OVERRIDE(0,61,0xF,0xD),
    0xFCFC08C3, // TX power override DC/DC regulator: In Tx with 14 dBm PA setting, use DCDCCTL5[3:0]=0xF (DITHER_EN=1 and IPEAK=7). In Rx, use DCDCCTL5[3:0]=0xC (DITHER_EN=1 and IPEAK=4).
    0x82A86C2B, 0xFFFFFFFF, // Stop word
];

// Radio and data commands bitfields
bitfield! {
    #[derive(Copy, Clone)]
    pub struct RfcTrigger(u8);
    impl Debug;
    pub _trigger_type, _set_trigger_type : 3, 0;
    pub _enable_cmd, _set_enable_cmd      : 4;
    pub _trigger_no, _set_trigger_no      : 6, 5;
    pub _past_trigger, _set_past_trigger  : 7;
}

bitfield! {
    #[derive(Copy, Clone)]
    pub struct RfcCondition(u8);
    impl Debug;
    pub _rule, set_rule : 3, 0;
    pub _skip, _set_skip : 7, 4;
}

bitfield! {
    #[derive(Copy, Clone)]
    pub struct RfcSetupConfig(u16);
    impl Debug;
    pub _frontend_mode, set_frontend_mode: 2, 0;
    pub _bias_mode, set_bias_mode: 3;
    pub _analog_cfg_mode, set_analog_config_mode: 9, 4;
    pub _no_fs_powerup, set_no_fs_powerup: 10;
}

// Radio Commands

// RFC Immediate commands
pub const RFC_CMD0: u16 = 0x607; // found in driverlib SDK
pub const RFC_PING: u16 = 0x406;
pub const RFC_BUS_REQUEST: u16 = 0x40E;
pub const RFC_START_RAT_TIMER: u16 = 0x080A;
pub const RFC_STOP_RAT_TIMER: u16 = 0x0809;
pub const RFC_SETUP: u16 = 0x0802;
pub const RFC_STOP: u16 = 0x0402;
pub const RFC_FS_POWERDOWN: u16 = 0x080D;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DirectCommand {
    pub command_no: u16,
    pub params: u16,
}

impl DirectCommand {
    pub const fn new(command_no: u16, params: u16) -> DirectCommand {
        DirectCommand { command_no, params }
    }
}

#[repr(C)]
pub struct CommandCommon {
    pub command_no: ReadOnly<u16>,
    pub status: ReadOnly<u16>,
    pub p_nextop: ReadOnly<u32>,
    pub ratmr: ReadOnly<u32>,
    pub start_trigger: ReadOnly<u8>,
    pub condition: RfcCondition,
}

#[repr(C)]
pub struct AddDataEntry {
    pub command_no: u16, // 0x0005
    pub _reserved: u16,
    pub p_queue: u32,
    pub p_entry: u32,
}

unsafe impl RadioCommand for AddDataEntry {
    fn guard(&mut self) {}
}

// Command and parameters for radio setup

pub unsafe trait RadioCommand {
    fn guard(&mut self);
}

pub mod prop_commands {
    #![allow(unused)]
    use kernel::common::registers::ReadOnly;
    use radio::commands::{RadioCommand, RfcCondition, RfcSetupConfig, RfcTrigger};

    // Radio and data commands bitfields
    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcModulation(u16);
        impl Debug;
        pub _mod_type, set_mod_type                : 2, 0;
        pub _deviation, set_deviation              : 13, 3;
        pub _deviation_step, set_deviation_step    : 15, 14;
    }

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcSymbolRate(u32);
        impl Debug;
        pub _prescale, set_prescale    : 7, 0;
        pub _rate_word, set_rate_word  : 28, 8;
    }

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcPreambleConf(u8);
        impl Debug;
        pub _num_preamble_bytes, set_num_preamble_bytes    : 5, 0;
        pub _pream_mode, set_pream_mode                    : 6, 7;
    }

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcFormatConf(u16);
        impl Debug;
        pub _num_syncword_bits, set_num_syncword_bits  : 5, 0;
        pub _bit_reversal, set_bit_reversal            : 6;
        pub _msb_first, set_msb_first                  : 7;
        pub _fec_mode, set_fec_mode                    : 11, 8;
        pub _whiten_mode, set_whiten_mode              : 15, 13;
    }

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcPacketConfTx(u8);
        impl Debug;
        pub _fs_off, set_fs_off         : 0;
        pub _reserved, _set_reserved    : 2, 1;
        pub _use_crc, set_use_crc       : 3;
        pub _var_len, set_var_len       : 4;
        pub _reserved2, _set_reserved2  : 7, 5;
    }

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcPacketConfRx(u8);
        impl Debug;
        pub _fs_off, set_fs_off                 : 0;
        pub _brepeat_ok, set_brepeat_ok        : 1;
        pub _brepeat_nok, set_brepeat_nok      : 2;
        pub _use_crc, set_use_crc               : 3;
        pub _var_len, set_var_len               : 4;
        pub _check_address, set_check_address  : 5;
        pub _end_type, set_end_type            : 6;
        pub _filter_op, set_filter_op          : 7;
    }

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcSynthConf(u8);
        impl Debug;
        pub _tx_mode, set_tx_mode       : 0;
        pub _ref_freq, set_ref_freq     : 6, 1;
        pub _reserved, _set_reserved    : 7;
    }

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RxConfiguration(u8);
        impl Debug;
        pub _auto_flush_ignored, set_auto_flush_ignored     : 0;
        pub _auto_flush_crc_error, set_auto_flush_crc_error : 1;
        pub _reserved, _set_reserved                        : 2;
        pub _include_header, set_include_header             : 3;
        pub _include_crc, set_include_crc                   : 4;
        pub _append_rssi, set_append_rssi                   : 5;
        pub _append_timestamp, set_append_timestamp         : 6;
        pub _append_status, set_append_status               : 7;
    }

    // Radio Operation Commands
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct CommandRadioDivSetup {
        pub command_no: u16, // 0x3807
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub modulation: RfcModulation,
        pub symbol_rate: RfcSymbolRate,
        pub rx_bandwidth: u8,
        pub preamble_conf: RfcPreambleConf,
        pub format_conf: RfcFormatConf,
        pub config: RfcSetupConfig,
        pub tx_power: u16,
        pub reg_overrides: u32,
        pub center_freq: u16,
        pub int_freq: u16,
        pub lo_divider: u8,
    }

    unsafe impl RadioCommand for CommandRadioDivSetup {
        fn guard(&mut self) {}
    }

    #[repr(C)]
    pub struct CommandRadioSetup {
        pub command_no: u16, // 0x3806
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub modulation: RfcModulation,
        pub symbol_rate: RfcSymbolRate,
        pub rx_bandwidth: u8,
        pub preamble_conf: RfcPreambleConf,
        pub format_conf: RfcFormatConf,
        pub config: RfcSetupConfig,
        pub tx_power: u16,
        pub reg_overrides: u32,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct CommandSyncRat {
        pub command_no: u16,
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub _reserved: u16,
        pub rat0: u32,
    }

    unsafe impl RadioCommand for CommandSyncRat {
        fn guard(&mut self) {}
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct CommandTx {
        pub command_no: u16, // 0x3801
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub packet_conf: RfcPacketConfTx,
        pub packet_len: u8,
        pub sync_word: u32,
        pub packet_pointer: u32,
    }

    unsafe impl RadioCommand for CommandTx {
        fn guard(&mut self) {}
    }

    // Custom FS
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct CommandFS {
        pub command_no: u16, // 0x0803
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub frequency: u16,
        pub fract_freq: u16,
        pub synth_conf: RfcSynthConf,
        pub dummy0: u8,
        pub dummy1: u8,
        pub dummy2: u8,
        pub dummy3: u16,
    }

    unsafe impl RadioCommand for CommandFS {
        fn guard(&mut self) {}
    }

    #[repr(C)]
    pub struct CommandFSPowerdown {
        pub command_no: u16, // 0x080D
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
    }

    unsafe impl RadioCommand for CommandFSPowerdown {
        fn guard(&mut self) {}
    }

    #[repr(C)]
    pub struct CommandRx {
        pub command_no: u16, // 0x080D
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub packet_conf: RfcPacketConfRx,
        pub rx_config: RxConfiguration,
        pub sync_word: u32,
        pub max_packet_len: u8,
        pub address_0: u8,
        pub address_1: u8,
        pub end_trigger: u8,
        pub end_time: u8,
        pub p_queue: *mut u8,
        pub p_output: *mut u8,
    }

    unsafe impl RadioCommand for CommandRx {
        fn guard(&mut self) {}
    }
}
