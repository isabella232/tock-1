pub const CPE_PATCH: Patches = Patches::new();

#[repr(C)]
pub struct CPERam {
    rfc_ram: [u32; 54],
}

#[repr(C)]
pub struct PTab {
    patch_tab: [u8; 150],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Patches {
    patch_tab_offset: *mut PTab,
    patch_vec_offset: *mut CPERam,
}

impl Patches {
    pub const fn new() -> Patches {
        Patches {
            patch_tab_offset: 0x2100_03D4 as *mut PTab,
            patch_vec_offset: 0x2100_404C as *mut CPERam,
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
            (*self.patch_tab_offset).patch_tab[76] = 0;
            (*self.patch_tab_offset).patch_tab[62] = 1;
            (*self.patch_tab_offset).patch_tab[64] = 2;
            (*self.patch_tab_offset).patch_tab[91] = 3;
        }
    }

    fn enter_prop_cpe_patch(&self) {
        for i in 0..PATCH_IMAGE_PROP.len() {
            unsafe { (*self.patch_vec_offset).rfc_ram[i] = PATCH_IMAGE_PROP[i] }
        }
    }
}

static PATCH_IMAGE_PROP: [u32; 54] = [
    0x2100405d, 0x210040c7, 0x21004089, 0x210040e9, 0x79654c07, 0xf809f000, 0x40697961, 0xd5030749,
    0x4a042101, 0x60110389, 0xb570bd70, 0x47084902, 0x21000380, 0x40041108, 0x0000592d, 0xf819f000,
    0x296cb2e1, 0x2804d00b, 0x2806d001, 0x490ed107, 0x07c97809, 0x7821d103, 0xd4000709, 0x490b2002,
    0x210c780a, 0xd0024211, 0x22804909, 0xb003600a, 0xb5f0bdf0, 0x4907b083, 0x48044708, 0x22407801,
    0x70014391, 0x47004804, 0x210000c8, 0x21000133, 0xe000e200, 0x00031641, 0x00031b23, 0x490cb510,
    0x4a0c4788, 0x5e512106, 0xd0072900, 0xd0052902, 0xd0032909, 0xd0012910, 0xd1072911, 0x43c92177,
    0xdd014288, 0xdd012800, 0x43c0207f, 0x0000bd10, 0x000065a9, 0x21000380,
];
