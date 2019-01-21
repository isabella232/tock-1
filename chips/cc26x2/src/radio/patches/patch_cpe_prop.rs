use kernel::common::cells::VolatileCell;

pub const CPE_PATCH: Patches = Patches::new();

#[repr(C)]
pub struct CPERam {
    rfc_ram: [VolatileCell<u32>; 252],
    //rfc_ram: [u32; 252],
}

#[repr(C)]
pub struct PatchTab {
    patch_tab: [VolatileCell<u8>; 150],
}

#[repr(C)]
pub struct PTab {
    patch_tab: [u8; 150],
}

#[repr(C)]
pub struct IrqTab {
    irq_tab: [u32; 22],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Patches {
    patch_tab_offset: *mut PTab,
    irq_patch_offset: *mut IrqTab,
    patch_vec_offset: *mut CPERam,
}

impl Patches {
    pub const fn new() -> Patches {
        Patches {
            patch_tab_offset: 0x2100_0398 as *mut PTab,
            irq_patch_offset: 0x2100_0434 as *mut IrqTab,
            patch_vec_offset: 0x2100_4024 as *mut CPERam,
        }
    }

    pub fn apply_patch(&self) {
        self.enter_prop_cpe_patch();
        self.configure_prop_patch();
    }

    pub fn apply_patch_configure(&self) {
        self.configure_prop_patch();
    }

    fn configure_prop_patch(&self) {
        unsafe {
            (*self.patch_tab_offset).patch_tab[84] = 0;
            (*self.patch_tab_offset).patch_tab[142] = 1;
            (*self.patch_tab_offset).patch_tab[66] = 2;
            (*self.patch_tab_offset).patch_tab[102] = 3;
            (*self.patch_tab_offset).patch_tab[28] = 4;
            (*self.patch_tab_offset).patch_tab[104] = 5;
            (*self.patch_tab_offset).patch_tab[75] = 6;
            (*self.patch_tab_offset).patch_tab[73] = 7;
            (*self.patch_tab_offset).patch_tab[105] = 8;
            (*self.patch_tab_offset).patch_tab[106] = 9;
            (*self.patch_tab_offset).patch_tab[70] = 10;
            (*self.patch_tab_offset).patch_tab[71] = 11;
            (*self.patch_tab_offset).patch_tab[69] = 12;
            (*self.irq_patch_offset).irq_tab[21] = 0x210042ED;
        }
    }

