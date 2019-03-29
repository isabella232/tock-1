use crate::chip::SleepMode;
use crate::enum_primitive::cast::FromPrimitive;
use crate::osc;
use crate::peripheral_manager::PowerClient;
use crate::radio::commands as cmd;
use crate::radio::commands::{
    prop_commands as prop, DirectCommand, RfcCondition, RfcTrigger, CWM_RFPARAMS, LR_RFPARAMS,
    TX_20_PARAMS, TX_STD_PARAMS_10, TX_STD_PARAMS_2, TX_STD_PARAMS_9,
};
use crate::radio::queue;
use crate::radio::rfc;
use crate::rtc;
use core::cell::Cell;
use core::slice;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::rf_frontend;
use kernel::hil::rfcore;
use kernel::hil::rfcore::PaType;
use kernel::ReturnCode;

// Fields for testing
const TEST_PAYLOAD: [u8; 32] = [0; 32];
enum_from_primitive! {
pub enum TestType {
    Tx = 0,
    Rx = 1,
    Cwm = 2,
}
}

const MAX_RX_LENGTH: u16 = 255;

struct _32BitAlignedU8Array {
    buf: [u8; 256],
    _alignment: [u32; 0],
}

impl _32BitAlignedU8Array {
    pub const fn new() -> _32BitAlignedU8Array {
        _32BitAlignedU8Array {
            buf: [0; 256],
            _alignment: [],
        }
    }
}

static mut COMMAND: _32BitAlignedU8Array = _32BitAlignedU8Array::new();
static mut TX_BUF: [u8; 256] = [0; 256];

static mut RX_BUF: [u8; 512] = [0; 512];
static mut RX_DAT: [u8; 16] = [0; 16];
static mut RX_PAYLOAD: [u8; 256] = [0; 256];

#[allow(unused)]
// TODO Implement update config for changing radio modes and tie in the WIP power client to manage
// power state.
pub struct Radio {
    rfc: &'static rfc::RFCore,
    tx_client: OptionalCell<&'static rfcore::TxClient>,
    rx_client: OptionalCell<&'static rfcore::RxClient>,
    power_client: OptionalCell<&'static rfcore::PowerClient>,
    frontend_client: OptionalCell<&'static rf_frontend::SE2435L>,
    update_config: Cell<bool>,
    schedule_powerdown: Cell<bool>,
    can_sleep: Cell<bool>,
    tx_buf: TakeCell<'static, [u8]>,
    rx_buf: TakeCell<'static, [u8]>,
    tx_power: Cell<u16>,
    pub pa_type: Cell<PaType>,
}

// 2 dB PWR 0x12C9
// 9 dB PWR 0x3248
impl Radio {
    pub const fn new(rfc: &'static rfc::RFCore) -> Radio {
        Radio {
            rfc,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            power_client: OptionalCell::empty(),
            frontend_client: OptionalCell::empty(),
            update_config: Cell::new(false),
            schedule_powerdown: Cell::new(false),
            can_sleep: Cell::new(false),
            tx_buf: TakeCell::empty(),
            rx_buf: TakeCell::empty(),
            tx_power: Cell::new(0x38DA),
            pa_type: Cell::new(PaType::None),
        }
    }

    pub fn power_up(&self) {
        // TODO Need so have some mode setting done in initialize callback perhaps to pass into
        // power_up() here, the RadioMode enum is defined above which will set a mode in this
        // multimode context along with applying the patches which are attached. Maybe it would be
        // best for the client to just pass an int for the mode and do it all here? not sure yet.

        self.rfc.set_mode(rfc::RfcMode::BLE);

        osc::OSC.request_switch_to_hf_xosc();

        self.rfc.enable();

        self.rfc.start_rat();

        osc::OSC.switch_to_hf_xosc();

        //self.set_pa_restriction();
        // Need to match on patches here but for now, just default to genfsk patches
        unsafe {
            let reg_overrides: u32 = LR_RFPARAMS.as_mut_ptr() as u32;
            let tx_std_overrides: u32 = TX_STD_PARAMS_9.as_mut_ptr() as u32;
            let tx_20_overrides: u32 = TX_20_PARAMS.as_mut_ptr() as u32;
            self.rfc.setup(
                reg_overrides,
                tx_std_overrides,
                tx_20_overrides,
                self.tx_power.get(),
            );
        }

        self.set_radio_fs();
        self.power_client
            .map(|client| client.power_mode_changed(true));
        self.frontend_client.map(|client| client.bypass());
    }

