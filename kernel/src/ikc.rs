//! These are some primitive generics for Intra-Kernel Communication
use super::{AppSlice, Shared};
use core::cmp;


pub enum TxBuf<'a, T: Copy> {
    None,
    CONST(&'a [T]),
    MUT(&'a mut [T]),
    APP_SLICE(AppSlice<Shared, T>),
}

pub enum RxBuf<'a, T: Copy> {
    None,
    MUT(&'a mut [T]),
    //APP_SLICE(AppSlice<Shared, T>),
}

impl<'a, T: Copy> Default for RxBuf<'a, T> {
    fn default() -> Self { RxBuf::None }
}

impl<'a, T: Copy> Default for TxBuf<'a, T> {
    fn default() -> Self { TxBuf::None }
}

#[derive(Default)]
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

#[derive(Default)]
pub struct RxRequest<'a, T: Copy> {
    pub buf: RxBuf<'a, T>,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DriverState {
    BUSY,
    IDLE,
}

impl<'a, T: Copy> TxRequest<'a, T> {
    pub fn pop(&mut self) -> Option<T> {
        let ret = match &self.buf {
            TxBuf::CONST(s) => Some(s[self.popped]),
            TxBuf::MUT(ref s) => Some(s[self.popped]),
            TxBuf::APP_SLICE(ref s) => Some( s.as_ref()[self.popped]),
            TxBuf::None => None,
        };
        self.popped += 1;
        ret
    }

    // this interface is for a Client that wants to prepare a Request
    // as such, APP_SLICE and CONST types are not treated
    pub fn push(&mut self, element: T) {
        match &mut self.buf {
            TxBuf::MUT(buf) => {
                buf[self.pushed] = element;
            }
            TxBuf::CONST(_buf) => panic!("Should not be pushing data into constant TxRequest!"),
            TxBuf::APP_SLICE(_s) => panic!("Should not be pushing data into AppSlice TxRequest!"),
            TxBuf::None => panic!("Should not be pushing data into TxRequest with no TxBuf!")
        }

        // increment both the pushed and requested amount
        self.pushed += 1;
        self.requested += 1;
    }

    pub fn copy_from_app_slice(&mut self, input: &mut TxRequest<'a, T>) {
        match &mut self.buf {
            TxBuf::MUT(ref mut buf) => {
                match &input.buf{
                    TxBuf::APP_SLICE(s) => {
                        let num_elements = cmp::min(buf.len(), input.requested - input.popped);
                        for i in 0..num_elements {
                            buf[i] = s.as_ref()[i + input.popped];
                        }
                        input.popped += num_elements;
                    }
                    _ => panic!("Can only copy_from_app_slice if input is TxBuf::APP_SLICE!")
                }
            },
            _ => panic!("Can only copy_from_app_slice if self is TxBuf::MUT"),
        };
        self.popped += 1;
    }

    pub fn has_some(&self) -> bool {
       self.popped < self.pushed
    }


    pub fn requested_length(&self) -> usize {
        self.requested
    }

    pub fn remaining_request(&self) -> usize {
        self.requested - self.popped
    }

    pub fn request_completed(&self) -> bool {
        self.popped >= self.requested
    }

    pub fn has_room(&self) -> bool {
        match &self.buf {
            TxBuf::MUT(buf) => self.pushed < buf.len(),
            _ => false,
        }
    }

    pub fn room_available(&self) -> usize {
        match &self.buf {
            TxBuf::MUT(buf) => buf.len() - self.pushed,
            _ => 0,
        }
    }

    pub fn reset(&mut self) {
        self.pushed = 0;
        self.popped = 0;
        match &self.buf { 
            TxBuf::MUT(buf) => self.requested = 0,
            TxBuf::CONST(buf) => self.requested = buf.len(),
            TxBuf::APP_SLICE(s) => self.requested = s.len(),
            TxBuf::None => {},
        }
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

    // for TxRequest with mutable reference
    // it is assumed empty so client will fill before dispatching
    pub fn set_with_app_slice(&mut self, slice: AppSlice<Shared, T>) {
        let default_request_length  = slice.len();
        self.buf = TxBuf::APP_SLICE(slice);
        self.pushed = 0;
        self.popped = 0;
        self.requested = default_request_length;
    }

    // initializes space expect for the TxItem, which must be allocated elsewhere
    pub fn new() -> TxRequest<'a, T> {
        TxRequest {
            buf: TxBuf::None,
            pushed: 0,
            popped: 0,
            requested: 0,
            client_id: 0xFF,
        }
    }

    pub fn new_with_const_ref(buf: &'a [T]) -> TxRequest<'a, T> {
        let length = buf.len();
        Self::new_with_ref_set_len(TxBuf::CONST(buf), length)
    }

    pub fn new_with_mut_ref(buf: &'a mut [T]) -> TxRequest<'a, T> {
        let length = buf.len();
        Self::new_with_ref_set_len(TxBuf::MUT(buf), length)
    }

    // allow user to set request length, but don't let it exceed buffer/slice size
    pub fn set_request_len(&mut self, length: usize) {
        match &self.buf { 
            TxBuf::MUT(buf) => self.requested = cmp::min(length, buf.len()),
            TxBuf::CONST(buf) => self.requested = cmp::min(length, buf.len()),
            TxBuf::APP_SLICE(s) => self.requested = cmp::min(length, s.len()),
            TxBuf::None => {},
        }
    }

    pub fn new_with_ref_set_len(buf: TxBuf<'a, T>, length: usize) -> TxRequest<'a, T> {
        match buf {
            TxBuf::CONST(b) => TxRequest {
                buf: TxBuf::CONST(b),
                pushed: length,
                requested: length,
                popped: 0,
                client_id: 0xFF,
            },
            TxBuf::MUT(b) => TxRequest {
                buf: TxBuf::MUT(b),
                pushed: 0,
                popped: 0,
                requested: 0,
                client_id: 0xFF,
            },
            TxBuf::APP_SLICE(s) => TxRequest {
                buf: TxBuf::APP_SLICE(s),
                pushed: 0,
                popped: 0,
                requested: length,
                client_id: 0xFF,
            },
            TxBuf::None => TxRequest {
                buf: TxBuf::None,
                pushed: 0,
                popped: 0,
                requested: 0,
                client_id: 0xFF,
            },
        }
    }
}

impl<'a, T: Copy> RxRequest<'a, T> {
    pub fn new() -> RxRequest<'a, T> {
        RxRequest {
            buf: RxBuf::None,
            pushed: 0,
            popped: 0,
            requested: 0,
            client_id: 0xFF,
        }
    }

    pub fn new_with_mut_ref(buf: &'a mut [T]) -> RxRequest<'a, T> {
        RxRequest {
            requested: buf.len(),
            buf: RxBuf::MUT(buf),
            pushed: 0,
            popped: 0,
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

    // Host has pushed enough data to fulfill the request
    pub fn request_completed(&self) -> bool {
        self.pushed >= self.requested
    }

    // How much data has been pushed
    pub fn items_pushed(&self) -> usize {
        self.pushed
    }

    pub fn request_remaining(&self) -> usize {
        self.requested - self.pushed
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
            }
            RxBuf::None => {}
        }
        self.pushed += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        let ret = match &self.buf {
            RxBuf::MUT(s) => Some(s[self.popped]),
            RxBuf::None => None,
        };
        self.popped += 1;
        ret
    }
}
