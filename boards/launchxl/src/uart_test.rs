use kernel::common::cells::{MapCell, TakeCell};

const MSG1: &'static [u8; 15] = b"Hello, World!\r\n";
const MSG2: &'static [u8; 22] = b"You can start typing\r\n";

enum State {
    FIRST_MSG,
    SECOND_MSG,
    ECHO
}

pub struct TestClient<'a> {
    state: MapCell<State>,
    tx: TakeCell<'a, hil::uart::TxRequest<'a>>,
}

impl<'a> TestClient<'a> {

    pub fn space() -> ([u8; 1], hil::uart::TxRequest<'a>, [u8; 1], hil::uart::TxRequest<'a>) {
        ( [0], hil::uart::TxRequest::new(), [0], hil::uart::TxRequest::new())
    }

    pub fn new(space: &'a mut ([u8; 1], kernel::ikc::TxRequest<'a, u8>, [u8; 1], kernel::ikc::TxRequest<'a, u8>))-> TestClient<'a> {
       
        let (tx_request_buffer, tx_request, rx_request_buffer, rx_request) = space;

        tx_request.set_with_const_ref(MSG1);

        TestClient {
           state: MapCell::new(State::FIRST_MSG),
           tx: TakeCell::new(tx_request),
        }
    }
}

use kernel::hil;

impl <'a>hil::uart::Client<'a> for TestClient<'a> {

    fn has_tx_request(&self)-> bool {
        let mut ret = false;
        self.tx.take().map( |tx| { 
            ret = tx.has_some();
            self.tx.put(Some(tx));
        });
        ret
    }

    fn get_tx_request(&self) -> Option<&mut hil::uart::TxRequest<'a>> {
        self.tx.take()
    }

    fn tx_request_complete(&self, returned_request: &'a mut hil::uart::TxRequest<'a>) {

        self.state.take().map( |mut state| {
            match state {
                State::FIRST_MSG => {
                    returned_request.set_with_const_ref(MSG2);
                    state = State::SECOND_MSG;
                },
                State::SECOND_MSG => {
                    state = State::ECHO;
                },
                State::ECHO => {},
            }
            self.state.put(state);
        });

        //returned_buffer.set(kernel::ikc::TxItems::CONST(Some(msg2)));
        self.tx.put(Some(returned_request));
    }
}

