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

use enum_primitive::cast::{FromPrimitive, ToPrimitive};
use enum_primitive::enum_from_primitive;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum COMMAND {
    DRIVER_CHECK = 0,
    WRITESTR = 1,
    READLINE = 2,
}
}


#[derive(Default)]
pub struct App {
    tx: AppRequest,
}

pub struct Gps<'a> {
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

        		match &rx.req.buf {
	                ikc::RxBuf::MUT(buf) => {
	                    // for every item in the compeleted_rx
	                    for i in 0..rx.req.items_pushed() {
	                        debug!("{}", buf[i] as char);
	                    }

	                },
	                _ => {},
        		}
                debug!("received line\r\n");
        		rx.reset();
        		self.uart.receive_buffer(rx);
        	}

            if let Some(tx) = tx_complete {
                self.tx_request.put(Some(tx));
                if let Some(app_id) = self.tx_in_progress.take() {
                    self.apps.enter(app_id, |app, _| {
                        // if app tx request has no data left
                        if app.tx.remaining == 0 {
                            // Enqueue the application callback
                            let written = app.tx.len();
                            app.tx.callback.map(|mut cb| {
                                cb.schedule(written, 0, 0);
                            });
                            None
                        } else {
                            // Otherwise, return app_id
                            Some(app_id)
                        }
                    })
                    .unwrap();
                }
            }
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
        tx_request.set_with_mut_ref(tx_buf);
        self.tx_request.put(Some(tx_request));

        rx_request.req.set_buf(rx_buf);
        // TODO: set state?
        self.uart.receive_buffer(rx_request);
    
    }

    pub fn new(uart: &'a hil::uart::UartPeripheral<'a>, grant: Grant<App>) -> Gps<'a> {
        uart.configure(GPS_PARAMS);

        Gps {
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
        debug!("allow");

        let cmd = COMMAND::from_usize(arg2).expect("Invalid command passed by userspace driver");
        match cmd {
            WRITESTR => self.apps
                .enter(appid, |app, _| {
                    app.tx.slice = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            READLINE => self.apps
                .enter(appid, |app, _| {
                    //app.rx.slice = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
    fn subscribe(&self, arg1: usize, callback: Option<Callback>, app_id: AppId) -> ReturnCode {
        debug!("subscribe");

        let cmd = COMMAND::from_usize(arg1).expect("Invalid command passed by userspace driver");
        match cmd {
            WRITESTR /* putstr/write_done */ => {
                self.apps.enter(app_id, |app, _| {
                    app.tx.callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            READLINE /* getnstr done */ => {
                self.apps.enter(app_id, |app, _| {
                    //app.rx.callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            _ => ReturnCode::ENOSUPPORT
        }
    }

    fn command(&self, arg0: usize, len: usize, _: usize, appid: AppId) -> ReturnCode {
        debug!("cmd");

        let cmd = COMMAND::from_usize(arg0).expect("Invalid command passed by userspace driver");
        // let uart_num = (arg0 >> 16) as usize;
        match cmd {
            DRIVER_CHECK /* check if present */ => ReturnCode::SUCCESS,
            WRITESTR /* transmit request */ => {
                //update the request with length
                debug!("writestr cmd");
                if let Err(_err) = self.apps.enter(appid, |app, _| {
                    app.tx.set_len(len);
                    if let Some(request) = self.tx_request.take(){
                        request.reset();
                        request.copy_from_app_request(&mut app.tx);
                        self.uart.transmit_buffer(request);
                        self.tx_in_progress.set(appid);
                    }            
                }){ return ReturnCode::FAIL }
                ReturnCode::SUCCESS
            },
            READLINE /* get lines */ => {
                ReturnCode::SUCCESS
            },
            _ => ReturnCode::ENOSUPPORT
        }
    }
}
