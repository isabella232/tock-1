pub static mut EVENTS: u32 = 0;

use core::ptr;
use enum_primitive::cast::FromPrimitive;

enum_from_primitive!{
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EVENT_PRIORITY {
    GPIO = 0,
    UART0 = 2,
    UART1 = 1,
    AON_RTC = 3,
    RTC = 4,
    I2C0 = 6,
    AON_PROG = 7,
}
}

use cortexm::support::atomic;

pub fn has_event() -> bool {
    let event_flags;
    unsafe { event_flags = ptr::read_volatile(&EVENTS) }
    event_flags != 0
}

pub fn next_pending() -> Option<EVENT_PRIORITY> {
    let mut event_flags;
    unsafe { event_flags = ptr::read_volatile(&EVENTS) }

    let mut count = 0;
    // stay in loop until we found the flag
    while event_flags != 0 {
        // if flag is found, return the count
        if (event_flags & 0b1) != 0 {
            return Some(EVENT_PRIORITY::from_u8(count).expect("Unmapped EVENT_PRIORITY"));
        }
        // otherwise increment
        count += 1;
        event_flags >>= 1;
    }
    None
}

#[inline(never)]
pub fn set_event_flag(priority: EVENT_PRIORITY) {
    unsafe {
        let bm = 0b1 << (priority as u8) as u32;
        atomic(|| {
            let new_value = ptr::read_volatile(&EVENTS) | bm;
            EVENTS = new_value;
        })
    };
}

pub fn clear_event_flag(priority: EVENT_PRIORITY) {
    unsafe {
        let bm = !0b1 << (priority as u8) as u32;
        atomic(|| {
            let new_value = ptr::read_volatile(&EVENTS) & bm;
            EVENTS = new_value;
        })
    };
}
