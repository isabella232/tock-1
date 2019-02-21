//! These are some primitive generics for Intra-Kernel Communication
pub enum TxItems<'a, T>{
    CONST(Option<&'a [T]>),
    MUT(&'a mut [T])
}

pub struct TxRequest<'a, T> {
    pub items: TxItems<'a, T>,
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
    TX(&'a mut TxRequest<'a, T>),
    RX(&'a mut RxRequest<'a, T>),
}

pub enum DriverState<'a, T> {
    REQUEST_COMPLETE(Request<'a,T>),
    BUSY,
    IDLE,
}

impl<'a, T> TxRequest<'a, T> {
    pub fn pop_item(&mut self) -> Option<T> {
        let ret = match &self.items {
            TxItems::CONST(maybe_s) => {
                match maybe_s {
                    Some(ref s) => Some(s[self.index]),
                    None => None,
                }
            }
            TxItems::MUT(ref s) => Some(s[self.index]),
        };
        self.index += 1;
        ret
    }

    pub fn new() -> TxRequest<'a,T> {
        TxRequest {
            length: 0,
            items: TxItems::CONST(None),
            index: 0,
            client_id: 0xFF,
        }
    }

    pub fn new_with_ref(items: TxItems<'a, T>) -> TxRequest<'a,T> {
        match items {
            TxItems::CONST(maybe_s) => {
                match maybe_s {
                    Some(s) => {
                        let length = s.len();
                        Self::new_with_ref_set_len(items, length)
                    },
                    None => Self::new(),
                }
            },
            TxItems::MUT(s) => {
                let length = s.len();
                Self::new_with_ref_set_len(TxItems::MUT(s), length)
            }
        }
    }

    pub fn new_with_ref_set_len(items: TxItems<'a, T>, length: usize) -> TxRequest<'a,T> {
        match items {
            TxItems::CONST(maybe_s) => {
                match maybe_s {
                    Some(s) => {
                        TxRequest {
                            length: length,
                            items: TxItems::CONST(Some(s)),
                            index: 0,
                            client_id: 0xFF,
                        }
                    },
                    None => Self::new(),
                }
                
            },
            TxItems::MUT(s) => {
                TxRequest {
                    length: 0,
                    items: TxItems::MUT(s),
                    index: 0,
                    client_id: 0xFF,
                }
            }
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