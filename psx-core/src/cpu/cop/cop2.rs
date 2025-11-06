use crate::{cpu::cop::Cop, gteidx};

#[derive(Clone, Copy)]
pub struct Cop2 {
    registers: [u32; 64],
}

impl Cop2 {
    pub fn new() -> Self {
        Self {
            registers: [0; 64],
        }
    }

    #[inline(always)]
    pub fn v0_xy(&self) -> (i16, i16) {
        let xy = self.registers[gteidx("v0_xy") as usize];
        ((xy & 0xFFFF) as i16, (xy >> 16) as i16)
    }

    #[inline(always)]
    pub fn v0_z(&self) -> i16 {
        self.registers[gteidx("v0_z") as usize] as i16
    }

    #[inline(always)]
    pub fn v1_xy(&self) -> (i16, i16) {
        let xy = self.registers[gteidx("v1_xy") as usize];
        ((xy & 0xFFFF) as i16, (xy >> 16) as i16)
    }

    #[inline(always)]
    pub fn v1_z(&self) -> i16 {
        self.registers[gteidx("v1_z") as usize] as i16
    }

    #[inline(always)]
    pub fn v2_xy(&self) -> (i16, i16) {
        let xy = self.registers[gteidx("v2_xy") as usize];
        ((xy & 0xFFFF) as i16, (xy >> 16) as i16)
    }

    #[inline(always)]
    pub fn v2_z(&self) -> i16 {
        self.registers[gteidx("v2_z") as usize] as i16
    }

    #[inline(always)]
    pub fn rgbc(&self) -> (u8, u8, u8, u8) {
        let rgbc = self.registers[gteidx("rgbc") as usize];
        (
            (rgbc & 0xFF) as u8,
            ((rgbc >> 8) & 0xFF) as u8,
            ((rgbc >> 16) & 0xFF) as u8,
            ((rgbc >> 24) & 0xFF) as u8,
        )
    }

    #[inline(always)]
    pub fn otz(&self) -> u16 {
        self.registers[gteidx("otz") as usize] as u16
    }

    #[inline(always)]
    pub fn ir0(&self) -> i16 {
        self.registers[gteidx("ir0") as usize] as i16
    }

    #[inline(always)]
    pub fn ir1(&self) -> i16 {
        self.registers[gteidx("ir1") as usize] as i16
    }

    #[inline(always)]
    pub fn ir2(&self) -> i16 {
        self.registers[gteidx("ir2") as usize] as i16
    }

    #[inline(always)]
    pub fn ir3(&self) -> i16 {
        self.registers[gteidx("ir3") as usize] as i16
    }

    pub fn sxy0(&self) -> (i16, i16) {
        let sxy0 = self.registers[gteidx("sxy0") as usize];
        ((sxy0 & 0xFFFF) as i16, (sxy0 >> 16) as i16)
    }

    pub fn sxy1(&self) -> (i16, i16) {
        let sxy1 = self.registers[gteidx("sxy1") as usize];
        ((sxy1 & 0xFFFF) as i16, (sxy1 >> 16) as i16)
    }

    pub fn sxy2(&self) -> (i16, i16) {
        let sxy2 = self.registers[gteidx("sxy2") as usize];
        ((sxy2 & 0xFFFF) as i16, (sxy2 >> 16) as i16)
    }

    pub fn sxyp(&self) -> (i16, i16) {
        let sxyp = self.registers[gteidx("sxyp") as usize];
        ((sxyp & 0xFFFF) as i16, (sxyp >> 16) as i16)
    }

    pub fn sz0(&self) -> u16 {
        self.registers[gteidx("sz0") as usize] as u16
    }

    pub fn sz1(&self) -> u16 {
        self.registers[gteidx("sz1") as usize] as u16
    }

    pub fn sz2(&self) -> u16 {
        self.registers[gteidx("sz2") as usize] as u16
    }

    pub fn sz3(&self) -> u16 {
        self.registers[gteidx("sz3") as usize] as u16
    }