    pub fn power_down(&self) {
        self.rfc.disable();

        self.frontend_client.map(|client| client.sleep());

        self.power_client
            .map(|client| client.power_mode_changed(false));
    }

    unsafe fn replace_and_send_tx_buffer(&self, buf: &'static mut [u8], len: usize) {
        self.frontend_client.map(|client| client.enable_pa());

        for i in 0..COMMAND.buf.len() {
            COMMAND.buf[i] = 0;
        }
        for i in 0..TX_BUF.len() {
            TX_BUF[i] = 0;
        }
        for (i, c) in buf.as_ref()[0..len].iter().enumerate() {
            TX_BUF[i] = *c;
        }
        self.tx_buf.put(Some(buf));

        self.tx_buf.map(|buf| {
            let p_packet = buf.as_mut_ptr() as u32;

            let cmd: &mut prop::CommandTx =
                &mut *(COMMAND.buf.as_mut_ptr() as *mut prop::CommandTx);
            cmd.command_no = 0x3801;
            cmd.status = 0;
            cmd.p_nextop = 0;
            cmd.start_time = 0;
            cmd.start_trigger = {
                let mut trig = RfcTrigger(0);
                trig.set_trigger_type(0);
                trig.set_enable_cmd(false);
                trig.set_trigger_no(0);
                trig.set_past_trigger(true);
                trig
            };
            cmd.condition = {
                let mut cond = RfcCondition(0);
                cond.set_rule(0x01);
                cond
            };
            cmd.packet_conf = {
                let mut packet = prop::RfcPacketConfTx(0);
                packet.set_fs_off(false);
                packet.set_use_crc(true);
                packet.set_var_len(true);
                packet
            };
            cmd.packet_len = len as u8;
            cmd.sync_word = 0x00000000;
            cmd.packet_pointer = p_packet;

            self.rfc.send_async(cmd).ok();
        });
    }

    unsafe fn start_rx_cmd(&self) -> ReturnCode {
        self.frontend_client.map(|client| client.enable_lna());

        for i in 0..COMMAND.buf.len() {
            COMMAND.buf[i] = 0;
        }

        for i in 0..RX_BUF.len() {
            RX_BUF[i] = 0;
        }

        let cmd: &mut prop::CommandRx = &mut *(COMMAND.buf.as_mut_ptr() as *mut prop::CommandRx);

        let mut data_queue = queue::DataQueue::new(RX_BUF.as_mut_ptr(), RX_BUF.as_mut_ptr());

        data_queue.define_queue(RX_BUF.as_mut_ptr(), 600, 2, MAX_RX_LENGTH + 2);

        let p_queue: *mut queue::DataQueue = &mut data_queue as *mut queue::DataQueue;

        cmd.command_no = 0x3802;
        cmd.status = 0;
        cmd.p_nextop = 0;
        cmd.start_time = 0;
        cmd.start_trigger = 0;
        cmd.condition = {
            let mut cond = RfcCondition(0);
            cond.set_rule(0x01);
            cond
        };
        cmd.packet_conf = {
            let mut packet = prop::RfcPacketConfRx(0);
            packet.set_fs_off(false);
            packet.set_brepeat_ok(false);
            packet.set_brepeat_nok(false);
            packet.set_use_crc(true);
            packet.set_var_len(true);
            packet.set_check_address(false);
            packet.set_end_type(false);
            packet.set_filter_op(false);
            packet
        };
        cmd.rx_config = {
            let mut config = prop::RxConfiguration(0);
            config.set_auto_flush_ignored(true);
            config.set_auto_flush_crc_error(true);
            config.set_include_header(true);
            config.set_include_crc(false);
            config.set_append_rssi(false);
            config.set_append_timestamp(false);
            config.set_append_status(true);
            config
        };
        cmd.sync_word = 0x00000000;
        cmd.max_packet_len = 0xFF;
        cmd.address_0 = 0xAA;
        cmd.address_1 = 0xBB;
        cmd.end_trigger = 0x1;
        cmd.end_time = 0;
        cmd.p_queue = p_queue;
        cmd.p_output = RX_DAT.as_mut_ptr();

        self.rfc
            .send_sync(cmd)
            .and_then(|_| self.rfc.wait(cmd))
            .ok();

        // TODO: Need to do some command success or fail checking return code here
        ReturnCode::SUCCESS
    }

