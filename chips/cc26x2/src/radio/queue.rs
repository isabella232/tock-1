use core;

pub static mut READENTRY: *mut dataEntryGeneral = 0 as (*mut dataEntryGeneral);

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct DataQueue {
    pub p_curr_entry: *mut u8,
    pub p_last_entry: *mut u8,
}

impl DataQueue {
    pub fn new(curr_entry: *mut u8, next_entry: *mut u8) -> DataQueue {
        DataQueue {
            p_curr_entry: curr_entry,
            p_last_entry: next_entry,
        }
    }

    pub unsafe fn define_queue(
        &mut self,
        mut buf: *mut u8,
        buf_len: u16,
        num_entries: u32,
        length: u16,
    ) {
        if buf_len as (u32) < num_entries * (length as u32 + 8 + (4 - (length as u32 + 8) % 4)) {
            debug!("Queue Error: Buffer length shorter than entry length.");
            return;
        } else {
            let pad: u8 = (4u32 - (length as (u32) + 8u32) % 4u32) as u8;
            let first_entry: *mut u8 = buf;
            let mut i: u32;
            i = 0;
            while i < num_entries {
                buf = first_entry.offset((i * (8 + length as u32 + pad as u32)) as isize);
                (*(buf as *mut dataEntry)).status = 0u8;
                (*(buf as *mut dataEntry)).config.d_type = 0u8;
                (*(buf as *mut dataEntry)).config.len_sz = 1u8;
                (*(buf as *mut dataEntry)).config.irq_intv = 4u8;
                (*(buf as *mut dataEntry)).length = length;
                (*(buf as *mut dataEntryGeneral)).p_next_entry =
                    (&mut (*(buf as (*mut dataEntryGeneral))).data as (*mut u8))
                        .offset(length as (isize))
                        .offset(pad as (isize));
                i = i + 1;
            }
            (*(buf as (*mut dataEntry))).p_next_entry = first_entry;
            self.p_curr_entry = first_entry;
            self.p_last_entry = core::ptr::null_mut();
            READENTRY = first_entry as (*mut dataEntryGeneral);
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DataConfig {
    pub d_type: u8,
    pub len_sz: u8,
    pub irq_intv: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dataEntry {
    pub p_next_entry: *mut u8,
    pub status: u8,
    pub config: DataConfig,
    pub length: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dataEntryGeneral {
    pub p_next_entry: *mut u8,
    pub status: u8,
    pub config: DataConfig,
    pub length: u16,
    pub data: u8,
}

#[repr(C)]
pub struct TestQueue {
    pub p_next_entry: *mut u8,
    pub status: u8,
    pub d_type: u8,
    pub len_sz: u8,
    pub irq_intv: u8,
    pub length: u16,
    pub data: u8,
}
