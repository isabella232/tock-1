//! These are some primitive generics for Intra-Kernel Communication
pub enum TxItems<'a, T: Copy> {
    None,
    CONST(&'a [T]),
    MUT(&'a mut [T])
}

pub struct TxRequest<'a, T: Copy> {
    pub items: TxItems<'a, T>,
    // The total amount to transmit
    pub length: usize,
    // The index of the byte currently being sent
    pub index: usize,
    // Identifier to route response to owner
    pub client_id: usize,
}

// Stores an ongoing TX or RX Request
pub struct RxRequest<'a, T: Copy> {
    pub items: &'a mut [T],
    // The total amount to transmit
    pub length: usize,
    // The index of the byte currently being sent
    pub index: usize,
    // Identifier to route response to owner
    pub client_id: usize,
}

pub enum Request<'a, T: Copy> {
    TX(&'a mut TxRequest<'a, T>),
    RX(&'a mut RxRequest<'a, T>),
}

pub enum DriverState<'a, T: Copy> {
    REQUEST_COMPLETE(Request<'a,T>),
    BUSY,
    IDLE,
}

impl<'a, T: Copy> TxRequest<'a, T> {
    pub fn pop_item(&mut self) -> Option<T> {
        let ret = match &self.items {
            TxItems::CONST(s) => {
                Some(s[self.index])
            }
            TxItems::MUT(ref s) => Some(s[self.index]),
            TxItems::None => None
        };
        self.index += 1;
        ret
    }

    pub fn has_some(&self) -> bool {
        self.index < self.length
    }


    pub fn set_with_const_ref(&mut self, items: &'a [T]) {
        self.length = items.len();
        self.items = TxItems::CONST(items);
        self.index = 0;
    }

    pub fn set_with_mut_ref(&mut self, items: &'a mut [T]) {
        self.length = items.len();
        self.items = TxItems::MUT(items);
        self.index = 0;
    }

    pub fn new() -> TxRequest<'a,T> {
        TxRequest {
            length: 0,
            items: TxItems::None,
            index: 0,
            client_id: 0xFF,
        }
    }

    pub fn new_with_const_ref(items: &'a [T]) -> TxRequest<'a,T> {
        let length = items.len();
        Self::new_with_ref_set_len(TxItems::CONST(items), length)
    }

    pub fn new_with_mut_ref(items: &'a mut [T]) -> TxRequest<'a,T> {
        let length = items.len();
        Self::new_with_ref_set_len(TxItems::MUT(items), length)
    }

    pub fn new_with_ref_set_len(items: TxItems<'a, T>, length: usize) -> TxRequest<'a,T> {
        match items {
            TxItems::CONST(s) => {
                TxRequest {
                    length: length,
                    items: TxItems::CONST(s),
                    index: 0,
                    client_id: 0xFF,
                }
            },
            TxItems::MUT(s) => {
                TxRequest {
                    length: length,
                    items: TxItems::MUT(s),
                    index: 0,
                    client_id: 0xFF,
                }
            }
            TxItems::None => {
                TxRequest {
                    length: 0,
                    items: TxItems::None,
                    index: 0,
                    client_id: 0xFF,
                }
            }
        }
    }
}

impl<'a, T: Copy> RxRequest<'a, T> {
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