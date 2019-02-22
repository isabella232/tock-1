//! These are some primitive generics for Intra-Kernel Communication
pub enum TxBuf<'a, T: Copy> {
    None,
    CONST(&'a [T]),
    MUT(&'a mut [T])
}

pub enum RxBuf<'a, T: Copy> {
    None,
    MUT(&'a mut [T])
}

pub struct TxRequest<'a, T: Copy> {
    buf: TxBuf<'a, T>,
    // The total amount of data written in
    pushed: usize,
    // The total amount of data read out
    popped: usize,
    // The total size of the request
    requested: usize,
    // Identifier to route response to owner
    pub client_id: usize,
}

// Stores an ongoing TX or RX Request
pub struct RxRequest<'a, T: Copy> {
    buf: RxBuf<'a, T>,
    // The total amount of data written in
    pushed: usize,
    // The total amount of data read out
    popped: usize,
    // The total size of the request
    requested: usize,
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
    pub fn pop(&mut self) -> Option<T> {
        let ret = match &self.buf {
            TxBuf::CONST(s) => {
                Some(s[self.popped])
            }
            TxBuf::MUT(ref s) => Some(s[self.popped]),
            TxBuf::None => None
        };
        self.popped += 1;
        ret
    }

    pub fn push(&mut self, element: T) {
        match &mut self.buf {
            TxBuf::MUT(buf) => {
                buf[self.pushed] = element;
            },
            TxBuf::CONST(buf) => {},
            TxBuf::None => {},
        }

        // increment both the pushed and requested amount
        self.pushed += 1;
        self.requested += 1;
    }

    pub fn has_some(&self) -> bool {
        self.popped < self.requested
    }

    pub fn requested_completed(&self) -> bool {
        self.popped >= self.requested
    }

    pub fn has_room(&self) -> bool {
        match &self.buf {
            TxBuf::MUT(buf) => self.pushed < buf.len(),
            TxBuf::CONST(buf) => false,
            TxBuf::None => false,
        }
    }

    pub fn room_available(&self) -> usize {
        match &self.buf {
            TxBuf::MUT(buf) => buf.len() - self.pushed,
            TxBuf::CONST(buf) => 0,
            TxBuf::None => 0,
        }
    }

    pub fn reset(&mut self) {
        self.pushed = 0;
        self.popped = 0;
        self.requested = 0;
    }

    // for TxRequest with const reference, pushed = requested = buffer length
    pub fn set_with_const_ref(&mut self, buf: &'a [T]) {
        self.pushed = buf.len();
        self.requested = buf.len();
        self.buf = TxBuf::CONST(buf);
        self.popped = 0;
    }

    // for TxRequest with mutable reference
    // it is assumed empty so client will fill before dispatching
    pub fn set_with_mut_ref(&mut self, buf: &'a mut [T]) {
        self.buf = TxBuf::MUT(buf);
        self.pushed = 0;
        self.popped = 0;
        self.requested = 0;
    }

    // initializes space expect for the TxItem, which must be allocated elsewhere
    pub fn new() -> TxRequest<'a,T> {
        TxRequest {
            buf: TxBuf::None,
            pushed: 0,
            popped: 0,
            requested: 0,
            client_id: 0xFF,
        }
    }

    pub fn new_with_const_ref(buf: &'a [T]) -> TxRequest<'a,T> {
        let length = buf.len();
        Self::new_with_ref_set_len(TxBuf::CONST(buf), length)
    }

    pub fn new_with_mut_ref(buf: &'a mut [T]) -> TxRequest<'a,T> {
        let length = buf.len();
        Self::new_with_ref_set_len(TxBuf::MUT(buf), length)
    }

    pub fn new_with_ref_set_len(buf: TxBuf<'a, T>, length: usize) -> TxRequest<'a,T> {
        match buf {
            TxBuf::CONST(b) => {
                TxRequest {
                    buf: TxBuf::CONST(b),
                    pushed: length,
                    requested: length,
                    popped: 0,
                    client_id: 0xFF,
                }
            },
            TxBuf::MUT(b) => {
                TxRequest {
                    buf: TxBuf::MUT(b),
                    pushed: 0,
                    popped: 0,
                    requested: 0,
                    client_id: 0xFF,
                }
            }
            TxBuf::None => {
                TxRequest {
                    buf: TxBuf::None,
                    pushed: 0,
                    popped: 0,
                    requested: 0,
                    client_id: 0xFF,
                }
            }
        }
    }
}

impl<'a, T: Copy> RxRequest<'a, T> {
    pub fn new() -> RxRequest<'a,T> {
        RxRequest {
            buf: RxBuf::None,
            pushed: 0,
            popped: 0,
            requested: 0,
            client_id: 0xFF,
        }
    }

    // RxRequest is assumed empty and we assume client wants host to fill buffer
    pub fn set_buf(&mut self, buf: &'a mut [T]) {
        self.requested = buf.len();
        self.buf = RxBuf::MUT(buf);
        self.pushed = 0;
        self.popped = 0;
    }

    // Reset pushed/popped values and assume client wants host to fill buffer
    pub fn reset(&mut self) {
        self.pushed = 0;
        self.popped = 0;
        match &self.buf {
            RxBuf::MUT(buf) => self.requested = buf.len(),
            RxBuf::None => self.requested = 0,
        }
    }

    // Host has pushed enough data to fill the buffer
    pub fn requested_completed(&self) -> bool {
        self.pushed >= self.requested
    }

    pub fn set_requested_len(&mut self, buf: &'a mut [T]) {
        self.requested = 0;
    }

    pub fn has_room(&self) -> bool {
        match &self.buf {
            RxBuf::MUT(buf) => self.pushed < buf.len(),
            RxBuf::None => false,
        }
    }

    pub fn push(&mut self, element: T) {
        match &mut self.buf {
            RxBuf::MUT(buf) => {
                buf[self.pushed] = element;
            },
            RxBuf::None => {},
        }
        self.pushed += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        let ret = match &self.buf {
            RxBuf::MUT(s) => {
                Some(s[self.popped])
            }
            RxBuf::None => None
        };
        self.popped += 1;
        ret
    }
}