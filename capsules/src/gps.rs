use kernel::common::cells::{MapCell, OptionalCell, TakeCell};

use kernel::ikc;
use kernel::hil;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::GPS as usize;

pub type AppRequest = ikc::AppRequest<u8>;

static GPS_PARAMS: hil::uart::Parameters = hil::uart::Parameters {
    baud_rate: 9600, // baud rate in bit/s
    width: hil::uart::Width::Eight,
    parity: hil::uart::Parity::None,
    stop_bits: hil::uart::StopBits::One,
    hw_flow_control: false,
};

const PMTK_SET_NMEA_OUTPUT_RMCGGA: &'static [u8; 49] = b"$PMTK314,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0*28";
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


#[derive(Default)]
pub struct App {
    tx: AppRequest,
    rx_slice: Option<AppSlice<Shared, u8>>,
    rx_callback: Option<Callback>,
    read: usize,
    write: usize,
}

enum State {
    Init,
    ReceivedOne,
    SentOne,
    Ready,
}

pub struct Gps<'a> {
    state: MapCell<State>,
    uart: &'a hil::uart::UartPeripheral<'a>,
    uart_state: MapCell<hil::uart::PeripheralState>,
    rx_request: TakeCell<'a, hil::uart::RxRequest<'a>>,
    tx_request: TakeCell<'a, hil::uart::TxRequest<'a>>,
    tx_in_progress: OptionalCell<AppId>,
    // app grant providing space fo app clients
    apps: Grant<App>,
}

impl<'a> Gps<'a> {
    pub fn space() -> (
        [u8; 64],
        [u8; 64],
        hil::uart::RxRequest<'a>,
        hil::uart::TxRequest<'a>,
    ) {
        (
            [0; 64],
            [0; 64],
            hil::uart::RxRequest::new(),
            hil::uart::TxRequest::new(),
        )
    }

    pub fn handle_irq(&self){
    	 self.uart_state.map(|state| {
    	 	// pass a copy of state to the HIL's handle interrupt routine
	        // it will return completed requests if there are any
        	let (tx_complete, rx_complete) = self.uart.handle_interrupt(*state);

        	if let Some(rx) = rx_complete {
                // self.state.take().map(|mut state| {
                //     match state {
                //         State::Init => state = State::ReceivedOne,
                //         State::ReceivedOne => {
                //             state = State::SentOne;
                //             self.tx_request.take().map(|tx| 
                //             {
                //                 tx.set_with_const_ref(PMTK_SET_NMEA_OUTPUT_RMCGGA);
                //                 self.uart.transmit_buffer(tx);
                //             });
                            
                //         }
                //         _ => (),
                //     }
                //     self.state.put(state);
                // });

        		match &rx.req.buf {

	                ikc::RxBuf::MUT(buf) => {
                        //for every item in the compeleted_rx
                        for i in 0..rx.req.items_pushed() {
                            debug!("{}", buf[i] as char);
                        }

                        // for app in self.apps.iter() {

                        //     app.enter(|app, _| {
                        //         let mut offset = app.write;

                        //         if let Some(ref mut slice) = app.rx_slice {

                        //             if offset + rx.req.items_pushed() > slice.len() {
                        //                 offset = 0;
                        //             }

                        //             for i in 0..rx.req.items_pushed() {
                        //                     slice.as_mut()[offset + i] = buf[i];
                        //             }

                        //             app.write = offset + rx.req.items_pushed();
     

                        //             app.rx_callback.map(|mut cb| {
                        //                 cb.schedule(From::from(ReturnCode::SUCCESS), offset, 0);
                        //             });

                        //         }
                                
                        //     });

                        // }               

	                },
	                _ => (),
        		}

        		rx.reset();
        		self.uart.receive_buffer(rx);
        	}

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
    	 });        
    }

    pub fn set_with_default_space(&self,
        space: &'a mut (
            [u8; 64],
            [u8; 64],
            hil::uart::RxRequest<'a>,
            hil::uart::TxRequest<'a>,
        ),
    ) {
        let (buf0, buf1, rx_request, tx_request) = space;
        self.set_space(buf0, buf1, rx_request, tx_request)
    }

    pub fn set_space(&self, 
    	rx_buf: &'a mut [u8],
		tx_buf: &'a mut [u8],
        rx_request: &'a mut hil::uart::RxRequest<'a>,
        tx_request: &'a mut hil::uart::TxRequest<'a>,
    ) {
        self.tx_request.put(Some(tx_request));


        rx_request.req.set_buf(rx_buf);
        // TODO: set state?
        self.uart.receive_buffer(rx_request);

    }

    pub fn new(uart: &'a hil::uart::UartPeripheral<'a>, grant: Grant<App>) -> Gps<'a> {
        uart.configure(GPS_PARAMS);

        Gps {
            state: MapCell::new(State::Init),
            rx_request: TakeCell::empty(),
            tx_request: TakeCell::empty(),
            tx_in_progress: OptionalCell::empty(),
            uart,
            uart_state: MapCell::new(hil::uart::PeripheralState::new()),
            apps: grant,
        }

    }
}

impl Driver for Gps<'a> {
    fn allow(&self, appid: AppId, arg2: usize, slice: Option<AppSlice<Shared, u8>>) -> ReturnCode {
        let cmd = COMMAND::from_usize(arg2).expect("Invalid command passed by userspace driver");
        match cmd {
            COMMAND::READLINE => self.apps
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
                    app.read += read;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            _ => ReturnCode::ENOSUPPORT
        }
    }
}