    pub fn rgb0(&self) -> (u8, u8, u8, u8) {
        let rgb0 = self.registers[gteidx("rgb0") as usize];
        (
            (rgb0 & 0xFF) as u8,
            ((rgb0 >> 8) & 0xFF) as u8,
            ((rgb0 >> 16) & 0xFF) as u8,
            ((rgb0 >> 24) & 0xFF) as u8,
        )
    }

    pub fn rgb1(&self) -> (u8, u8, u8, u8) {
        let rgb1 = self.registers[gteidx("rgb1") as usize];
        (
            (rgb1 & 0xFF) as u8,
            ((rgb1 >> 8) & 0xFF) as u8,
            ((rgb1 >> 16) & 0xFF) as u8,
            ((rgb1 >> 24) & 0xFF) as u8,
        )
    }

    pub fn rgb2(&self) -> (u8, u8, u8, u8) {
        let rgb2 = self.registers[gteidx("rgb2") as usize];
        (
            (rgb2 & 0xFF) as u8,
            ((rgb2 >> 8) & 0xFF) as u8,
            ((rgb2 >> 16) & 0xFF) as u8,
            ((rgb2 >> 24) & 0xFF) as u8,
        )
    }

    pub fn mac0(&self) -> i32 {
        self.registers[gteidx("mac0") as usize] as i32
    }

    pub fn mac1(&self) -> i32 {
        self.registers[gteidx("mac1") as usize] as i32
    }

    pub fn mac2(&self) -> i32 {
        self.registers[gteidx("mac2") as usize] as i32
    }

    pub fn mac3(&self) -> i32 {
        self.registers[gteidx("mac3") as usize] as i32
    }

    pub fn irgb(&self) -> u16 {
        let irgb = self.registers[gteidx("irgb") as usize];
        (irgb & 0xFFFF) as u16
    }

    pub fn orgb(&self) -> u16 {
        let orgb = self.registers[gteidx("orgb") as usize];
        (orgb & 0xFFFF) as u16
    }

    pub fn lzcs(&self) -> i32 {
        self.registers[gteidx("lzcs") as usize] as i32
    }

    pub fn lzcr(&self) -> i32 {
        self.registers[gteidx("lzcr") as usize] as i32
    }

    #[inline(always)]
    pub fn r11r12(&self) -> (i16, i16) {
        let val = self.registers[gteidx("r11r12") as usize];
        ((val & 0xFFFF) as i16, (val >> 16) as i16)
    }

    #[inline(always)]
    pub fn r13r21(&self) -> (i16, i16) {
        let val = self.registers[gteidx("r13r21") as usize];
        ((val & 0xFFFF) as i16, (val >> 16) as i16)
    }

    #[inline(always)]
    pub fn r22r23(&self) -> (i16, i16) {
        let val = self.registers[gteidx("r22r23") as usize];
        ((val & 0xFFFF) as i16, (val >> 16) as i16)
    }

    #[inline(always)]
    pub fn r31r32(&self) -> (i16, i16) {
        let val = self.registers[gteidx("r31r32") as usize];
        ((val & 0xFFFF) as i16, (val >> 16) as i16)
    }

    #[inline(always)]
    pub fn r33(&self) -> i16 {
        self.registers[gteidx("r33") as usize] as i16
    }

    #[inline(always)]
    pub fn tr_x(&self) -> i32 {
        self.registers[gteidx("trx") as usize] as i32
    }

    #[inline(always)]
    pub fn tr_y(&self) -> i32 {
        self.registers[gteidx("try") as usize] as i32
    }

    #[inline(always)]
    pub fn tr_z(&self) -> i32 {
        self.registers[gteidx("trz") as usize] as i32
    }

    #[inline(always)]
    pub fn l11l12(&self) -> (i16, i16) {
        let val = self.registers[gteidx("l11l12") as usize];
        ((val & 0xFFFF) as i16, (val >> 16) as i16)
    }

    #[inline(always)]
    pub fn l13l21(&self) -> (i16, i16) {
        let val = self.registers[gteidx("l13l21") as usize];
        ((val & 0xFFFF) as i16, (val >> 16) as i16)
    }

