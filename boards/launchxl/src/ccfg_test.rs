static CCFG_CONF: [u32; 22] = [
    0x01800000, 0xFF820010, 0x0058FFFE, 0xF3FBFF3A, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
    0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x00FFFFFF, 0xFFFFFFFF, 0xFFFFFF00, 0xFFC5C5C5,
    0xFFC5C5C5, 0x00000000, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
];

use kernel::common::registers::ReadWrite;
use cc26x2::ccfg::*;

const CCFG: Registers = Registers::new(
    RegisterInitializer {
        ext_lf_clk: ReadWrite::new(0x01800000),
        mode_conf0: ReadWrite::new(0xF3FBFF3A),
        mode_conf1: ReadWrite::new(0xFF820010),
        bl_config: ReadWrite::new(0x00FFFFFA), //las char should be F
    }
);


pub fn test(){
    unsafe {
        let raw_array: *const u32 = &CCFG_CONF[0] as *const u32;
        let constructed = &CCFG.ext_lf_clk.value as *const u32;

        for n in 0..22 {
            let raw = *(raw_array.offset(n));
            let new = *(constructed.offset(n));
            if raw != new {
                debug!("Mismatch at location {:} : OLD {:x} != {:x} NEW", n, raw, new);
            }
            //*(ui32RegAddr as (*mut u16));
        }
    }

}