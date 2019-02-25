use core::cmp;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil;
use kernel::hil::uart::InterruptHandler;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::CONSOLE as usize;

use kernel::ikc::DriverState::{BUSY, IDLE, REQUEST_COMPLETE};
use kernel::ikc::Request::{RX, TX};
use kernel::ikc;

pub fn handle_irq(num: usize, driver: &UartDriver<'a>, clients: &[&'a hil::uart::Client<'a>]) {

    let state = driver.uart[num].handle_interrupt();

    let mut ready_for_tx = false;
    match state.tx {
        // if request complete, return it to client
        REQUEST_COMPLETE(TX(request)) => {
            let client_id = request.client_id;
            clients[client_id].tx_request_complete(num, request);
            ready_for_tx = true;
        }
        IDLE => {
            ready_for_tx = true;
        }
        _ => {}
    }

    let mut ready_for_new_rx = false;
    match state.rx {
        // if request complete, return it to client
        REQUEST_COMPLETE(RX(request)) => {
            // give the transaction to the driver level for muxing out received bytes to other buffers
            let request = driver.uart[num].mux_completed_tx_to_others(request);

            // return the originally completed rx
            let client_id = request.client_id;
            clients[client_id].rx_request_complete(num, request);

            while let Some(next_request) = driver.uart[num].get_other_completed_rx() {
                let client_id = next_request.client_id;
                clients[client_id].rx_request_complete(num, next_request);
            }

            ready_for_new_rx = true;
        },
        IDLE => {
            ready_for_new_rx = true;
        }
        _ => {}
    }

    // // Dispatch new requests only after both TX/RX interrupt have been handled
    // TX'es are dispatched one by one, so only take it if we are ready for another one
    if ready_for_tx {
        dispatch_next_tx_request(num, driver, clients);
    }
    // Each client can have one (and only one) pending RX concurrently with other clients
    // so take any new ones that have occured this go-around
    take_new_tx_requests(num, driver, clients);
    // If a request completed, dispatch the shortest pending request (if there is one)
    if ready_for_new_rx {
        driver.uart[num].dispatch_shortest_tx_request();
    }
}

fn dispatch_next_tx_request<'a>(
    num: usize,
    driver: &UartDriver<'a>,
    clients: &[&'a hil::uart::Client<'a>],
) {
    for index in 0..clients.len() {
        let client = clients[index];
        if client.has_tx_request() {
            if let Some(tx) = client.get_tx_request() {
                tx.client_id = index;
                driver.handle_tx_request(num, tx);
                return;
            }
        }
    }
}

fn take_new_tx_requests<'a>(
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
pub struct App {
    write_callback: Option<Callback>,
    write_buffer: Option<AppSlice<Shared, u8>>,
    write_len: usize,
    write_remaining: usize, // How many bytes didn't fit in the buffer and still need to be printed.
    pending_write: bool,

    read_callback: Option<Callback>,
    read_buffer: Option<AppSlice<Shared, u8>>,
    read_len: usize,
}

pub struct Uart<'a> {
    // uart peripheral that this item is responsible for
    uart: &'a hil::uart::UartPeripheral<'a>,
    // slots of each intrakernel client
    rx_requests: Option<&'a [TakeCell<'a, hil::uart::RxRequest<'a>>]>,
    // app grant providing space fo app clients
    apps: Grant<App>,
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
        grant: Grant<App>) -> Uart<'a> {

        uart.configure(DEFAULT_PARAMS);

        Uart {
            uart,
            rx_requests: rx_requests,
            apps: grant,
        }
    }

    fn handle_interrupt(&self) -> hil::uart::PeripheralState<'a>{
        self.uart.handle_interrupt()
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

        let mut other_request_completed = false;

        if let Some(requests_stash) = self.rx_requests {
            match &completed_rx.buf {
                ikc::RxBuf::MUT(buf) => {
                    // for every item in the compeleted_rx
                    for i in 0..completed_rx.items_pushed(){
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
            }
        }
        // no more completed rx
        None
    }

    fn dispatch_shortest_tx_request(&self) {
        if let Some(requests_stash) = self.rx_requests {

            let mut min: Option<usize> = None;
            let mut index: usize = 0;

            for i in 0..requests_stash.len() {
                if let Some(request) = requests_stash[i].take() {

                    let request_remaining = request.request_remaining();

                    // if there is a minimum already, compare to see if this is shorter
                    if let Some(mut min) = min {
                        if request_remaining <  min {
                            min = request_remaining;
                            index  = i;
                        }
                    }
                    // otherwise, this is the min so far
                    else{
                        min = Some(request_remaining);
                        index  = i;
                    }
                    requests_stash[index].put(Some(request));
                }
            }

            // if there was a request found,dispatch it
            if let Some(_min) = min {
                if let Some(request) = requests_stash[index].take() {
                    self.uart.receive_buffer(request);
                }
            }

        }
        else {
            panic!("UART has not been provisioned with space to store any client requests!")
        }
    }

    /// Internal helper function for setting up a new send transaction
    fn send_new(&self, app_id: AppId, app: &mut App, len: usize) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    /// Internal helper function for continuing a previously set up transaction
    /// Returns true if this send is still active, or false if it has completed
    fn send_continue(&self, app_id: AppId, app: &mut App) -> Result<bool, ReturnCode> {
        Ok(false)
    }

    /// Internal helper function for sending data for an existing transaction.
    /// Cannot fail. If can't send now, it will schedule for sending later.
    fn send(&self, app_id: AppId, app: &mut App, slice: AppSlice<Shared, u8>) {}

    /// Internal helper function for starting a receive operation
    fn receive_new(&self, app_id: AppId, app: &mut App, len: usize) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    fn receive_abort(&self) {
        self.uart.receive_abort();
    }
}

impl Driver for UartDriver<'a> {
    /// Setup shared buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `1`: Writeable buffer for write buffer
    /// - `2`: Writeable buffer for read buffer
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            1 => self.uart[0]
                .apps
                .enter(appid, |app, _| {
                    app.write_buffer = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            2 => self.uart[0]
                .apps
                .enter(appid, |app, _| {
                    app.read_buffer = slice;
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
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            1 /* putstr/write_done */ => {
                self.uart[0].apps.enter(app_id, |app, _| {
                    app.write_callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            2 /* getnstr done */ => {
                self.uart[0].apps.enter(app_id, |app, _| {
                    app.read_callback = callback;
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
    fn command(&self, cmd_num: usize, arg1: usize, _: usize, appid: AppId) -> ReturnCode {
        match cmd_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            1 /* putstr */ => {
                let len = arg1;
                self.uart[0].apps.enter(appid, |app, _| {
                    self.uart[0].send_new(appid, app, len)
                }).unwrap_or_else(|err| err.into())
            },
            2 /* getnstr */ => {
                let len = arg1;
                self.uart[0].apps.enter(appid, |app, _| {
                    self.uart[0].receive_new(appid, app, len)
                }).unwrap_or_else(|err| err.into())
            },
            3 /* abort rx */ => {
                self.uart[0].receive_abort();
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT
        }
    }
}
