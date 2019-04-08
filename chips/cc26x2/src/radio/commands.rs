use kernel::common::registers::ReadOnly;
// Radio setup configuration overrides
//

pub static mut LR_RFPARAMS: [u32; 6] = [
    0x030c5068, 0x50884446, 0x0017609c, 0x000288A3, 0x7ddf0002, 0xFFFFFFFF,
];
pub static mut CWM_RFPARAMS: [u32; 5] =
    [0x50884446, 0x000288A3, 0x001a609c, 0x7ddf0002, 0xFFFFFFFF];
pub static mut TX_STD_PARAMS_10: [u32; 4] = [0x0141362b, 0x11310703, 0x001a6028, 0xFFFFFFFF]; // 10 dB output
pub static mut TX_STD_PARAMS_9: [u32; 4] = [0x00c9222b, 0x11310703, 0x001a6028, 0xFFFFFFFF]; // 9 dB output
pub static mut TX_STD_PARAMS_2: [u32; 4] = [0x004f262b, 0x11310703, 0x001a6028, 0xFFFFFFFF]; // 2 dB output
pub static mut TX_20_PARAMS: [u32; 4] = [0x82a86c2b, 0x11310703, 0x001f6028, 0xFFFFFFFF];

// Radio and data commands bitfields
bitfield! {
    #[derive(Copy, Clone)]
    pub struct RfcTrigger(u8);
    impl Debug;
    pub _trigger_type, set_trigger_type : 3, 0;
    pub _enable_cmd, set_enable_cmd      : 4;
    pub _trigger_no, set_trigger_no      : 6, 5;
    pub _past_trigger, set_past_trigger  : 7;
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

unsafe impl RadioCommand for DirectCommand {}

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

unsafe impl RadioCommand for AddDataEntry {}

// Command and parameters for radio setup

pub unsafe trait RadioCommand {}

pub mod prop_commands {
    #![allow(unused)]
    use crate::radio::commands::{RadioCommand, RfcCondition, RfcSetupConfig, RfcTrigger};
    use crate::radio::queue;
    use kernel::common::registers::ReadOnly;

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

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcHeaderConf(u16);
        impl Debug;
        pub _num_header_bits, set_num_header_bits           : 5, 0;
        pub _len_pos, set_len_pos                           :10, 6;
        pub _num_len_bits, set_num_len_bits                 :15, 11;
    }

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcAddressConf(u16);
        impl Debug;
        pub _addr_type, set_addr_type                       : 0;
        pub _addr_size, set_addr_size                       : 5, 1;
        pub _addr_pos, set_addr_pos                         : 10, 6;
        pub _num_addr, set_num_addr                         : 15, 11;
    }

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcTxTestConf(u8);
        impl Debug;
        pub _use_cw, set_use_cw                             : 0;
        pub _fs_off, set_fs_off                             : 1;
        pub _whiten_mode, set_whiten_mode                   :3, 2;
    }

    // Radio Operation Commands
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct CommandRadioDivSetup_P {
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
        pub reg_override_tx_std: u32,
        pub reg_override_tx_20: u32,
    }

    unsafe impl RadioCommand for CommandRadioDivSetup_P {}
    // Radio Operation Commands
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct CommandRadioDivSetup_R {
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

    unsafe impl RadioCommand for CommandRadioDivSetup_R {}

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

    unsafe impl RadioCommand for CommandSyncRat {}

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct CommandTx {
        pub command_no: u16, // 0x3801
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: RfcTrigger,
        pub condition: RfcCondition,
        pub packet_conf: RfcPacketConfTx,
        pub packet_len: u8,
        pub sync_word: u32,
        pub packet_pointer: u32,
    }

    unsafe impl RadioCommand for CommandTx {}

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

    unsafe impl RadioCommand for CommandFS {}

    #[repr(C)]
    pub struct CommandFSPowerdown {
        pub command_no: u16, // 0x080D
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
    }

    unsafe impl RadioCommand for CommandFSPowerdown {}

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
        pub end_time: u32,
        pub p_queue: *mut queue::DataQueue,
        pub p_output: *mut u8,
        pub _rx_sniff: [u8; 14],
    }

    unsafe impl RadioCommand for CommandRx {}

    #[repr(C)]
    pub struct CommandRxAdv {
        pub command_no: u16, // 0x080D
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub packet_conf: RfcPacketConfRx,
        pub rx_config: RxConfiguration,
        pub sync_word_0: u32,
        pub sync_word_1: u32,
        pub max_packet_len: u16,
        pub header_conf: RfcHeaderConf,
        pub address: RfcAddressConf,
        pub len_offset: u8,
        pub end_trigger: u8,
        pub end_time: u32,
        pub p_addr: u32,
        pub p_queue: *mut queue::DataQueue,
        pub p_output: *mut u8,
        pub _rx_sniff: [u8; 14],
    }

    unsafe impl RadioCommand for CommandRxAdv {}

    #[repr(C)]
    pub struct CommandTxTest {
        pub command_no: u16, // 0808
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub config: RfcTxTestConf,
        pub _reserved0: u8,
        pub tx_word: u16,
        pub _reserved1: u8,
        pub end_trigger: RfcTrigger,
        pub sync_word: u32,
        pub end_time: u32,
    }

    unsafe impl RadioCommand for CommandTxTest {}
}
