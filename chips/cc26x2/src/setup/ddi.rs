pub fn ddi16bit_write(base: u32, reg: u32, mut mask: u32, wr_data: u32) {
    let mut reg_addr = base.wrapping_add(reg << 1).wrapping_add(0x400);
    if mask & 0xffff0000 != 0 {
        reg_addr = reg_addr.wrapping_add(4);
        mask = mask >> 16;
    }
    let data: u32;
    if wr_data != 0 {
        data = mask;
    } else {
        data = 0;
    }
    let ptr = reg_addr as *mut usize;
    unsafe { *ptr = (mask << 16 | data) as usize };
}

pub fn ddi16bitfield_write(base: u32, reg: u32, mut mask: u32, mut shift: u32, data: u16) {
    let mut reg_addr = base.wrapping_add(reg << 1).wrapping_add(0x400);
    if shift >= 16 {
        shift = shift.wrapping_sub(16);
        reg_addr = reg_addr.wrapping_add(4);
        mask = mask >> 16 as i32;
    }
    let wr_data = data << shift;

    let ptr = reg_addr as *mut usize;
    unsafe { *ptr = (mask << 16 | wr_data as u32) as usize };
}

pub fn ddi16bit_read(base: u32, reg: u32, mut mask: u32) -> u16 {
    let mut reg_addr = base.wrapping_add(reg).wrapping_add(0);
    if mask & 0xffff0000 != 0 {
        reg_addr = reg_addr.wrapping_add(2);
        mask = mask >> 16 as i32;
    }
    let ptr = reg_addr as *mut usize;
    let mut data: u16;
    unsafe { data = *ptr as u16 };
    data = (data as u32 & mask) as u16;
    data
}

pub fn ddi16bitfield_read(base: u32, reg: u32, mut mask: u32, mut shift: u32) -> u16 {
    let mut reg_addr = base.wrapping_add(reg).wrapping_add(0);
    if shift >= 16 {
        shift = shift.wrapping_sub(16);
        reg_addr = reg_addr.wrapping_add(2);
        mask = mask >> 16;
    }
    let ptr = reg_addr as *mut usize;
    let mut data: u16;
    unsafe { data = *ptr as u16 };
    data = (data as u32 & mask) as u16;
    data = data >> shift as u16;
    data
}

pub fn ddi32reg_write(base: u32, reg: u32, val: u32) {
    let ptr = base.wrapping_add(reg) as *mut usize;
    unsafe { *ptr = val as usize };
}