    pub fn run_tests(&self, test: u8, power: u8) {
        self.rfc.set_mode(rfc::RfcMode::BLE);

        osc::OSC.request_switch_to_hf_xosc();
        self.rfc.enable();
        self.rfc.start_rat();

        osc::OSC.switch_to_hf_xosc();

        if let Some(t) = TestType::from_u8(test) {
            match t {
                TestType::Tx => {
                    unsafe {
                        let reg_overrides: u32;
                        let tx_std_overrides: u32;
                        let tx_20_overrides: u32;
                        match power {
                            2 => {
                                self.tx_power.set(0x12C9);
                                reg_overrides = LR_RFPARAMS.as_mut_ptr() as u32;
                                tx_std_overrides = TX_STD_PARAMS_2.as_mut_ptr() as u32;
                                tx_20_overrides = TX_20_PARAMS.as_mut_ptr() as u32;
                            }
                            9 => {
                                self.tx_power.set(0x38DA);
                                reg_overrides = LR_RFPARAMS.as_mut_ptr() as u32;
                                tx_std_overrides = TX_STD_PARAMS_9.as_mut_ptr() as u32;
                                tx_20_overrides = TX_20_PARAMS.as_mut_ptr() as u32;
                            }
                            10 => {
                                reg_overrides = LR_RFPARAMS.as_mut_ptr() as u32;
                                tx_std_overrides = TX_STD_PARAMS_10.as_mut_ptr() as u32;
                                tx_20_overrides = TX_20_PARAMS.as_mut_ptr() as u32;
                            }
                            _ => {
                                reg_overrides = LR_RFPARAMS.as_mut_ptr() as u32;
                                tx_std_overrides = TX_STD_PARAMS_2.as_mut_ptr() as u32;
                                tx_20_overrides = TX_20_PARAMS.as_mut_ptr() as u32;
                                debug!("Invalid power parameter");
                            }
                        }
                        self.rfc.setup(
                            reg_overrides,
                            tx_std_overrides,
                            tx_20_overrides,
                            self.tx_power.get(),
                        );
                    }
                    self.set_radio_fs();
                    self.test_radio_tx();
                }
                TestType::Rx => {
                    unsafe {
                        let reg_overrides: u32;
                        let tx_std_overrides: u32;
                        let tx_20_overrides: u32;
                        match power {
                            2 => {
                                self.tx_power.set(0x12C9);
                                reg_overrides = LR_RFPARAMS.as_mut_ptr() as u32;
                                tx_std_overrides = TX_STD_PARAMS_2.as_mut_ptr() as u32;
                                tx_20_overrides = TX_20_PARAMS.as_mut_ptr() as u32;
                            }
                            9 => {
                                self.tx_power.set(0x38DA);
                                reg_overrides = LR_RFPARAMS.as_mut_ptr() as u32;
                                tx_std_overrides = TX_STD_PARAMS_9.as_mut_ptr() as u32;
                                tx_20_overrides = TX_20_PARAMS.as_mut_ptr() as u32;
                            }
                            10 => {
                                reg_overrides = LR_RFPARAMS.as_mut_ptr() as u32;
                                tx_std_overrides = TX_STD_PARAMS_10.as_mut_ptr() as u32;
                                tx_20_overrides = TX_20_PARAMS.as_mut_ptr() as u32;
                            }
                            _ => {
                                reg_overrides = LR_RFPARAMS.as_mut_ptr() as u32;
                                tx_std_overrides = TX_STD_PARAMS_2.as_mut_ptr() as u32;
                                tx_20_overrides = TX_20_PARAMS.as_mut_ptr() as u32;
                                debug!("Invalid power parameter");
                            }
                        }
                        self.rfc.setup(
                            reg_overrides,
                            tx_std_overrides,
                            tx_20_overrides,
                            self.tx_power.get(),
                        );
                    }
                    self.set_radio_fs();
                    self.test_radio_rx();
                }
                TestType::Cwm => {
                    unsafe {
                        let reg_overrides: u32;
                        let tx_std_overrides: u32;
                        let tx_20_overrides: u32;
                        match power {
                            2 => {
                                self.tx_power.set(0x12C9);
                                reg_overrides = CWM_RFPARAMS.as_mut_ptr() as u32;
                                tx_std_overrides = TX_STD_PARAMS_2.as_mut_ptr() as u32;
                                tx_20_overrides = TX_20_PARAMS.as_mut_ptr() as u32;
                            }
                            9 => {
                                self.tx_power.set(0x38DA);
                                reg_overrides = CWM_RFPARAMS.as_mut_ptr() as u32;
                                tx_std_overrides = TX_STD_PARAMS_9.as_mut_ptr() as u32;
                                tx_20_overrides = TX_20_PARAMS.as_mut_ptr() as u32;
                            }
                            10 => {
                                reg_overrides = CWM_RFPARAMS.as_mut_ptr() as u32;
                                tx_std_overrides = TX_STD_PARAMS_10.as_mut_ptr() as u32;
                                tx_20_overrides = TX_20_PARAMS.as_mut_ptr() as u32;
                            }
                            _ => {
                                reg_overrides = CWM_RFPARAMS.as_mut_ptr() as u32;
                                tx_std_overrides = TX_STD_PARAMS_2.as_mut_ptr() as u32;
                                tx_20_overrides = TX_20_PARAMS.as_mut_ptr() as u32;
                                debug!("Invalid power parameter");
                            }
                        }
                        self.setup_cwm(
                            reg_overrides,
                            tx_std_overrides,
                            tx_20_overrides,
                            self.tx_power.get(),
                        );
                    }
                    self.set_radio_fs();
                    self.test_radio_cwm();
                }
            }
        }
    }

