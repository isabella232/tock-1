
enum State {
    UNINITIALIZED,
}

pub struct Client {
    state: State,
}

impl Client{
    pub fn new()-> Client{
        Client { state: State::UNINITIALIZED }
    }
}

use kernel::hil;

impl <'a>hil::uart::Client<'a> for Client{

    fn has_tx_request(&self)-> bool {
        true
    }
}

