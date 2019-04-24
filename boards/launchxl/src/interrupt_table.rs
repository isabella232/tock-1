use crate::event_priority;
use cortexm::{events, generic_isr};
use cortexm4::{
    generic_isr, hard_fault_handler, set_privileged_thread,
    stash_process_state, svc_handler, systick_handler,
};

use cc26x2::default_interrupt_table;

default_interrupt_table!();