    fn test_radio_tx(&self) {
        self.frontend_client.map(|client| client.enable_pa());
        let mut packet = TEST_PAYLOAD;
        let mut seq: u8 = 0;
        for p in packet.iter_mut() {
            *p = seq;
            seq += 1;
        }
        let p_packet = packet.as_mut_ptr() as u32;

        unsafe {
            for i in 0..COMMAND.buf.len() {
                COMMAND.buf[i] = 0;
            }
        }

        unsafe {
            let cmd: &mut prop::CommandTx =
                &mut *(COMMAND.buf.as_mut_ptr() as *mut prop::CommandTx);
            cmd.command_no = 0x3801;
            cmd.status = 0;
            cmd.p_nextop = 0;
            cmd.start_time = 0;
            cmd.start_trigger = {
                let mut trig = RfcTrigger(0);
                trig.set_trigger_type(0);
                trig.set_enable_cmd(false);
                trig.set_trigger_no(0);
                trig.set_past_trigger(true);
                trig
            };

            cmd.condition = {
                let mut cond = RfcCondition(0);
                cond.set_rule(0x01);
                cond
            };

            cmd.packet_conf = {
                let mut packet = prop::RfcPacketConfTx(0);
                packet.set_fs_off(false);
                packet.set_use_crc(true);
                packet.set_var_len(true);
                packet
            };
            cmd.packet_len = 0x14;
            cmd.sync_word = 0x00000000;
            cmd.packet_pointer = p_packet;

            self.rfc
                .send_sync(cmd)
                .and_then(|_| self.rfc.wait(cmd))
                .ok();
        }

        self.frontend_client.map(|client| client.bypass());
    }

