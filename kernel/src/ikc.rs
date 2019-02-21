pub struct TxRequest<'a, T> {
    pub items: &'a [T],
    // The total amount to transmit
    pub length: usize,
    // The index of the byte currently being sent
    pub index: usize,
    // Identifier to route response to owner
    pub client_id: usize,
}

// Stores an ongoing TX or RX Request
pub struct RxRequest<'a, T> {
    pub items: &'a mut [T],
    // The total amount to transmit
    pub length: usize,
    // The index of the byte currently being sent
    pub index: usize,
    // Identifier to route response to owner
    pub client_id: usize,
}

pub enum Request<'a, T> {
    RX(&'a mut RxRequest<'a, T>),
    TX(&'a mut TxRequest<'a, T>),
}

pub enum DriverState<'a, T> {
    REQUEST_COMPLETE(Request<'a,T>),
    BUSY,
    IDLE,
}

impl<'a, T> TxRequest<'a, T> {

    pub fn new(items: &'a [T]) -> TxRequest<'a,T> {
        // throw error if length > buffer.length()
        TxRequest {
            length: items.len(),
            items,
            index: 0,
            client_id: 0xFF,
        }
    }

    pub fn new_set_len(items: &'a [T], length: usize) -> TxRequest<'a,T> {
        // throw error if length > buffer.length()
        TxRequest {
            items,
            length,
            index: 0,
            client_id: 0xFF,
        }
    }
}

impl<'a, T> RxRequest<'a, T> {
    pub fn new(items: &'a mut [T]) -> RxRequest<'a,T> {
        // throw error if length > buffer.length()
        RxRequest {
            length: items.len(),
            items,
            index: 0,
            client_id: 0xFF,
        }
    }

    pub fn new_set_len(items: &'a mut [T], length: usize) -> RxRequest<'a,T> {
        // throw error if length > buffer.length()
        RxRequest {
            items,
            length,
            index: 0,
            client_id: 0xFF,
        }
    }
}