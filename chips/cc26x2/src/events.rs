use kernel::common::cells::VolatileCell;

pub static mut EVENTS: u32 = 0;

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
    RF_CORE_HW = 8,
    RF_CMD_ACK = 9,
    RF_CORE_CPE0 = 10,
    RF_CORE_CPE1 = 11,
    OSC = 12,
}
}

use cortexm::support::atomic;

pub fn has_event() -> bool {
    let mut event_flags = 0;
    unsafe {
        atomic(|| {
                event_flags = EVENTS;
        });
    }
    event_flags != 0
}

pub fn next_pending() -> Option<EVENT_PRIORITY> {
    let mut event_flags = 0;
    unsafe {
        atomic(|| {
            event_flags = EVENTS;
         });
    }

    let mut count = 0;
    // stay in loop until we found the flag
    while event_flags != 0 {
        // if flag is found, return the count
        if (event_flags & 0b1) != 0 {
            //debug!("count: {}", count);
            return Some(EVENT_PRIORITY::from_u8(count).expect("Unmapped EVENT_PRIORITY"));
        }
        // otherwise increment
        count += 1;
        event_flags >>= 1;
    }
    None
}

#[inline(always)]
#[naked]
pub fn set_event_flag(priority: EVENT_PRIORITY) {
    unsafe {
        let bm = 0b1 << (priority as u8) as u64;
        //atomic(|| {
            EVENTS |= bm;
        //})
    };
}

pub fn clear_event_flag(priority: EVENT_PRIORITY) {
    unsafe {
        let bm = !0b1 << (priority as u8) as u64;
        atomic(|| {
            EVENTS &= bm;
        })
    };
}
