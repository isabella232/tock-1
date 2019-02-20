use kernel::common::cells::{TakeCell, MapCell};


pub struct TestClient<'a> {
    state: MapCell<usize>,
    buffer: [u8; 14],
    tx: TakeCell<'a, hil::uart::TxTransaction<'a>>,
}

impl<'a> TestClient<'a> {
    pub fn new()-> TestClient<'a> {

        TestClient {
            state: MapCell::new(0),
            buffer: [0; 14],
            tx: TakeCell::empty(),
        }
    }
}

use kernel::hil;

impl <'a>hil::uart::Client<'a> for TestClient<'a> {

    fn has_tx_request(&self)-> bool {
        true
    }

    fn get_tx(&self) -> Option<&mut hil::uart::TxTransaction<'a>> {
        //let bull = hil::uart::TxTransaction::new(b"hello world\r\n");
        //&bull
        self.tx.take()
    }

    fn tx_complete(&self, returned_buffer: &hil::uart::TxTransaction<'a>) {


        //increment state counter
        self.state.map(|val| *val += 1);
    }
}