    #[inline(always)]
    pub fn l22l23(&self) -> (i16, i16) {
        let val = self.registers[gteidx("l22l23") as usize];
        ((val & 0xFFFF) as i16, (val >> 16) as i16)
    }

    #[inline(always)]
    pub fn l31l32(&self) -> (i16, i16) {
        let val = self.registers[gteidx("l31l32") as usize];
        ((val & 0xFFFF) as i16, (val >> 16) as i16)
    }

    #[inline(always)]
    pub fn l33(&self) -> i16 {
        self.registers[gteidx("l33") as usize] as i16
    }

    #[inline(always)]
    pub fn rbk(&self) -> i32 {
        self.registers[gteidx("rbk") as usize] as i32
    }

    #[inline(always)]
    pub fn gbk(&self) -> i32 {
        self.registers[gteidx("gbk") as usize] as i32
    }

    #[inline(always)]
    pub fn bbk(&self) -> i32 {
        self.registers[gteidx("bbk") as usize] as i32
    }

    #[inline(always)]
    pub fn lr1lr2(&self) -> (i16, i16) {
        let val = self.registers[gteidx("lr1lr2") as usize];
        ((val & 0xFFFF) as i16, (val >> 16) as i16)
    }

    #[inline(always)]
    pub fn lr3lg1(&self) -> (i16, i16) {
        let val = self.registers[gteidx("lr3lg1") as usize];
        ((val & 0xFFFF) as i16, (val >> 16) as i16)
    }

    #[inline(always)]
    pub fn lg2lg3(&self) -> (i16, i16) {
        let val = self.registers[gteidx("lg2lg3") as usize];
        ((val & 0xFFFF) as i16, (val >> 16) as i16)
    }

    #[inline(always)]
    pub fn lb1lb2(&self) -> (i16, i16) {
        let val = self.registers[gteidx("lb1lb2") as usize];
        ((val & 0xFFFF) as i16, (val >> 16) as i16)
    }

    #[inline(always)]
    pub fn lb3(&self) -> i16 {
        self.registers[gteidx("lb3") as usize] as i16
    }

    #[inline(always)]
    pub fn rfc(&self) -> i32 {
        self.registers[gteidx("rfc") as usize] as i32
    }

    #[inline(always)]
    pub fn gfc(&self) -> i32 {
        self.registers[gteidx("gfc") as usize] as i32
    }

    #[inline(always)]
    pub fn bfc(&self) -> i32 {
        self.registers[gteidx("bfc") as usize] as i32
    }

    #[inline(always)]
    pub fn ofx(&self) -> i32 {
        self.registers[gteidx("ofx") as usize] as i32
    }

    #[inline(always)]
    pub fn ofy(&self) -> i32 {
        self.registers[gteidx("ofy") as usize] as i32
    }

    #[inline(always)]
    pub fn h(&self) -> u16 {
        self.registers[gteidx("h") as usize] as u16
    }

    #[inline(always)]
    pub fn dqa(&self) -> i16 {
        self.registers[gteidx("dqa") as usize] as i16
    }

    #[inline(always)]
    pub fn dqb(&self) -> i32 {
        self.registers[gteidx("dqb") as usize] as i32
    }

    #[inline(always)]
    pub fn zsf3(&self) -> i16 {
        self.registers[gteidx("zsf3") as usize] as i16
    }

    #[inline(always)]
    pub fn zsf4(&self) -> i16 {
        self.registers[gteidx("zsf4") as usize] as i16
    }

    #[inline(always)]
    pub fn flag(&self) -> u32 {
        self.registers[gteidx("flag") as usize]
    }