    fn test_radio_rx(&self) {
        self.frontend_client.map(|client| client.enable_lna());

        unsafe {
            for i in 0..COMMAND.buf.len() {
                COMMAND.buf[i] = 0;
            }

            let cmd: &mut prop::CommandRx =
                &mut *(COMMAND.buf.as_mut_ptr() as *mut prop::CommandRx);

            let mut data_queue = queue::DataQueue::new(RX_BUF.as_mut_ptr(), RX_BUF.as_mut_ptr());

            data_queue.define_queue(RX_BUF.as_mut_ptr(), 600, 2, MAX_RX_LENGTH + 2);

            let p_queue: *mut queue::DataQueue = &mut data_queue as *mut queue::DataQueue;

            cmd.command_no = 0x3802;
            cmd.status = 0;
            cmd.p_nextop = 0;
            cmd.start_time = 0;
            cmd.start_trigger = 0;
            cmd.condition = {
                let mut cond = RfcCondition(0);
                cond.set_rule(0x01);
                cond
            };
            cmd.packet_conf = {
                let mut packet = prop::RfcPacketConfRx(0);
                packet.set_fs_off(false);
                packet.set_brepeat_ok(false);
                packet.set_brepeat_nok(false);
                packet.set_use_crc(true);
                packet.set_var_len(true);
                packet.set_check_address(false);
                packet.set_end_type(false);
                packet.set_filter_op(false);
                packet
            };
            cmd.rx_config = {
                let mut config = prop::RxConfiguration(0);
                config.set_auto_flush_ignored(true);
                config.set_auto_flush_crc_error(true);
                config.set_include_header(true);
                config.set_include_crc(false);
                config.set_append_rssi(false);
                config.set_append_timestamp(false);
                config.set_append_status(true);
                config
            };
            cmd.sync_word = 0x00000000;
            cmd.max_packet_len = 0xFF;
            cmd.address_0 = 0xAA;
            cmd.address_1 = 0xBB;
            cmd.end_trigger = 0x1;
            cmd.end_time = 0;
            cmd.p_queue = p_queue;
            cmd.p_output = RX_DAT.as_mut_ptr();

            self.rfc
                .send_sync(cmd)
                .and_then(|_| self.rfc.wait(cmd))
                .ok();
        }
    }

    fn set_radio_fs(&self) {
        let cmd_fs = prop::CommandFS {
            command_no: 0x0803,
            status: 0,
            p_nextop: 0,
            start_time: 0,
            start_trigger: 0,
            condition: {
                let mut cond = RfcCondition(0);
                cond.set_rule(0x01);
                cond
            },
            frequency: 0x038B,
            fract_freq: 0x0000,
            synth_conf: {
                let mut synth = prop::RfcSynthConf(0);
                synth.set_tx_mode(false);
                synth.set_ref_freq(0x00);
                synth
            },
            dummy0: 0x00,
            dummy1: 0x00,
            dummy2: 0x00,
            dummy3: 0x0000,
        };

        self.rfc
            .send_sync(&cmd_fs)
            .and_then(|_| self.rfc.wait(&cmd_fs))
            .ok();
    }

