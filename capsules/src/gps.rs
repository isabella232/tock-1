use kernel::common::cells::{MapCell, TakeCell};

use kernel::hil;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::GPS as usize;

const PMTK_SET_NMEA_OUTPUT_RMCGGA: &'static [u8; 49] =
    b"$PMTK314,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0*28";
const PMTK_SET_NMEA_UPDATE_1HZ: &'static [u8; 16] = b"$PMTK220,1000*1F";

use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum COMMAND {
    DRIVER_CHECK = 0,
    READLINE = 1,
    WAKE = 2,
    SLEEP = 3,
}
}

const RX_BUF_LEN: usize = 128;
pub static mut TX_BUF: [u8; 64] = [0; 64];
pub static mut RX_BUF: [u8; RX_BUF_LEN] = [0; RX_BUF_LEN];

#[derive(Copy, Clone)]
enum State {
    Init,
    SentOne,
    Ready,
}

#[derive(Default)]
pub struct App {
    rx_slice: Option<AppSlice<Shared, u8>>,
    rx_callback: Option<Callback>,
    rx_read: usize,
    rx_write: usize,
}

pub struct Gps<'a, G: hil::gpio::Pin + hil::gpio::PinCtl> {
    state: MapCell<State>,
    hw: &'a hil::uart::Uart<'a>,
    tx_buffer: TakeCell<'a, [u8]>,
    apps: Grant<App>,
    en_pin: &'a G,
}

impl<'a, G: hil::gpio::Pin + hil::gpio::PinCtl> Gps<'a, G> {
    pub const fn new(uart: &'a hil::uart::Uart<'a>, en_pin: &'a G, apps: Grant<App>) -> Gps<'a, G> {
        Gps {
            state: MapCell::new(State::Init),
            hw: uart,
            tx_buffer: TakeCell::empty(),
            apps,
            en_pin
        }
    }

    pub fn initialize(&self, tx_buf: &'static mut [u8], rx_buf: &'static mut [u8]) {
        self.en_pin.clear();
        for i in 0..::core::cmp::min(PMTK_SET_NMEA_OUTPUT_RMCGGA.len(), tx_buf.len()) {
            tx_buf[i] = PMTK_SET_NMEA_OUTPUT_RMCGGA[i];
        }
        self.hw
            .transmit_buffer(tx_buf, PMTK_SET_NMEA_OUTPUT_RMCGGA.len());
        self.hw.receive_buffer(rx_buf, RX_BUF_LEN);
    }
}

impl<'a, G: hil::gpio::Pin + hil::gpio::PinCtl> hil::uart::TransmitClient for Gps<'a, G> {
    fn transmitted_buffer(&self, tx_buf: &'static mut [u8], _tx_len: usize, _rcode: ReturnCode) {
        let mut send = false;

        self.state.take().map(|mut state| {
            match state {
                State::Init => {
                    state = State::SentOne;
                    send = true;
                }
                State::SentOne => {
                    state = State::Ready;
                }
                State::Ready => (),
            }
            self.state.put(state);
        });

        if send {
            for i in 0..::core::cmp::min(PMTK_SET_NMEA_UPDATE_1HZ.len(), tx_buf.len()) {
                tx_buf[i] = PMTK_SET_NMEA_UPDATE_1HZ[i];
            }
            self.hw
                .transmit_buffer(tx_buf, PMTK_SET_NMEA_UPDATE_1HZ.len());
        } else {
            self.tx_buffer.put(Some(tx_buf));
        }
    }
}

impl<'a, G: hil::gpio::Pin + hil::gpio::PinCtl> hil::uart::ReceiveClient for Gps<'a, G> {
    fn received_buffer(
        &self,
        buffer: &'static mut [u8],
        rx_len: usize,
        _rcode: ReturnCode,
        _error: hil::uart::Error,
    ) {
        for app in self.apps.iter() {
            app.enter(|app, _| {
                let mut offset = app.rx_write;
                if let Some(ref mut slice) = app.rx_slice {
                    if offset + rx_len + 1 > slice.len() {
                        offset = 0;
                    }
                    for i in 0..rx_len {
                        slice.as_mut()[offset + i] = buffer[i];
                    }
                    // application depends on null terminated byte
                    slice.as_mut()[offset + rx_len] = b'\0';

                    app.rx_write = offset + rx_len + 1;
                    app.rx_callback.map(|mut cb| {
                        cb.schedule(From::from(ReturnCode::SUCCESS), offset, 0);
                    });
                }
            });
        }

        self.hw.receive_buffer(buffer, RX_BUF_LEN);
    }
}

impl<'a, G: hil::gpio::Pin + hil::gpio::PinCtl> Driver for Gps<'a, G> {
    fn allow(&self, appid: AppId, arg2: usize, slice: Option<AppSlice<Shared, u8>>) -> ReturnCode {
        let cmd = COMMAND::from_usize(arg2).expect("Invalid command passed by userspace driver");
        match cmd {
            COMMAND::READLINE => self
                .apps
                .enter(appid, |app, _| {
                    app.rx_slice = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
    fn subscribe(&self, arg1: usize, callback: Option<Callback>, appid: AppId) -> ReturnCode {
        let cmd = COMMAND::from_usize(arg1).expect("Invalid command passed by userspace driver");

        match cmd {
            COMMAND::READLINE /* getnstr done */ => {
                self.apps.enter(appid, |app, _| {
                    app.rx_callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            _ => ReturnCode::ENOSUPPORT
        }
    }

    fn command(&self, arg0: usize, read: usize, _: usize, appid: AppId) -> ReturnCode {
        let cmd = COMMAND::from_usize(arg0).expect("Invalid command passed by userspace driver");
        match cmd {
            COMMAND::DRIVER_CHECK /* check if present */ => ReturnCode::SUCCESS,
            COMMAND::READLINE /* get lines */ => {
                self.apps.enter(appid, |app, _| {
                    app.rx_read += read;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            COMMAND::SLEEP /* get lines */ => {
                self.en_pin.set();
                ReturnCode::SUCCESS
            },
            COMMAND::WAKE /* get lines */ => {
                self.en_pin.clear();
                ReturnCode::SUCCESS
            },
        }
    }
}
