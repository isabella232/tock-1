use kernel::common::cells::{OptionalCell, TakeCell, MapCell};
use kernel::debug;

use kernel::hil;
use kernel::hil::uart::InterruptHandler;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::CONSOLE as usize;

use kernel::ikc::DriverState::{BUSY, IDLE};
use kernel::ikc::Request::{RX, TX};
use kernel::ikc;

pub fn handle_irq(num: usize, driver: &UartDriver<'a>, clients: &[&'a hil::uart::Client<'a>]) {
    driver.uart[num].state.map( |state| {
        // pass a copy of state to the HIL's handle interrupt routine
        let (tx_complete, rx_complete) = driver.uart[num].handle_interrupt(*state);

        // if we have receive a completed transmit, then we need to handle it
        if let Some(request) = tx_complete {
            let client_id = request.client_id;
            clients[client_id].tx_request_complete(num, request);
            state.tx = IDLE;
        }

        // if we have receive a completed receive, then we need to handle it
        if let Some(request) = rx_complete {

            // give the transaction to the driver level for muxing out received bytes to other buffers
            let request = driver.uart[num].mux_completed_tx_to_others(request);


            // return the originally completed rx
            let client_id = request.client_id;

            clients[client_id].rx_request_complete(num, request);

            // if the muxing out completed any other rx'es, return them as well
            while let Some(next_request) = driver.uart[num].get_other_completed_rx() {
                let client_id = next_request.client_id;
                clients[client_id].rx_request_complete(num, next_request);
            }
            state.rx = IDLE;
        }

        // // Dispatch new requests only after both TX/RX completed have been handled
        // TX'es are dispatched one by one, so only take it if we are ready for another one
        if state.tx == IDLE {
            if dispatch_next_tx_request(num, driver, clients){
                state.tx = BUSY;
            }
        }
        // Each client can have one (and only one) pending RX concurrently with other clients
        // so take any new ones that have occured this go-around
        take_new_rx_requests(num, driver, clients);

        // If a request completed, dispatch the shortest pending request (if there is one)
        if state.rx == IDLE {
            if driver.uart[num].dispatch_shortest_rx_request(){
                state.rx = BUSY;
            }
        }
    });

}

fn dispatch_next_tx_request<'a>(
    num: usize,
    driver: &UartDriver<'a>,
    clients: &[&'a hil::uart::Client<'a>],
) -> bool {
    for index in 0..clients.len() {
        let client = clients[index];
        if client.has_tx_request() {
            if let Some(tx) = client.get_tx_request() {
                tx.client_id = index;
                driver.handle_tx_request(num, tx);
                return true;
            }
        }
    }
    false
}

fn take_new_rx_requests<'a>(
    num: usize,
    driver: &UartDriver<'a>,
    clients: &[&'a hil::uart::Client<'a>],
) {
    for index in 0..clients.len() {
        let client = clients[index];
        if client.has_rx_request() {
            if let Some(rx) = client.get_rx_request() {
                rx.client_id = index;
                driver.uart[num].stash_rx_request(rx);
            }
        }
    }
}

#[derive(Default)]
pub struct App<'a> {
    tx_request: hil::uart::TxRequest<'a>,
    tx_callback: Option<Callback>,
    rx_request: hil::uart::RxRequest<'a>,
    rx_callback: Option<Callback>,
}

pub struct Uart<'a> {
    // uart peripheral that this item is responsible for
    uart: &'a hil::uart::UartPeripheral<'a>,
    state: MapCell<hil::uart::PeripheralState>,
    // slots of each intrakernel client
    rx_requests: Option<&'a [TakeCell<'a, hil::uart::RxRequest<'a>>]>,
    app_tx_in_progress: OptionalCell<AppId>,
    app_rx_in_progress: OptionalCell<AppId>,
    // space for copying requests from Apps before dispatching to UART HIL
    app_requests: AppRequests<'a>,
    // app grant providing space fo app clients
    apps: Grant<App<'a>>,
}

pub struct AppRequests<'a> {
    tx: MapCell<&'a mut hil::uart::TxRequest<'a>>,
    rx: MapCell<&'a mut hil::uart::RxRequest<'a>>,
}

impl<'a> AppRequests<'a> {
    pub fn space() -> (
        [u8; 1024],
        hil::uart::TxRequest<'a>,
        [u8; 1024],
        hil::uart::RxRequest<'a>,
    ) {
        (
            [0; 1024],
            hil::uart::TxRequest::new(),
            [0; 1024],
            hil::uart::RxRequest::new(),
        )
    }