    fn test_radio_cwm(&self) {
        self.frontend_client.map(|client| client.bypass());

        let cmd_cwm = prop::CommandTxTest {
            command_no: 0x808,
            status: 0,
            p_nextop: 0,
            start_time: 0,
            start_trigger: 0,
            condition: {
                let mut cond = RfcCondition(0);
                cond.set_rule(0x01);
                cond
            },
            config: {
                let mut conf = prop::RfcTxTestConf(0);
                conf.set_use_cw(true);
                conf.set_fs_off(false);
                conf.set_whiten_mode(0x0);
                conf
            },
            _reserved0: 0,
            tx_word: 0xAAAA,
            _reserved1: 0,
            end_trigger: {
                let mut trig = cmd::RfcTrigger(0);
                trig.set_trigger_type(0x01);
                trig.set_enable_cmd(false);
                trig.set_trigger_no(0x0);
                trig.set_past_trigger(false);
                trig
            },
            sync_word: 0x930B51DE,
            end_time: 0x00000000,
        };

        self.rfc.send_async(&cmd_cwm);
        //.and_then(|_| self.rfc.wait(&cmd_cwm))
        //.ok();
    }
    // Call commands to setup RFCore with optional register overrides and power output
    pub fn setup_cwm(
        &self,
        reg_overrides: u32,
        tx_std_overrides: u32,
        tx_20_overrides: u32,
        tx_power: u16,
    ) {
        let setup_cmd = prop::CommandRadioDivSetup_P {
            command_no: 0x3807,
            status: 0,
            p_nextop: 0,
            start_time: 0,
            start_trigger: 0,
            condition: {
                let mut cond = cmd::RfcCondition(0);
                cond.set_rule(0x01);
                cond
            },
            modulation: {
                let mut mdl = prop::RfcModulation(0);
                mdl.set_mod_type(0x01);
                mdl.set_deviation(0x64);
                mdl.set_deviation_step(0x0);
                mdl
            },
            symbol_rate: {
                let mut sr = prop::RfcSymbolRate(0);
                sr.set_prescale(0xF);
                sr.set_rate_word(0x8000);
                sr
            },
            rx_bandwidth: 0x52,
            preamble_conf: {
                let mut preamble = prop::RfcPreambleConf(0);
                preamble.set_num_preamble_bytes(0x4);
                preamble.set_pream_mode(0x0);
                preamble
            },
            format_conf: {
                let mut format = prop::RfcFormatConf(0);
                format.set_num_syncword_bits(0x20);
                format.set_bit_reversal(false);
                format.set_msb_first(true);
                format.set_fec_mode(0x0);
                format.set_whiten_mode(0x0);
                format
            },
            config: {
                let mut cfg = cmd::RfcSetupConfig(0);
                cfg.set_frontend_mode(0);
                cfg.set_bias_mode(true);
                cfg.set_analog_config_mode(0x0);
                cfg.set_no_fs_powerup(false);
                cfg
            },
            tx_power: tx_power,
            reg_overrides: reg_overrides,
            center_freq: 0x0395,
            int_freq: 0x8000,
            lo_divider: 0x05,
            reg_override_tx_std: tx_std_overrides,
            reg_override_tx_20: tx_20_overrides,
        };

        self.rfc
            .send_sync(&setup_cmd)
            .and_then(|_| self.rfc.wait(&setup_cmd))
            .ok();
    }

    fn set_pa_restriction(&self) {
        let tx_power = self.tx_power.get();
        let pa = self.pa_type.get();
        match pa {
            PaType::None => (),
            PaType::Internal => (),
            PaType::Skyworks => {
                if tx_power > 0x3248 {
                    self.tx_power.set(0x3248);
                }
            }
        }
    }
}

impl rfc::RFCoreClient for Radio {
    fn command_done(&self) {
        unsafe { rtc::RTC.sync() };

        if self.schedule_powerdown.get() {
            // TODO Need to handle powerdown failure here or we will not be able to enter low power
            // modes
            self.power_down();
            osc::OSC.switch_to_hf_rcosc();

            self.schedule_powerdown.set(false);
            // do sleep mode here later
        }

        self.tx_buf.take().map_or(ReturnCode::ERESERVE, |tbuf| {
            self.tx_client
                .map(move |client| client.transmit_event(tbuf, ReturnCode::SUCCESS));
            ReturnCode::SUCCESS
        });
    }

    fn tx_done(&self) {
        self.frontend_client.map(|client| client.bypass());

        unsafe { rtc::RTC.sync() };

        if self.schedule_powerdown.get() {
            // TODO Need to handle powerdown failure here or we will not be able to enter low power
            // modes
            self.frontend_client.map(|client| client.sleep());

            self.power_down();
            osc::OSC.switch_to_hf_rcosc();

            self.schedule_powerdown.set(false);
            // do sleep mode here later
        }
        self.tx_buf.take().map_or(ReturnCode::ERESERVE, |tbuf| {
            self.tx_client
                .map(move |client| client.transmit_event(tbuf, ReturnCode::SUCCESS));
            ReturnCode::SUCCESS
        });
    }

