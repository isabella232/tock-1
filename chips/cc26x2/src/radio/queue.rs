pub trait Queue<T> {
    /// Returns true if there are any items in the queue, false otherwise.
    fn has_elements(&self) -> bool;

    /// Returns true if the queue is full, false otherwise.
    fn is_full(&self) -> bool;

    /// Returns how many elements are in the queue.
    fn len(&self) -> usize;

    /// Add a new element to the back of the queue.
    fn enqueue(&mut self, val: T) -> bool;

    /// Remove the element from the front of the queue.
    fn dequeue(&mut self) -> Option<T>;

    /// Remove all elements from the ring buffer.
    fn empty(&mut self);
}

#[repr(C)]
pub struct RFBuffer<'a, T: 'a> {
    head: u8,
    tail: u8,
    ring: &'a mut [T],
}

#[repr(C)]
pub struct QueueEntry {
    next_entry: u8,
    dtype: u8,
    len_sz: u8,
    irq_int: u8,
    length: u16,
    data: u8,
}

impl<'a, T: Copy> RFBuffer<'a, T> {
    pub fn new(ring: &'a mut [T]) -> RFBuffer<'a, T> {
        // let cmd: &mut prop::CommandRx = &mut *(COMMAND_BUF.as_mut_ptr() as *mut prop::CommandRx);
        unsafe {
            let p_ring = &mut *(ring.as_mut_ptr() as *mut QueueEntry);
            p_ring.next_entry = 0;
            p_ring.dtype = 2;
            p_ring.len_sz = 2;
            p_ring.irq_int = 4;
            p_ring.length = 240;
        }

        RFBuffer {
            head: ring.as_ptr() as u8,
            tail: ring.as_ptr() as u8,
            ring: ring,
        }
    }
}

/*
impl<'a, T: Copy> Queue<T> for RFBuffer<'a, T> {
    fn has_elements(&self) -> bool {
        self.head != self.tail
    }

    fn is_full(&self) -> bool {
        self.head == ((self.tail + 1) % self.ring.len())
    }

    fn len(&self) -> usize {
        if self.tail > self.head {
            self.tail - self.head
        } else if self.tail < self.head {
            (self.ring.len() - self.head) + self.tail
        } else {
            // head equals tail, length is zero
            0
        }
    }

    fn enqueue(&mut self, val: T) -> bool {
        if ((self.tail + 1) % self.ring.len()) == self.head {
            // Incrementing tail will overwrite head
            return false;
        } else {
            self.ring[self.tail] = val;
            self.tail = (self.tail + 1) % self.ring.len();
            return true;
        }
    }

    fn dequeue(&mut self) -> Option<T> {
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            None
        }
    }

    fn empty(&mut self) {
        self.head = 0;
        self.tail = 0;
    }
}
*/
