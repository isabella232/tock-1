use crate::helium::channels::ChannelParams;
use crate::helium::framer::FrameInfo;
use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::rfcore;
use kernel::hil::time::Frequency;
use kernel::ReturnCode;

pub trait RFCore {
    /// Initializes the layer; may require a buffer to temporarily retaining frames to be
    /// transmitted
    fn initialize(&self) -> ReturnCode;
    /// Check if radio is on and ready to accept any command
    fn is_on(&self) -> bool;
    /// Sets the notified client for transmission completions
    fn set_transmit_client(&self, client: &'static rfcore::TxClient);
    /// Sets the notified client for frame receptions
    fn set_receive_client(&self, client: &'static rfcore::RxClient);
    /// Sets the buffer for packet reception
    fn set_receive_buffer(&self, buffer: &'static mut [u8]);

    fn set_power_client(&self, client: &'static rfcore::PowerClient);

    /// Must be called after one or more calls to `set_*`. If
    /// `set_*` is called without calling `config_commit`, there is no guarantee
    /// that the underlying hardware configuration (addresses, pan ID) is in
    /// line with this MAC protocol implementation. The specificed config_client is
    /// notified on completed reconfiguration.
    fn config_commit(&self) -> ReturnCode;

    /// Indicates whether or not the MAC protocol is active and can send frames
    fn get_radio_status(&self) -> bool;

    fn send_stop_command(&self) -> ReturnCode;

    fn send_kill_command(&self) -> ReturnCode;

    fn get_command_status(&self) -> (ReturnCode, Option<u32>);

    fn set_tx_power(&self, power: u16) -> ReturnCode;

    fn set_frequency(&self) -> ReturnCode;
    /// Transmits complete MAC frames, which must be prepared by an ieee802154::device::MacDevice
    /// before being passed to the Mac layer. Returns the frame buffer in case of an error.
    fn transmit(
        &self,
        full_mac_frame: &'static mut [u8],
        frame_info: FrameInfo,
    ) -> (ReturnCode, Option<&'static mut [u8]>);
}

#[derive(Copy, Clone, Debug)]
pub enum RadioState {
    Sleep,
    Awake,
    StartUp,
    TxDone,
    TxPending,
    TxDelay,
}

#[derive(Copy, Clone)]
enum Expiration {
    Disabled,
    Abs(u32),
}

#[derive(Copy, Clone)]
struct AlarmData {
    t0: u32,
    expiration: Expiration,
}

impl AlarmData {
    fn new() -> AlarmData {
        AlarmData {
            t0: 0,
            expiration: Expiration::Disabled,
        }
    }
}
/// Process specific memory
pub struct App {
    process_status: Option<RadioState>,
    alarm_data: AlarmData,

    // Channels meta-data
    adv_data: Option<ChannelParams>,
    channel_interval_ms: u32,
    tx_power: u16,

    /// The state of an app-specific pseudo random number.
    ///
    /// For example, it can be used for the pseudo-random frequency parameter.
    /// It should be read using the `random_number` method, which updates it as
    /// well.
    random_nonce: u32,

    // Scanning meta-data
    hop_callback: Option<kernel::Callback>,
}

impl Default for App {
    fn default() -> App {
        App {
            process_status: Some(RadioState::Sleep),
            alarm_data: AlarmData::new(),
            adv_data: None,
            tx_power: 0,
            channel_interval_ms: 200,
            // Just use any non-zero starting value by default
            random_nonce: 0xdeadbeef,
            hop_callback: None,
        }
    }
}

impl App {
    // Returns a new pseudo-random number and updates the randomness state.
    //
    // Uses the [Xorshift](https://en.wikipedia.org/wiki/Xorshift) algorithm to
    // produce pseudo-random numbers. Uses the `random_nonce` field to keep
    // state.
    fn random_nonce(&mut self) -> u32 {
        let mut next_nonce = ::core::num::Wrapping(self.random_nonce);
        next_nonce ^= next_nonce << 13;
        next_nonce ^= next_nonce >> 17;
        next_nonce ^= next_nonce << 5;
        self.random_nonce = next_nonce.0;
        self.random_nonce
    }

    // Set the next alarm for this app using the period and provided start time.
    fn set_next_alarm<F: Frequency>(&mut self, now: u32) {
        self.alarm_data.t0 = now;
        let nonce = self.random_nonce() % 10;

        let period_ms = (self.channel_interval_ms + nonce) * F::frequency() / 1000;
        self.alarm_data.expiration = Expiration::Abs(now.wrapping_add(period_ms));
    }
}
pub struct VirtualRadio<'a, R: rfcore::Radio> {
    radio: &'a R,
    tx_client: OptionalCell<&'static rfcore::TxClient>,
    rx_client: OptionalCell<&'static rfcore::RxClient>,
    power_client: OptionalCell<&'static rfcore::PowerClient>,
    tx_payload: TakeCell<'static, [u8]>,
    tx_payload_len: Cell<usize>,
    tx_pending: Cell<bool>,
    radio_state: Cell<RadioState>,
    channel_params: Cell<ChannelParams>,
}

impl<R: rfcore::Radio> VirtualRadio<'a, R> {
    pub fn new(radio: &'a R) -> VirtualRadio<'a, R> {
        VirtualRadio {
            radio: radio,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            power_client: OptionalCell::empty(),
            tx_payload: TakeCell::empty(),
            tx_payload_len: Cell::new(0),
            tx_pending: Cell::new(false),
            radio_state: Cell::new(RadioState::Sleep),
            channel_params: Cell::default(),
        }
    }

    pub fn transmit_packet(&self) {
        self.tx_payload.take().map_or((), |buf| {
            let (result, rbuf) = self.radio.transmit(buf, self.tx_payload_len.get());
            match result {
                ReturnCode::SUCCESS => (),
                _ => {
                    if rbuf.is_some() {
                        self.send_client_result(rbuf.unwrap(), result);
                    }
                }
            };
        });
    }

    pub fn send_client_result(&self, buf: &'static mut [u8], result: ReturnCode) {
        self.radio_state.set(RadioState::Awake);
        self.tx_client.map(move |c| {
            c.transmit_event(buf, result);
        });
    }
}

impl<R: rfcore::Radio> RFCore for VirtualRadio<'a, R> {
    fn initialize(&self) -> ReturnCode {
        self.radio_state.set(RadioState::StartUp);
        self.radio.initialize();
        ReturnCode::SUCCESS
    }

    fn is_on(&self) -> bool {
        self.radio.is_on()
    }

    fn set_transmit_client(&self, client: &'static rfcore::TxClient) {
        self.tx_client.set(client);
    }

    fn config_commit(&self) -> ReturnCode {
        self.radio.config_commit()
    }

    fn set_receive_client(&self, client: &'static rfcore::RxClient) {
        self.rx_client.set(client);
    }

    fn set_power_client(&self, client: &'static rfcore::PowerClient) {
        self.power_client.set(client);
    }

    fn set_receive_buffer(&self, buffer: &'static mut [u8]) {
        self.radio.set_receive_buffer(buffer);
    }

    fn get_radio_status(&self) -> bool {
        self.radio.is_on()
    }

    fn send_stop_command(&self) -> ReturnCode {
        self.radio.send_stop_command()
    }

    fn send_kill_command(&self) -> ReturnCode {
        self.radio.send_kill_command()
    }

    fn get_command_status(&self) -> (ReturnCode, Option<u32>) {
        // TODO Parsing with the returned Option<retval> which is some u32 hex code the
        // radio responds with during radio operation command processing
        let (status, _retval) = self.radio.get_command_status();
        (status, None)
    }

    fn set_tx_power(&self, power: u16) -> ReturnCode {
        self.radio.set_tx_power(power)
    }

    fn set_frequency(&self) -> ReturnCode {
        let params = self.channel_params.get();
        self.radio
            .set_frequency(params.frequency, params.fract_freq)
    }
    fn transmit(
        &self,
        frame: &'static mut [u8],
        frame_info: FrameInfo,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if self.tx_payload.is_some() {
            return (ReturnCode::EBUSY, Some(frame));
        } else if frame_info.header.data_len > 240 {
            return (ReturnCode::ESIZE, Some(frame));
        }

        self.tx_payload.replace(frame);
        self.tx_payload_len.set(frame_info.header.data_len);

        if self.radio.is_on() {
            self.radio_state.set(RadioState::TxPending);
            self.transmit_packet();
            return (ReturnCode::SUCCESS, None);
        } else {
            self.radio_state.set(RadioState::StartUp);
            self.tx_pending.set(true);
            self.radio.initialize();
            return (ReturnCode::SUCCESS, None);
        }
    }
}

impl<R: rfcore::Radio> rfcore::TxClient for VirtualRadio<'a, R> {
    fn transmit_event(&self, buf: &'static mut [u8], result: ReturnCode) {
        match self.radio_state.get() {
            // Transmission Completed
            RadioState::TxDone => self.send_client_result(buf, result),
            // Transmission Pending
            RadioState::TxPending => match result {
                ReturnCode::SUCCESS => {
                    self.radio_state.set(RadioState::TxDone);
                    self.send_client_result(buf, result);
                }
                ReturnCode::EBUSY => {
                    self.tx_payload.replace(buf);
                    self.transmit_packet();
                }
                _ => self.radio_state.set(RadioState::TxDone),
            },
            RadioState::TxDelay => {
                // Something has happened and the last TxPending has failed for some reason so
                // replace the buffer and try again
                self.tx_payload.replace(buf);
            }
            _ => {}
        };
    }
}

impl<R: rfcore::Radio> rfcore::RxClient for VirtualRadio<'a, R> {
    fn receive_event(
        &self,
        buf: &'static mut [u8],
        frame_len: usize,
        crc_valid: bool,
        result: ReturnCode,
    ) {
        // Filter packets by destination because radio is in promiscuous mode
        let addr_match = false;
        // CHECK IF THE RECEIVE PACKET DECAUT AND DECODE IS OK HERE

        if addr_match {
            self.rx_client.map(move |c| {
                c.receive_event(buf, frame_len, crc_valid, result);
            });
        } else {
            self.radio.set_receive_buffer(buf);
        }
    }
}

impl<R: rfcore::Radio> rfcore::PowerClient for VirtualRadio<'a, R> {
    fn power_mode_changed(&self, on: bool) {
        if on {
            if let RadioState::StartUp = self.radio_state.get() {
                if self.tx_pending.get() {
                    self.radio_state.set(RadioState::TxPending);
                } else {
                    self.radio_state.set(RadioState::Awake);
                }
            }
        }
    }
}