    fn rx_ok(&self) {
        self.frontend_client.map(|client| client.bypass());
        unsafe {
            rtc::RTC.sync();
            //TODO: FIX THIS DISGUSTING CODE!
            let entry_data: *mut u8 = &mut (*queue::READENTRY).data as *mut u8;
            let packet_p = entry_data.offset(-1);
            let length_p = packet_p.offset(-1);
            let length = *length_p;
            let packet: &[u8] = slice::from_raw_parts(packet_p, length as usize);

            for (i, c) in packet[0..length as usize].iter().enumerate() {
                RX_PAYLOAD[i] = *c;
            }

            self.rx_buf.put(Some(&mut RX_PAYLOAD));
        }

        self.rx_buf.take().map_or(ReturnCode::ERESERVE, |rbuf| {
            debug!("RX: {:X?}", rbuf);
            let frame_len = rbuf.len();
            let crc_valid = true;
            self.rx_client.map(move |client| {
                client.receive_event(rbuf, frame_len, crc_valid, ReturnCode::SUCCESS)
            });
            ReturnCode::SUCCESS
        });
    }

    fn rx_nok(&self) {
        unsafe {
            rtc::RTC.sync();
            self.rx_buf.put(Some(&mut RX_BUF));
        }

        self.rx_buf.take().map_or(ReturnCode::ERESERVE, |rbuf| {
            let frame_len = rbuf.len();
            let crc_valid = true;
            self.rx_client.map(move |client| {
                client.receive_event(rbuf, frame_len, crc_valid, ReturnCode::SUCCESS)
            });
            ReturnCode::SUCCESS
        });
    }

    fn rx_buf_full(&self) {
        unsafe {
            rtc::RTC.sync();
            self.rx_buf.put(Some(&mut RX_BUF));
        }

        self.rx_buf.take().map_or(ReturnCode::ERESERVE, |rbuf| {
            let frame_len = rbuf.len();
            let crc_valid = true;
            self.rx_client.map(move |client| {
                client.receive_event(rbuf, frame_len, crc_valid, ReturnCode::SUCCESS)
            });
            ReturnCode::SUCCESS
        });
    }

    fn rx_entry_done(&self) {
        unsafe {
            rtc::RTC.sync();
            self.rx_buf.put(Some(&mut RX_BUF));
        }

        self.rx_buf.take().map_or(ReturnCode::ERESERVE, |rbuf| {
            let frame_len = rbuf.len();
            let crc_valid = true;
            self.rx_client.map(move |client| {
                client.receive_event(rbuf, frame_len, crc_valid, ReturnCode::SUCCESS)
            });
            ReturnCode::SUCCESS
        });
    }
}

impl rfcore::Radio for Radio {}

impl rfcore::RadioDriver for Radio {
    fn set_transmit_client(&self, tx_client: &'static rfcore::TxClient) {
        self.tx_client.set(tx_client);
    }

    fn set_receive_client(&self, rx_client: &'static rfcore::RxClient, _rx_buf: &'static mut [u8]) {
        self.rx_client.set(rx_client);
    }

    fn set_receive_buffer(&self, _rx_buf: &'static mut [u8]) {
        // maybe make a rx buf only when needed?
    }

    fn set_power_client(&self, power_client: &'static rfcore::PowerClient) {
        self.power_client.set(power_client);
    }

    fn set_rf_frontend_client(&self, frontend_client: &'static rf_frontend::SE2435L) {
        self.frontend_client.set(frontend_client);
    }

    fn transmit(
        &self,
        buf: &'static mut [u8],
        frame_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if frame_len > 240 {
            return (ReturnCode::ENOSUPPORT, Some(buf));
        }

        if self.tx_buf.is_none() {
            unsafe { self.replace_and_send_tx_buffer(buf, frame_len) };
            (ReturnCode::SUCCESS, None)
        } else {
            (ReturnCode::EBUSY, Some(buf))
        }
    }

    fn receive(&self) -> ReturnCode {
        unsafe { self.start_rx_cmd() }
    }
}

impl rfcore::RadioConfig for Radio {
    fn initialize(&self) {
        self.power_up();
    }

