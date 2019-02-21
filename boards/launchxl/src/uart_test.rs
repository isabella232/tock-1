use kernel::common::cells::{MapCell, TakeCell};

const msg: &'static [u8; 15] = b"Hello, World!\r\n";


pub struct TestClient<'a> {
    state: MapCell<usize>,
    tx: TakeCell<'a, hil::uart::TxRequest<'a>>,
}

impl<'a> TestClient<'a> {
    pub fn new(msg: &'a mut hil::uart::TxRequest<'a>)-> TestClient<'a> {
        TestClient {
            state: MapCell::new(0),
            tx: TakeCell::new(msg),
        }

    }
}

use kernel::hil;

impl <'a>hil::uart::Client<'a> for TestClient<'a> {

    fn has_tx_request(&self)-> bool {
        self.tx.is_some()
    }

    fn get_tx_request(&self) -> Option<&mut hil::uart::TxRequest<'a>> {
        self.tx.take()
    }

    fn tx_request_complete(&self, returned_buffer: &'a mut hil::uart::TxRequest<'a>) {
        returned_buffer.index = 0;
        self.tx.put(Some(returned_buffer));
    }
}

