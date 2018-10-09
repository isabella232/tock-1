pub struct Event{
    pub flags: u64,
}

use support::{atomic, atomic_read};

impl Event{

    pub fn has_event(&self) -> bool {
        let event_flags;
        unsafe { event_flags = atomic_read(&self.flags) }
        event_flags != 0
    }

    pub fn next_pending(&self) -> Option<usize> {
        let mut event_flags;
        unsafe { event_flags = atomic_read(&self.flags) }

        let mut count = 0;
        // stay in loop until we found the flag
        while event_flags != 0 {
            // if flag is found, return the count
            if (event_flags & 0b1) != 0 {
                return Some(count);
            }
            // otherwise increment
            count += 1;
            event_flags >>= 1;
        }
        None
    }

    #[inline]
    pub fn set_event_flag(&mut self, priority: usize){
        unsafe {
            let bm = 0b1 << priority;
            atomic(|| {
                self.flags |= bm;
            })
        };
    }

    pub fn clear_event_flag(&mut self, priority: usize){
        unsafe {
            let bm = !0b1 << priority;
            atomic(|| {
                self.flags &= bm;
            })
        };
    }
}
