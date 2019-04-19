use kernel::common::cells::{MapCell, TakeCell};

use kernel::hil;
use kernel::ReturnCode;

const TX_BUF_LEN: usize = 128;
const RX_BUF_LEN: usize = TX_BUF_LEN;

pub static mut RX_BUF: [u8; RX_BUF_LEN] = [0; RX_BUF_LEN];
// for echoing out received
pub static mut TX_BUF_DEBUG: [u8; RX_BUF_LEN] = [0; RX_BUF_LEN];
pub static mut TX_BUF: [u8; TX_BUF_LEN] = [0; TX_BUF_LEN];

pub struct Plt<'a> {
    plt: &'a hil::uart::Uart<'a>,
    debug: &'a hil::uart::Uart<'a>,
    tx_buffer: TakeCell<'a, [u8]>,
    debug_buffer: TakeCell<'a, [u8]>,
}

impl<'a> Plt<'a> {

    pub const fn new(plt_uart: &'a (dyn kernel::hil::uart::Uart<'a> + 'a), debug_uart: &'a (dyn kernel::hil::uart::Uart<'a> + 'a)) -> Plt <'a> {
        Plt {
            plt: plt_uart,
            debug: debug_uart,
            tx_buffer: TakeCell::empty(),
            debug_buffer: TakeCell::empty(),
        }
    }

    pub fn initialize(&self, tx_buf: &'static mut [u8], rx_buf: &'static mut [u8], debug_buf: &'static mut [u8]) {
        self.tx_buffer.put(Some(tx_buf));
        self.debug_buffer.put(Some(debug_buf));
        self.plt.receive_buffer(rx_buf, RX_BUF_LEN);
    }
}

impl<'a> hil::uart::TransmitClient for Plt<'a> {
    fn transmitted_buffer(&self, tx_buf: &'static mut [u8], _tx_len: usize, _rcode: ReturnCode) {

    }
}

impl<'a> hil::uart::ReceiveClient for Plt<'a> {
    fn received_buffer(
        &self,
        buffer: &'static mut [u8],
        rx_len: usize,
        _rcode: ReturnCode,
        _error: hil::uart::Error,
    ) {

        self.plt.receive_buffer(buffer, RX_BUF_LEN);
    }
}