    fn reset(&self) {
        self.power_down();
        self.power_up();
    }

    fn stop(&self) -> ReturnCode {
        let cmd_stop = DirectCommand::new(0x0402, 0);
        let stopped = self.rfc.send_direct(&cmd_stop).is_ok();
        if stopped {
            ReturnCode::SUCCESS
        } else {
            ReturnCode::FAIL
        }
    }

    fn is_on(&self) -> bool {
        // TODO IMPL RADIO OPERATION COMMAND PING HERE
        true
    }

    fn busy(&self) -> bool {
        // TODO Might be an obsolete command here in favor of get_command_status and some logic on the
        // user size to determine if the radio is busy. Not sure what is best to have here but
        // arguing best might be bikeshedding
        let status = self.rfc.status.get();
        match status {
            0x0001 => true,
            0x0002 => true,
            _ => false,
        }
    }

    fn config_commit(&self) -> ReturnCode {
        // TODO confirm set new config here
        ReturnCode::SUCCESS
    }

    fn get_tx_power(&self) -> u16 {
        // TODO get tx power radio command
        self.tx_power.get()
    }

    fn get_radio_status(&self) -> u32 {
        // TODO get power status of radio
        0x00000000
    }

    fn get_command_status(&self) -> (ReturnCode, Option<u32>) {
        // TODO get command status specifics
        let status = self.rfc.status.get();
        match status & 0x0F00 {
            0 => (ReturnCode::SUCCESS, Some(status)),
            4 => (ReturnCode::SUCCESS, Some(status)),
            8 => (ReturnCode::FAIL, Some(status)),
            _ => (ReturnCode::EINVAL, Some(status)),
        }
    }

    fn set_tx_power(&self, power: u16) -> ReturnCode {
        // Send direct command for TX power change
        // TODO put some guards around the possible range for TX power
        self.tx_power.set(power);
        self.set_pa_restriction();
        let command = DirectCommand::new(0x0010, self.tx_power.get());
        if self.rfc.send_direct(&command).is_ok() {
            return ReturnCode::SUCCESS;
        } else {
            return ReturnCode::FAIL;
        }
    }

    fn send_stop_command(&self) -> ReturnCode {
        // Send "Gracefull" stop radio operation direct command
        let command = DirectCommand::new(0x0402, 0);
        if self.rfc.send_direct(&command).is_ok() {
            return ReturnCode::SUCCESS;
        } else {
            return ReturnCode::FAIL;
        }
    }

    fn send_kill_command(&self) -> ReturnCode {
        // Send immidiate command kill all radio operation commands
        let command = DirectCommand::new(0x0401, 0);
        if self.rfc.send_direct(&command).is_ok() {
            return ReturnCode::SUCCESS;
        } else {
            return ReturnCode::FAIL;
        }
    }

    fn set_frequency(&self, frequency: u16, fract_freq: u16) -> ReturnCode {
        let cmd_fs = prop::CommandFS {
            command_no: 0x0803,
            status: 0,
            p_nextop: 0,
            start_time: 0,
            start_trigger: 0,
            condition: {
                let mut cond = RfcCondition(0);
                cond.set_rule(0x01);
                cond
            },
            frequency: frequency,
            fract_freq: fract_freq,
            synth_conf: {
                let mut synth = prop::RfcSynthConf(0);
                synth.set_tx_mode(false);
                synth.set_ref_freq(0x00);
                synth
            },
            dummy0: 0x00,
            dummy1: 0x00,
            dummy2: 0x00,
            dummy3: 0x0000,
        };

        if self.rfc.send_async(&cmd_fs).is_ok() {
            return ReturnCode::SUCCESS;
        } else {
            return ReturnCode::FAIL;
        }
    }
}

impl PowerClient for Radio {
    fn before_sleep(&self, _sleep_mode: u32) {}

    fn after_wakeup(&self, _sleep_mode: u32) {}

    fn lowest_sleep_mode(&self) -> u32 {
        if self.can_sleep.get() {
            SleepMode::DeepSleep as u32
        } else {
            SleepMode::Sleep as u32
        }
    }
}
