use kernel::common::cells::TakeCell;
use core::fmt::{write, Arguments, Result, Write};
use core::str;

use kernel::hil;
use kernel::ReturnCode;

const TX_BUF_LEN: usize = 128;
const RX_BUF_LEN: usize = TX_BUF_LEN;

pub static mut RX_BUF: [u8; RX_BUF_LEN] = [0; RX_BUF_LEN];
// for echoing out received
pub static mut TX_BUF_DEBUG: [u8; RX_BUF_LEN] = [0; RX_BUF_LEN];
pub static mut TX_BUF: [u8; TX_BUF_LEN] = [0; TX_BUF_LEN];

static mut PLT_WRITER: PltWriter = PltWriter {
    tx_buffer: TakeCell::empty(),
};

pub struct DebugClient {
    uart: &'static hil::uart::Uart<'static>,
    tx_buffer: TakeCell<'static, [u8]>,
}



pub struct Plt {
    uart: &'static hil::uart::Uart<'static>,
    pub debug: DebugClient,
    chip_id: u32,
}

pub struct PltWriter {
    tx_buffer: TakeCell<'static, [u8]>,
}

impl Plt {
    pub const fn new(
        plt_uart: &'static hil::uart::Uart<'static>,
        debug_uart: &'static hil::uart::Uart<'static>,
        chip_id: u32,
    ) -> Plt {
        Plt {
            uart: plt_uart,
            debug: DebugClient {
                uart: debug_uart,
                tx_buffer: TakeCell::empty(),
            },
            chip_id,
        }
    }

    //set client must be called before this
    pub fn initialize(
        &self,
        tx_buf: &'static mut [u8],
        rx_buf: &'static mut [u8],
        debug_buf: &'static mut [u8],
    ) {
        unsafe { PLT_WRITER.tx_buffer.put(Some(tx_buf)) };
        self.debug.tx_buffer.put(Some(debug_buf));
        self.uart.receive_buffer(rx_buf, RX_BUF_LEN);
    }
}

impl hil::uart::TransmitClient for Plt {
    fn transmitted_buffer(&self, tx_buf: &'static mut [u8], _tx_len: usize, _rcode: ReturnCode) {
        unsafe { PLT_WRITER.tx_buffer.put(Some(tx_buf)) };
    }
}

impl Write for PltWriter {
    fn write_str(&mut self, s: &str) -> Result {
        Ok(())
    }

}

impl hil::uart::ReceiveClient for Plt {
    fn received_buffer(
        &self,
        buffer: &'static mut [u8],
        rx_len: usize,
        _rcode: ReturnCode,
        _error: hil::uart::Error,
    ) {
        self.debug.tx_buffer.take().map(|buf| {
            let len = ::core::cmp::min(rx_len, buf.len() - 1);

            for i in 0..len {
                buf[i] = buffer[i];
            }
            buf[len] = b'\n';
            self.debug.uart.transmit_buffer(buf, len + 1);
        });

        
            unsafe { write(&mut PLT_WRITER, format_args!(
                 "chip_id={}",
                 self.chip_id));
        };

       
        self.uart.receive_buffer(buffer, RX_BUF_LEN);
    }
}

impl hil::uart::TransmitClient for DebugClient {
    fn transmitted_buffer(&self, tx_buf: &'static mut [u8], _tx_len: usize, _rcode: ReturnCode) {
        self.tx_buffer.put(Some(tx_buf));
    }
}
