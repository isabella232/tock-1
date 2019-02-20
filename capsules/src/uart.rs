use core::cmp;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::CONSOLE as usize;

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

pub struct Uart<'a>{
    uart: &'a hil::uart::UartPeripheral<'a>,
    apps: Grant<App>,
    state: hil::uart::PeripheralState<'a>,
    current_tx_client: Option<usize>,
    current_rx_client: Option<usize>
}

pub struct UartDriver<'a>{
    pub uart: &'a [&'a Uart<'a>]
}

impl<'a> UartDriver<'a> {
    pub fn new(
        uarts: &'a [&'a Uart<'a>]
    ) -> UartDriver<'a> {
        UartDriver { uart: uarts}
    }

    pub fn with_peripheral<F>(&self, uart_num: usize, f: F) -> &[&'a hil::uart::Client<'a>]
    where
        F: FnOnce(&'a Uart<'a>)-> &[&'a hil::uart::Client<'a>]
    {
        f(self.uart[uart_num])
        //match driver_num {
        //    capsules::uart::DRIVER_NUM => f(Some(self.uart_driver)),
        //    _ => f(None),
        //}
    }

    // pub fn handle_interrupt(&self, uart_num: usize, clients: &'a[&'a hil::uart::Client<'a>]) -> &'a[&'a hil::uart::Client<'a>]{
    //     self.uart[uart_num].handle_interrupt(clients)
    // }
}


static DEFAULT_PARAMS: hil::uart::Parameters  = hil::uart::Parameters {
    baud_rate: 115200, // baud rate in bit/s
    width: hil::uart::Width::Eight,
    parity: hil::uart::Parity::None,
    stop_bits: hil::uart::StopBits::One,
    hw_flow_control: false,
};

impl<'a, 'b> Uart<'a> {
    pub fn new(
        uart: &'a hil::uart::UartPeripheral<'a>,
        grant: Grant<App>,
    ) -> Uart<'a> {
        
        uart.configure(DEFAULT_PARAMS);

        Uart {
            uart: uart,
            apps: grant,
            state: hil::uart::PeripheralState::new(),
            current_tx_client: Some(0),
            current_rx_client: None
        }
    }

    // used just to trigger this thing (delete later)
    pub fn write_buffer(&self, tx: &'a mut hil::uart::TxTransaction<'a>) {
       self.uart.transmit_buffer(tx);
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
    fn send(&self, app_id: AppId, app: &mut App, slice: AppSlice<Shared, u8>) {

    }

    /// Internal helper function for starting a receive operation
    fn receive_new(&self, app_id: AppId, app: &mut App, len: usize) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    fn receive_abort(&self) {
        self.uart.receive_abort();
    }

    pub fn handle_interrupt(&self, clients: &'b[&'a hil::uart::Client<'a>]) -> &'b[&'a hil::uart::Client<'a>] {
        // dispatch the interrupt event to the HIL implementation
        let status = self.uart.handle_interrupt();

        // handle Tx complete status
        if let  hil::uart::State::COMPLETE  = status.tx_state {
            
            if let Some(tx) = status.tx_ret {

                if let Some(client_index) = self.current_tx_client {

                    clients[client_index].tx_complete(tx);

                } else{
                    //panic!("HIL indicated complete transaction and returned buffer, but no client index. UART Driver cleared index or forgot to set!")
                }
            } else {
                panic!("HIL Implementation indicated complete status, but no buffer returned!")
            }
        }
        for client in clients {
            if client.has_tx_request(){
                if let Some(tx) = client.get_tx() {
                    self.uart.transmit_buffer(tx);
                }
                
            }
        }
        clients
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
