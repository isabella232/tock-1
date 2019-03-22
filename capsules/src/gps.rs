use kernel::common::cells::{MapCell, OptionalCell, TakeCell};

use kernel::hil;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::GPS as usize;

static GPS_PARAMS: hil::uart::Parameters = hil::uart::Parameters {
    baud_rate: 9600, // baud rate in bit/s
    width: hil::uart::Width::Eight,
    parity: hil::uart::Parity::None,
    stop_bits: hil::uart::StopBits::One,
    hw_flow_control: false,
};

const PMTK_SET_NMEA_OUTPUT_RMCGGA: &'static [u8; 49] =
    b"$PMTK314,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0*28";
const PMTK_SET_NMEA_UPDATE_1HZ: &'static [u8; 16] = b"$PMTK220,1000*1F";

use enum_primitive::cast::{FromPrimitive, ToPrimitive};
use enum_primitive::enum_from_primitive;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum COMMAND {
    DRIVER_CHECK = 0,
    READLINE = 1,
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

pub struct Gps<'a> {
    state: MapCell<State>,
    hw: &'a hil::uart::Uart<'a>,
    tx_in_progress: OptionalCell<AppId>,
    tx_buffer: TakeCell<'a, [u8]>,
    rx_in_progress: OptionalCell<AppId>,
    rx_buffer: TakeCell<'a, [u8]>,
    apps: Grant<App>,
}

impl<'a> Gps<'a> {
    pub const fn new(uart: &'a hil::uart::Uart<'a>, apps: Grant<App>) -> Gps<'a> {
        Gps {
            state: MapCell::new(State::Init),
            hw: uart,
            tx_in_progress: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            rx_in_progress: OptionalCell::empty(),
            rx_buffer: TakeCell::empty(),
            apps,
        }
    }

    pub fn initialize(&self, mut tx_buf: &'static mut [u8], rx_buf: &'static mut [u8]) {
        for i in 0..::core::cmp::min(PMTK_SET_NMEA_OUTPUT_RMCGGA.len(), tx_buf.len()) {
            tx_buf[i] = PMTK_SET_NMEA_OUTPUT_RMCGGA[i];
        }
        self.hw
            .transmit_buffer(tx_buf, PMTK_SET_NMEA_OUTPUT_RMCGGA.len());
        self.hw.receive_buffer(rx_buf, RX_BUF_LEN);
    }
}

impl<'a> hil::uart::TransmitClient for Gps<'a> {
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

impl<'a> hil::uart::ReceiveClient for Gps<'a> {
    fn received_buffer(
        &self,
        buffer: &'static mut [u8],
        rx_len: usize,
        _rcode: ReturnCode,
        error: hil::uart::Error,
    ) {
        for app in self.apps.iter() {
            app.enter(|app, _| {
                let mut offset = app.rx_write;

                if let Some(ref mut slice) = app.rx_slice {
                    if offset + rx_len > slice.len() {
                        offset = 0;
                    }

                    for i in 0..rx_len {
                        slice.as_mut()[offset + i] = buffer[i];
                    }

                    app.rx_write = offset + rx_len;

                    app.rx_callback.map(|mut cb| {
                        cb.schedule(From::from(ReturnCode::SUCCESS), offset, 0);
                    });
                }
            });
        }

        self.hw.receive_buffer(buffer, RX_BUF_LEN);
    }
}
// pub fn handle_irq(&self){
// 	 self.uart_state.map(|state| {
// 	 	// pass a copy of state to the HIL's handle interrupt routine
//      // it will return completed requests if there are any
//     	let (tx_complete, rx_complete) = self.uart.handle_interrupt(*state);

//     	if let Some(rx) = rx_complete {
//             // self.state.take().map(|mut state| {
//             //     match state {
//             //         State::Init => state = State::ReceivedOne,
//             //         State::ReceivedOne => {
//             //             state = State::SentOne;
//             //             self.tx_request.take().map(|tx|
//             //             {
//             //                 tx.set_with_const_ref(PMTK_SET_NMEA_OUTPUT_RMCGGA);
//             //                 self.uart.transmit_buffer(tx);
//             //             });

//             //         }
//             //         _ => (),
//             //     }
//             //     self.state.put(state);
//             // });

//     		match &rx.req.buf {

//              ikc::RxBuf::MUT(buf) => {
//                     //for every item in the compeleted_rx
//                     for i in 0..rx.req.items_pushed() {
//                         debug!("{}", buf[i] as char);
//                     }

//                     // for app in self.apps.iter() {

//                     //     app.enter(|app, _| {
//                     //         let mut offset = app.write;

//                     //         if let Some(ref mut slice) = app.rx_slice {

//                     //             if offset + rx.req.items_pushed() > slice.len() {
//                     //                 offset = 0;
//                     //             }

//                     //             for i in 0..rx.req.items_pushed() {
//                     //                     slice.as_mut()[offset + i] = buf[i];
//                     //             }

//                     //             app.write = offset + rx.req.items_pushed();

//                     //             app.rx_callback.map(|mut cb| {
//                     //                 cb.schedule(From::from(ReturnCode::SUCCESS), offset, 0);
//                     //             });

//                     //         }

//                     //     });

//                     // }

//              },
//              _ => (),
//     		}

//     		rx.reset();
//     		self.uart.receive_buffer(rx);
//     	}

// if let Some(tx) = tx_complete {
//     self.state.take().map(|mut state| {
//         match state {
//             State::SentOne => {
//                 tx.set_with_const_ref(PMTK_SET_NMEA_UPDATE_1HZ);
//                 state = State::Ready;
//             },
//             _=> (),
//         }
//         self.state.put(state);
//     });
//     if tx.has_some() {
//         self.uart.transmit_buffer(tx);
//     }
//     else{
//         self.tx_request.put(Some(tx));
//     }
// }
// 	 });
// }

//   pub fn set_with_default_space(&self,
//       space: &'a mut (
//           [u8; 64],
//           [u8; 64],
//           hil::uart::RxRequest<'a>,
//           hil::uart::TxRequest<'a>,
//       ),
//   ) {
//       let (buf0, buf1, rx_request, tx_request) = space;
//       self.set_space(buf0, buf1, rx_request, tx_request)
//   }

//   pub fn set_space(&self,
//   	rx_buf: &'a mut [u8],
// tx_buf: &'a mut [u8],
//       rx_request: &'a mut hil::uart::RxRequest<'a>,
//       tx_request: &'a mut hil::uart::TxRequest<'a>,
//   ) {
//       self.tx_request.put(Some(tx_request));

//       rx_request.req.set_buf(rx_buf);
//       // TODO: set state?
//       self.uart.receive_buffer(rx_request);

//   }

impl Driver for Gps<'a> {
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
        //debug!("subscribe: {:?}\r\n", cmd);

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
            _ => ReturnCode::ENOSUPPORT
        }
    }
}