    pub fn new_with_default_space(
        space: &'a mut (
            [u8; 1024],
            hil::uart::TxRequest<'a>,
            [u8; 1024],
            hil::uart::RxRequest<'a>,
        ),
    ) -> AppRequests<'a> {
        let (tx_request_buffer, tx_request, rx_request_buffer, rx_request) = space;

        Self::new(tx_request_buffer, tx_request, rx_request_buffer, rx_request)
    }

    pub fn new(
        tx_request_buffer: &'a mut [u8],
        tx_request: &'a mut kernel::ikc::TxRequest<'a, u8>,
        rx_request_buffer: &'a mut [u8],
        rx_request: &'a mut kernel::ikc::RxRequest<'a, u8>,
    ) -> AppRequests<'a> {
        tx_request.set_with_mut_ref(tx_request_buffer);
        rx_request.set_buf(rx_request_buffer);

        AppRequests {
            tx: MapCell::new(tx_request),
            rx: MapCell::new(rx_request),
        }
    }
}

pub struct UartDriver<'a> {
    pub uart: &'a [&'a Uart<'a>],
}

impl<'a> UartDriver<'a> {
    pub fn new(uarts: &'a [&'a Uart<'a>]) -> UartDriver<'a> {
        UartDriver { uart: uarts }
    }

    pub fn handle_tx_request(&self, uart_num: usize, tx: &'a mut hil::uart::TxRequest<'a>) {
        self.uart[uart_num].uart.transmit_buffer(tx);
    }

    fn transmit_app_request(&self, uart_num: usize, app_id: AppId) -> ReturnCode {
        if let Some(request) = self.uart[uart_num].app_requests.tx.take(){

            //TODO: handle error from apps.enter
            self.uart[uart_num].apps.enter(app_id, |app, _| {
                request.copy_from_app_slice(&mut app.tx_request);
            });
            self.uart[uart_num].app_tx_in_progress.set(app_id);
            self.uart[uart_num].uart.transmit_buffer(request)
        }
        else{
            panic!("Should not invoke transmit_app_request if there is no app request tx buffer available!");
            ReturnCode::ENOSUPPORT
        }
    }

}

static DEFAULT_PARAMS: hil::uart::Parameters = hil::uart::Parameters {
    baud_rate: 115200, // baud rate in bit/s
    width: hil::uart::Width::Eight,
    parity: hil::uart::Parity::None,
    stop_bits: hil::uart::StopBits::One,
    hw_flow_control: false,
};

impl<'a> Uart<'a> {
    pub fn new(
        uart: &'a hil::uart::UartPeripheral<'a>,
        rx_requests: Option<&'a [TakeCell<'a, hil::uart::RxRequest<'a>>]>,
        app_requests: AppRequests<'a>,
        grant: Grant<App<'a>>) -> Uart<'a> {

        uart.configure(DEFAULT_PARAMS);

        Uart {
            uart,
            state: MapCell::new(hil::uart::PeripheralState::new()),
            rx_requests,
            app_tx_in_progress: OptionalCell::empty(),
            app_rx_in_progress: OptionalCell::empty(),
            app_requests,
            apps: grant,
        }
    }

    fn handle_interrupt(&self, state: hil::uart::PeripheralState) 
        -> (Option<&mut hil::uart::TxRequest<'a>>, Option<&mut hil::uart::RxRequest<'a>>) {
        self.uart.handle_interrupt(state)
    }

    fn stash_rx_request(&self, rx: &'a mut hil::uart::RxRequest<'a>){
        let index = rx.client_id;
        if let Some(requests_stash) = self.rx_requests {
            if let Some(existing_request) = requests_stash[index].take() {
                panic!("Client #{} should not be making new request when request is already pending!", index)
            }
            else {
                requests_stash[index].put(Some(rx));
            }
        }
        else {
            panic!("UART has not been provisioned with space to store any client requests!")
        }
    }

    fn mux_completed_tx_to_others(&self, completed_rx: &'a mut hil::uart::RxRequest<'a>) -> &'a mut hil::uart::RxRequest<'a> {

        if let Some(requests_stash) = self.rx_requests {
            match &completed_rx.buf {
                ikc::RxBuf::MUT(buf) => {
                    // for every item in the compeleted_rx
                    for i in 0..completed_rx.items_pushed() {
                        let item = buf[i];
                        // copy it into any existing requests in the requests_stash
                        for j in 0..requests_stash.len() {
                            if let Some(request) = requests_stash[j].take() {
                                if request.has_room(){
                                    request.push(item);
                                }
                                requests_stash[j].put(Some(request));
                            }
                        }
                    }
                },
                _ => panic!("A null buffer has become a completed request? It should have never been dispatched in the first place! Shame on console/uart.rs"),
            }
        } 
        else {
            panic!("UART has not been provisioned with space to store any client requests!")
        }
        completed_rx
    }

    fn get_other_completed_rx(&self) -> Option<&'a mut hil::uart::RxRequest<'a>> {
        if let Some(requests_stash) = self.rx_requests {
            for i in 0..requests_stash.len() {
                if let Some(request) = requests_stash[i].take() {
                    if request.request_completed(){
                        return Some(request)
                    }
                    else{
                        requests_stash[i].put(Some(request));
                    }
                }
                else {

                }
            }
        }
        // no more completed rx
        None
    }

    fn dispatch_shortest_rx_request(&self) -> bool {
        if let Some(requests_stash) = self.rx_requests {

            let mut min: Option<usize> = None;
            let mut min_index: usize = 0;

            for i in 0..requests_stash.len() {
                if let Some(request) = requests_stash[i].take() {
                    let request_remaining = request.request_remaining();

                    // if there is a minimum already, compare to see if this is shorter
                    if let Some(mut min) = min {
                        if request_remaining <  min {
                            min = request_remaining;
                            min_index  = i;
                        }
                    }
                    // otherwise, this is the min so far
                    else{
                        min = Some(request_remaining);
                        min_index  = i;
                    }
                    requests_stash[i].put(Some(request));
                }
            }

            // if there was a request found,dispatch it
            if let Some(_min) = min {
                if let Some(request) = requests_stash[min_index].take() {
                    self.uart.receive_buffer(request);
                    return true;
                }
            }

        }
        else {
            panic!("UART has not been provisioned with space to store any client requests!")
        }

        false
    }

    /// Internal helper function for starting a receive operation
    fn receive(&self, app_id: AppId, app: &mut App, len: usize) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }
}

impl Driver for UartDriver<'a> {
    /// Setup shared buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `1`: Writeable buffer for write buffer
    /// - `2`: Writeable buffer for read buffer
    fn allow(&self, appid: AppId, arg2: usize, slice: Option<AppSlice<Shared, u8>>) -> ReturnCode {
        let allow_num = arg2 as u16;
        let uart_num =  (arg2 >> 16) as usize;
        match allow_num {
            1 => self.uart[uart_num]
                .apps
                .enter(appid, |app, _| {
                    if let Some(buf) = slice {
                        app.tx_request.set_with_app_slice(buf);
                    }
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            2 => self.uart[uart_num]
                .apps
                .enter(appid, |app, _| {
                    if let Some(buf) = slice {
                        //app.rx_request.set_with_mut_ref(buf);
                    }
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Setup callbacks.
    ///
    /// ### `subscribe_num`
    ///
    /// - `1`: Write buffer completed callback
    fn subscribe(&self, arg1: usize, callback: Option<Callback>, app_id: AppId) -> ReturnCode {
        let subscribe_num = arg1 as u16;
        let uart_num =  (arg1 >> 16) as usize;
        match subscribe_num {
            1 /* putstr/write_done */ => {
                self.uart[uart_num].apps.enter(app_id, |app, _| {
                    app.tx_callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            2 /* getnstr done */ => {
                self.uart[uart_num].apps.enter(app_id, |app, _| {
                    app.rx_callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            _ => ReturnCode::ENOSUPPORT
        }
    }

    /// Initiate serial transfers
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Transmits a buffer passed via `allow`, up to the length
    ///        passed in `arg1`
    /// - `2`: Receives into a buffer passed via `allow`, up to the length
    ///        passed in `arg1`
    /// - `3`: Cancel any in progress receives and return (via callback)
    ///        what has been received so far.
    fn command(&self, arg0: usize, arg1: usize, _: usize, appid: AppId) -> ReturnCode {
        let cmd_num = arg0 as u16;
        let uart_num =  (arg0 >> 16) as usize;
        match cmd_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            1 /* transmit request */ => { 

                // update the request with length
                self.uart[uart_num].apps.enter(appid, |app, _| {
                    let len = arg1;
                    app.tx_request.set_request_len(len);
                });

                self.uart[uart_num].state.map_or(ReturnCode::ENOSUPPORT, 
                    |state| {
                    if state.tx == IDLE {
                            self.transmit_app_request(uart_num, appid)
                        }
                        else {
                            ReturnCode::SUCCESS
                        }
                    }
                )

            },
            2 /* getnstr */ => { ReturnCode::SUCCESS
                // let len = arg1;
                // self.uart[uart_num].apps.enter(appid, |app, _| {
                //     ReturnCode::SUCCESS
                //     //self.uart[uart_num].receive(appid, app, len)
                // }).unwrap_or_else(|err| err.into())
            },
            3 /* abort rx */ => {
                self.uart[uart_num].uart.receive_abort();
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT
        }
    }
}
