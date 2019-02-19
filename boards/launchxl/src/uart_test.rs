
const statement: &[u8] = b"hello world\r\n";

const transaction: hil::uart::TxTransaction = hil::uart::TxTransaction {
        length: statement.len(),
        buffer: statement,
        index: 0
};

pub struct Client {
    state: usize,

}

impl Client{
    pub fn new()-> Client{
        Client {
            state: 0
        }
    }
}

use kernel::hil;

impl <'a>hil::uart::Client<'a> for Client{

    fn has_tx_request(&self)-> bool {
        true
    }

    fn get_tx(&self) -> &hil::uart::TxTransaction<'a>{
        //let bull = hil::uart::TxTransaction::new(b"hello world\r\n");
        //&bull
        &transaction
    }

    fn tx_complete(&mut self, _returned_buffer: &hil::uart::TxTransaction<'a>) {
        
        // we don't care about our buffers, since they don't take up RAM

        //increment state counter
        self.state += 1;
    }
}