    fn enter_prop_cpe_patch(&self) {
        let regs = unsafe { &*self.patch_vec_offset };
        let mut i = 0;
        
        for reg in regs.rfc_ram.iter() {
            reg.set(PATCH_IMAGE_PROP[i]);
            i += 1;
        }
        
        /*
        for i in 0..PATCH_IMAGE_PROP.len() {
            unsafe { (*self.patch_vec_offset).rfc_ram[i] = PATCH_IMAGE_PROP[i] }
        }
        */
    }
}
static PATCH_IMAGE_PROP: [u32; 252] = [
    0x21004105, 0x21004125, 0x2100419d, 0x21004245, 0x21004339, 0x2100408d, 0x210040b1, 0x210040b5,
    0x2100434d, 0x21004367, 0x210040c9, 0x210040e5, 0x210040f5, 0xb081b5ff, 0x9d0a4803, 0xb5f84700,
    0x48024684, 0x47004613, 0x00007f43, 0x00005145, 0x460cb5f7, 0x47084900, 0x0000681d, 0x4801b510,
    0x00004700, 0x000009df, 0x460db5f8, 0x4b0b4616, 0x290c6d59, 0x4b03d104, 0x78192408, 0x70194321,
    0x47084901, 0x21000340, 0x0000699d, 0xe0014a04, 0x3ac84a03, 0x21804801, 0x47106041, 0x40045000,
    0x000056cb, 0x49044803, 0x05c068c0, 0x47880fc0, 0x47084902, 0x21000340, 0x000087d1, 0x000053cd,
    0xf0002000, 0x4604f94d, 0x47004800, 0x0000545b, 0xf0002004, 0x4605f945, 0x47004800, 0x0000533b,
    0x4905b672, 0x22206808, 0x600a4302, 0x6ad24a03, 0xb6626008, 0x4770b250, 0x40040000, 0x40046040,
    0x4614b5f8, 0x9b06461a, 0x46139300, 0xf7ff4622, 0x9000ff91, 0xd12d2800, 0x6848495f, 0xd0292800,
    0x00498809, 0x43411b09, 0x68c0485c, 0x00640844, 0x20187922, 0xb2c61a80, 0x04802001, 0x0cc71808,
    0x46317965, 0xf0004856, 0x4855f949, 0x19832201, 0x408a1e69, 0x21182000, 0xe0071b8e, 0x19090041,
    0x437988c9, 0x40e91889, 0x1c405419, 0xdcf54286, 0x4780484c, 0xbdf89800, 0xf7ffb570, 0x4a46ff60,
    0x49492300, 0x60534604, 0x25136808, 0x01ed8800, 0x2e030b06, 0x0760d00d, 0x6808d43f, 0x290c7bc1,
    0xdc0fd028, 0xd0132904, 0xd0142905, 0xd10d290a, 0xb2c0e01d, 0xd0032806, 0x8c006808, 0xe02d8010,
    0xe02b8015, 0xd018290f, 0xd019291e, 0xe0048015, 0x01802013, 0x4835e000, 0x48308010, 0x29c068c1,
    0x29d8d010, 0x39ffd010, 0xd1173939, 0x20ffe00a, 0xe7f130e7, 0x309620ff, 0x20ffe7ee, 0xe7eb3045,
    0xe7e920a2, 0xe000492a, 0x6051492a, 0x60c1492a, 0x48232118, 0xf8e2f000, 0x8013e000, 0xbd704620,
    0x4604b5f8, 0x4e1c481d, 0x88003040, 0x0a80460d, 0xd00407c0, 0x08644821, 0x43447900, 0x8830e025,
    0xd0222800, 0x19000960, 0xff02f7ff, 0x20014607, 0x0240491b, 0x46024788, 0x1bc02005, 0x40848831,
    0x18230848, 0x62034817, 0x21016241, 0x430d61c1, 0x60cd490a, 0x07c969c1, 0x6a80d1fc, 0x60704910,
    0x39124610, 0x46384788, 0x2000bdf8, 0x46206070, 0xfedef7ff, 0x0000bdf8, 0x2100440c, 0x21000028,
    0x21000000, 0x0000764d, 0x21000108, 0x000003cd, 0x00063b91, 0x0003fd29, 0x000090fd, 0x21000340,
    0x000040e5, 0x40044100, 0x480eb570, 0x4c0e6ac0, 0x0f000700, 0x28012502, 0x2804d005, 0x280cd00b,
    0x280dd001, 0x6960d106, 0x616043a8, 0x49072001, 0x60080280, 0x4906bd70, 0x47882001, 0x43286960,
    0xbd706160, 0x40046000, 0x40041100, 0xe000e180, 0x00007d05, 0x4c03b510, 0xfea0f7ff, 0x28006820,
    0xbd10d1fa, 0x40041100, 0x780a490b, 0xd1042aff, 0x7ad24a0a, 0x0f120712, 0x4908700a, 0x75883140,
    0x49054770, 0x29ff7809, 0x0900d005, 0x43080100, 0x31404902, 0x47707588, 0x210002a5, 0x40086200,
    0x4c19b570, 0x7ba14606, 0xf820f000, 0x7be14605, 0xf0004630, 0x4915f81b, 0x78094604, 0x070a2028,
    0x2d01d401, 0x2038d100, 0xd40106c9, 0xd1012c01, 0x43082140, 0x4788490e, 0xd0012dff, 0x6145480d,
    0xd0012cff, 0x61c4480c, 0xbd704808, 0xd0082900, 0xd00629ff, 0x070840c1, 0x281c0ec0, 0x2001d100,
    0x20ff4770, 0x00004770, 0x210000a8, 0x21000340, 0x000040e5, 0x40045040, 0x40046000, 0x4801b403,
    0xbd019001, 0x000089dd, 0x00000000, 0x00000000,
];
