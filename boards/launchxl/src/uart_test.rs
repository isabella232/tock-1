use kernel::common::cells::{MapCell, TakeCell};

const msg1: &'static [u8; 15] = b"Hello, World!\r\n";
const msg2: &'static [u8; 15] = b"Hello, World!\r\n";


pub struct TestClient<'a> {
    state: MapCell<usize>,
    tx: TakeCell<'a, hil::uart::TxRequest<'a>>,
}

impl<'a> TestClient<'a> {
    pub fn new(space: &'a mut hil::uart::TxRequest<'a>)-> TestClient<'a> {
        space.set(kernel::ikc::TxItems::CONST(Some(msg1)));
        TestClient {
            state: MapCell::new(0),
            tx: TakeCell::new(space),
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
        returned_buffer.set(kernel::ikc::TxItems::CONST(Some(msg2)));
        self.tx.put(Some(returned_buffer));
    }
}

