use core::cell::Cell;
use core::ptr;

pub enum ProcStack {}

impl ProcStack {
    pub fn new() -> *mut ProcStack {
        ptr::null_mut()
    }
}

#[allow(non_camel_case_types)]
pub struct ProcThread {
    // ID of the process
    process_id: usize,

    // ID of the parent process if there is one
    parent_id: usize,

    // Thread interval timer
    interval_timer: Cell<usize>,

    // Thread priority
    priority: usize,

    // ID of the "thread"
    proc_mem: ProcStack,
}

impl ProcThread {
    pub fn new() -> ProcThread {
        ProcThread {}
    }
}