    #[inline(always)]
    pub fn set_v0_xy(&mut self, x: i16, y: i16) {
        self.registers[gteidx("v0_xy") as usize] = ((x as u32) & 0xFFFF) | (((y as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_v0_z(&mut self, z: i16) {
        self.registers[gteidx("v0_z") as usize] = z as u32;
    }

    #[inline(always)]
    pub fn set_v1_xy(&mut self, x: i16, y: i16) {
        self.registers[gteidx("v1_xy") as usize] = ((x as u32) & 0xFFFF) | (((y as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_v1_z(&mut self, z: i16) {
        self.registers[gteidx("v1_z") as usize] = z as u32;
    }

    #[inline(always)]
    pub fn set_v2_xy(&mut self, x: i16, y: i16) {
        self.registers[gteidx("v2_xy") as usize] = ((x as u32) & 0xFFFF) | (((y as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_v2_z(&mut self, z: i16) {
        self.registers[gteidx("v2_z") as usize] = z as u32;
    }

    #[inline(always)]
    pub fn set_rgbc(&mut self, r: u8, g: u8, b: u8, c: u8) {
        self.registers[gteidx("rgbc") as usize] =
            (r as u32) | ((g as u32) << 8) | ((b as u32) << 16) | ((c as u32) << 24);
    }

    #[inline(always)]
    pub fn set_otz(&mut self, value: u16) {
        self.registers[gteidx("otz") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_ir0(&mut self, value: i16) {
        self.registers[gteidx("ir0") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_ir1(&mut self, value: i16) {
        self.registers[gteidx("ir1") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_ir2(&mut self, value: i16) {
        self.registers[gteidx("ir2") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_ir3(&mut self, value: i16) {
        self.registers[gteidx("ir3") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_sxy0(&mut self, x: i16, y: i16) {
        self.registers[gteidx("sxy0") as usize] = ((x as u32) & 0xFFFF) | (((y as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_sxy1(&mut self, x: i16, y: i16) {
        self.registers[gteidx("sxy1") as usize] = ((x as u32) & 0xFFFF) | (((y as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_sxy2(&mut self, x: i16, y: i16) {
        self.registers[gteidx("sxy2") as usize] = ((x as u32) & 0xFFFF) | (((y as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_sxyp(&mut self, x: i16, y: i16) {
        self.registers[gteidx("sxyp") as usize] = ((x as u32) & 0xFFFF) | (((y as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_sz0(&mut self, value: u16) {
        self.registers[gteidx("sz0") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_sz1(&mut self, value: u16) {
        self.registers[gteidx("sz1") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_sz2(&mut self, value: u16) {
        self.registers[gteidx("sz2") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_sz3(&mut self, value: u16) {
        self.registers[gteidx("sz3") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_rgb0(&mut self, r: u8, g: u8, b: u8, c: u8) {
        self.registers[gteidx("rgb0") as usize] =
            (r as u32) | ((g as u32) << 8) | ((b as u32) << 16) | ((c as u32) << 24);
    }

    #[inline(always)]
    pub fn set_rgb1(&mut self, r: u8, g: u8, b: u8, c: u8) {
        self.registers[gteidx("rgb1") as usize] =
            (r as u32) | ((g as u32) << 8) | ((b as u32) << 16) | ((c as u32) << 24);
    }

    #[inline(always)]
    pub fn set_rgb2(&mut self, r: u8, g: u8, b: u8, c: u8) {
        self.registers[gteidx("rgb2") as usize] =
            (r as u32) | ((g as u32) << 8) | ((b as u32) << 16) | ((c as u32) << 24);
    }

    #[inline(always)]
    pub fn set_mac0(&mut self, value: i32) {
        self.registers[gteidx("mac0") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_mac1(&mut self, value: i32) {
        self.registers[gteidx("mac1") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_mac2(&mut self, value: i32) {
        self.registers[gteidx("mac2") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_mac3(&mut self, value: i32) {
        self.registers[gteidx("mac3") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_irgb(&mut self, value: u16) {
        self.registers[gteidx("irgb") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_orgb(&mut self, value: u16) {
        self.registers[gteidx("orgb") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_lzcs(&mut self, value: i32) {
        self.registers[gteidx("lzcs") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_lzcr(&mut self, value: i32) {
        self.registers[gteidx("lzcr") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_r11r12(&mut self, r11: i16, r12: i16) {
        self.registers[gteidx("r11r12") as usize] = ((r11 as u32) & 0xFFFF) | (((r12 as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_r13r21(&mut self, r13: i16, r21: i16) {
        self.registers[gteidx("r13r21") as usize] = ((r13 as u32) & 0xFFFF) | (((r21 as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_r22r23(&mut self, r22: i16, r23: i16) {
        self.registers[gteidx("r22r23") as usize] = ((r22 as u32) & 0xFFFF) | (((r23 as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_r31r32(&mut self, r31: i16, r32: i16) {
        self.registers[gteidx("r31r32") as usize] = ((r31 as u32) & 0xFFFF) | (((r32 as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_r33(&mut self, value: i16) {
        self.registers[gteidx("r33") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_tr_x(&mut self, value: i32) {
        self.registers[gteidx("trx") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_tr_y(&mut self, value: i32) {
        self.registers[gteidx("try") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_tr_z(&mut self, value: i32) {
        self.registers[gteidx("trz") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_l11l12(&mut self, l11: i16, l12: i16) {
        self.registers[gteidx("l11l12") as usize] = ((l11 as u32) & 0xFFFF) | (((l12 as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_l13l21(&mut self, l13: i16, l21: i16) {
        self.registers[gteidx("l13l21") as usize] = ((l13 as u32) & 0xFFFF) | (((l21 as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_l22l23(&mut self, l22: i16, l23: i16) {
        self.registers[gteidx("l22l23") as usize] = ((l22 as u32) & 0xFFFF) | (((l23 as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_l31l32(&mut self, l31: i16, l32: i16) {
        self.registers[gteidx("l31l32") as usize] = ((l31 as u32) & 0xFFFF) | (((l32 as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_l33(&mut self, value: i16) {
        self.registers[gteidx("l33") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_rbk(&mut self, value: i32) {
        self.registers[gteidx("rbk") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_gbk(&mut self, value: i32) {
        self.registers[gteidx("gbk") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_bbk(&mut self, value: i32) {
        self.registers[gteidx("bbk") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_lr1lr2(&mut self, lr1: i16, lr2: i16) {
        self.registers[gteidx("lr1lr2") as usize] = ((lr1 as u32) & 0xFFFF) | (((lr2 as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_lr3lg1(&mut self, lr3: i16, lg1: i16) {
        self.registers[gteidx("lr3lg1") as usize] = ((lr3 as u32) & 0xFFFF) | (((lg1 as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_lg2lg3(&mut self, lg2: i16, lg3: i16) {
        self.registers[gteidx("lg2lg3") as usize] = ((lg2 as u32) & 0xFFFF) | (((lg3 as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_lb1lb2(&mut self, lb1: i16, lb2: i16) {
        self.registers[gteidx("lb1lb2") as usize] = ((lb1 as u32) & 0xFFFF) | (((lb2 as u32) & 0xFFFF) << 16);
    }

    #[inline(always)]
    pub fn set_lb3(&mut self, value: i16) {
        self.registers[gteidx("lb3") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_rfc(&mut self, value: i32) {
        self.registers[gteidx("rfc") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_gfc(&mut self, value: i32) {
        self.registers[gteidx("gfc") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_bfc(&mut self, value: i32) {
        self.registers[gteidx("bfc") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_ofx(&mut self, value: i32) {
        self.registers[gteidx("ofx") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_ofy(&mut self, value: i32) {
        self.registers[gteidx("ofy") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_h(&mut self, value: u16) {
        self.registers[gteidx("h") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_dqa(&mut self, value: i16) {
        self.registers[gteidx("dqa") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_dqb(&mut self, value: i32) {
        self.registers[gteidx("dqb") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_zsf3(&mut self, value: i16) {
        self.registers[gteidx("zsf3") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_zsf4(&mut self, value: i16) {
        self.registers[gteidx("zsf4") as usize] = value as u32;
    }

    #[inline(always)]
    pub fn set_flag(&mut self, value: u32) {
        self.registers[gteidx("flag") as usize] = value;
    }
}

impl Cop for Cop2 {
    #[inline(always)]
    fn read_register(&self, register: u8) -> u32 {
        self.registers[register as usize]
    }

    #[inline(always)]
    fn write_register(&mut self, register: u8, value: u32) {
        self.registers[register as usize] = value;
    }
}
