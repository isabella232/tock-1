use kernel::common::registers::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use memory_map::UDMA0_BASE;

pub struct UDMARegisters {
    _status: ReadOnly<u32, Stat::Register>,
    _cfg: WriteOnly<u32, Cfg::Register>,
    _ctrl: ReadWrite<u32, Ctrl::Register>,
    _alt_ctrl: ReadOnly<u32>,
    _wait_on_req: ReadOnly<u32>,
    _soft_req: ReadOnly<u32>,
    set_burst: ReadWrite<u32, Channels::Register>,
    clear_burst: WriteOnly<u32, Channels::Register>,
    set_req_mask: ReadWrite<u32, Channels::Register>,
    clear_req_mask: WriteOnly<u32, Channels::Register>,
    _set_channel_len: ReadWrite<u32, Channels::Register>,
    _clear_channel_len: WriteOnly<u32, Channels::Register>,
    set_channel_pri_alt: ReadWrite<u32, Channels::Register>,
    clear_channel_pri_alt: WriteOnly<u32, Channels::Register>,
    set_channel_pri: ReadWrite<u32, Channels::Register>,
    clear_channel_pri: WriteOnly<u32, Channels::Register>,
    _reserved: [ReadOnly<u32>; 3],
    _error: ReadWrite<u32, Error::Register>,
    _reserved2: [u8; 0x4b4],
    _req_done: ReadWrite<u32>,
    _reserved3: [u8; 0x18],
    _done_mask: ReadWrite<u32>,
}

register_bitfields! [
    u32,
    Stat [
        TEST            OFFSET(28) NUMBITS(4) [],
        // Reserved 27:21
        TOTAL_CHANNELS  OFFSET(21) NUMBITS(6) [],
        // Reserved 15:8
        STATE           OFFSET(4) NUMBITS(4) [],
        // Reserved 3:1
        MASTER_ENABLE   OFFSET(0) NUMBITS(1) []
    ],
    Cfg [
        // Reserved 31:8
        PROTO_CTRL      OFFSET(5) NUMBITS(3) [],
        // Reserved 4:1
        MASTER_Enable   OFFSET(0) NUMBITS(1) []
    ],
    Ctrl [
        BASE_PTR        OFFSET(10) NUMBITS(21) []
        // Reserved 9:0
    ],
    Channels [
        TOGGLE          OFFSET(0) NUMBITS(32) []
    ],
    Error [
        STATUS          OFFSET(0) NUMBITS(1) []
    ]
];

const UDMA_REG: StaticRef<UDMARegisters> =
    unsafe { StaticRef::new(UDMA0_BASE as *const UDMARegisters) };

pub const UDMA: Udma = Udma::new();

pub enum Attribute {
    UseBurst,
    AltSelect,
    HighPriority,
    ReqMask,
}

pub struct Udma {
    regs: StaticRef<UDMARegisters>,
}

impl Udma {
    pub const fn new() -> Udma {
        Udma { regs: UDMA_REG }
    }

    pub fn attribute_enable(&self, channel_num: u32, attribute: Attribute) {
        let channel_en = 1 << channel_num;
        match attribute {
            Attribute::UseBurst => {
                self.regs.set_burst.modify(Channels::TOGGLE.val(channel_en));
            }
            Attribute::AltSelect => {
                self.regs
                    .set_channel_pri_alt
                    .modify(Channels::TOGGLE.val(channel_en));
            }
            Attribute::HighPriority => {
                self.regs
                    .set_channel_pri
                    .modify(Channels::TOGGLE.val(channel_en));
            }
            Attribute::ReqMask => {
                self.regs
                    .set_req_mask
                    .modify(Channels::TOGGLE.val(channel_en));
            }
        }
    }

    pub fn attribute_disable(&self, channel_num: u32, attribute: Attribute) {
        let channel_dis = 1 << channel_num;
        match attribute {
            Attribute::UseBurst => {
                self.regs
                    .clear_burst
                    .write(Channels::TOGGLE.val(channel_dis));
            }
            Attribute::AltSelect => {
                self.regs
                    .clear_channel_pri_alt
                    .write(Channels::TOGGLE.val(channel_dis));
            }
            Attribute::HighPriority => {
                self.regs
                    .clear_channel_pri
                    .write(Channels::TOGGLE.val(channel_dis));
            }
            Attribute::ReqMask => {
                self.regs
                    .clear_req_mask
                    .write(Channels::TOGGLE.val(channel_dis));
            }
        }
    }

    pub fn attribute_get(&self, channel_num: u32) -> u32 {
        let mut channel_en: u32 = 0;

        if (self.regs.set_burst.get() & (1 << channel_num)) != 0 {
            channel_en |= 0x01;
        }
        if (self.regs.set_channel_pri_alt.get() & (1 << channel_num)) != 0 {
            channel_en |= 0x02;
        }
        if (self.regs.set_channel_pri.get() & (1 << channel_num)) != 0 {
            channel_en |= 0x04;
        }
        if (self.regs.set_req_mask.get() & (1 << channel_num)) != 0 {
            channel_en |= 0x08;
        }
        return channel_en;
    }
}
