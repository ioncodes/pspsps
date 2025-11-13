use crate::cpu::cop::Cop;
use crate::gteidx;

#[derive(Clone, Copy)]
pub struct Cop2 {
    registers: [u32; 64],
}

impl Cop2 {
    pub fn new() -> Self {
        Self { registers: [0; 64] }
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

    #[inline(always)]
    pub fn read_data_register(&self, register: u8) -> u32 {
        self.read_register(register)
    }

    #[inline(always)]
    pub fn write_data_register(&mut self, register: u8, value: u32) {
        self.write_register(register, value);
    }

    #[inline(always)]
    pub fn read_control_register(&self, register: u8) -> u32 {
        self.read_register(register + 32)
    }

    #[inline(always)]
    pub fn write_control_register(&mut self, register: u8, value: u32) {
        self.write_register(register + 32, value);
    }
}

impl Cop for Cop2 {
    #[inline(always)]
    fn read_register(&self, register: u8) -> u32 {
        match register {
            // Sign-extend 16-bit values on read
            1 | 3 | 5 | 9 | 10 | 11 => (self.registers[register as usize] & 0xFFFF) as i16 as i32 as u32,
            // SXYP reads SXY2
            15 => self.registers[14],
            // H sign-extends on read
            58 => (self.registers[58] & 0xFFFF) as i16 as i32 as u32,
            // IRGB/ORGB collapse IR1/IR2/IR3 to 5:5:5 RGB
            28 | 29 => {
                let mut red = self.registers[9] & 0xFFFF;
                let mut green = self.registers[10] & 0xFFFF;
                let mut blue = self.registers[11] & 0xFFFF;

                // Clamp negative values to 0
                if (red as i16) < 0 {
                    red = 0;
                }
                if (green as i16) < 0 {
                    green = 0;
                }
                if (blue as i16) < 0 {
                    blue = 0;
                }

                // Clamp to max 0xF80
                if (red as i16) > 0xF80 {
                    red = 0xF80;
                }
                if (green as i16) > 0xF80 {
                    green = 0xF80;
                }
                if (blue as i16) > 0xF80 {
                    blue = 0xF80;
                }

                // Convert to 5:5:5
                red >>= 7;
                green >>= 7;
                blue >>= 7;

                red | (green << 5) | (blue << 10)
            }
            // FLAG sets bit 31 if any error bits are set
            63 => {
                let flag = self.registers[63];
                if (flag & 0x7F87E000) != 0 {
                    flag | 0x80000000
                } else {
                    flag
                }
            }
            _ => self.registers[register as usize],
        }
    }

    #[inline(always)]
    fn write_register(&mut self, register: u8, value: u32) {
        match register {
            // Data registers (0-31)
            0 => self.registers[0] = value,                      // v0_xy (2x i16)
            1 => self.registers[1] = value as i16 as i32 as u32, // v0_z (1x i16, sign-extend)
            2 => self.registers[2] = value,                      // v1_xy (2x i16)
            3 => self.registers[3] = value as i16 as i32 as u32, // v1_z (1x i16, sign-extend)
            4 => self.registers[4] = value,                      // v2_xy (2x i16)
            5 => self.registers[5] = value as i16 as i32 as u32, // v2_z (1x i16, sign-extend)
            6 => self.registers[6] = value,                      // rgbc (4x u8)
            7 => self.registers[7] = value & 0xFFFF,             // otz (1x u16, zero-extend)
            8 => self.registers[8] = value as i16 as i32 as u32, // ir0 (1x i16, sign-extend)
            9 => self.registers[9] = value & 0xFFFF,             // ir1 (1x i16, zero-extend)
            10 => self.registers[10] = value & 0xFFFF,           // ir2 (1x i16, zero-extend)
            11 => self.registers[11] = value & 0xFFFF,           // ir3 (1x i16, zero-extend)
            12 => self.registers[12] = value,                    // sxy0 (2x i16)
            13 => self.registers[13] = value,                    // sxy1 (2x i16)
            14 => self.registers[14] = value,                    // sxy2 (2x i16)
            15 => {
                // sxyp - push through FIFO
                let sxy2 = self.registers[14];
                let sxy1 = self.registers[13];
                self.registers[12] = sxy1; // sxy0 = old sxy1
                self.registers[13] = sxy2; // sxy1 = old sxy2
                self.registers[14] = value; // sxy2 = new value
            }
            16 => self.registers[16] = value & 0xFFFF, // sz0 (1x u16, zero-extend)
            17 => self.registers[17] = value & 0xFFFF, // sz1 (1x u16, zero-extend)
            18 => self.registers[18] = value & 0xFFFF, // sz2 (1x u16, zero-extend)
            19 => self.registers[19] = value & 0xFFFF, // sz3 (1x u16, zero-extend)
            20 => self.registers[20] = value,          // rgb0 (4x u8)
            21 => self.registers[21] = value,          // rgb1 (4x u8)
            22 => self.registers[22] = value,          // rgb2 (4x u8)
            23 => self.registers[23] = value,          // res1 (prohibited)
            24 => self.registers[24] = value,          // mac0 (1x i32)
            25 => self.registers[25] = value,          // mac1 (1x i32)
            26 => self.registers[26] = value,          // mac2 (1x i32)
            27 => self.registers[27] = value,          // mac3 (1x i32)
            28 => {
                // irgb - expand 5:5:5 RGB to IR1/IR2/IR3
                let red = (value & 0x1F) * 0x80;
                let green = ((value >> 5) & 0x1F) * 0x80;
                let blue = ((value >> 10) & 0x1F) * 0x80;
                self.registers[9] = red; // IR1
                self.registers[10] = green; // IR2
                self.registers[11] = blue; // IR3
            }
            29 => {} // orgb is read-only
            30 => {
                // lzcs - calculate leading zeros/ones
                self.registers[30] = value;
                let mut v = value;
                if (v & 0x80000000) == 0 {
                    v = !v;
                }
                let mut zeros = 0;
                while (v & 0x80000000) != 0 {
                    zeros += 1;
                    v <<= 1;
                }
                self.registers[31] = zeros; // lzcr
            }
            31 => {} // lzcr is read-only
            // Control registers (32-63)
            32 => self.registers[32] = value,                      // r11r12 (2x i16)
            33 => self.registers[33] = value,                      // r13r21 (2x i16)
            34 => self.registers[34] = value,                      // r22r23 (2x i16)
            35 => self.registers[35] = value,                      // r31r32 (2x i16)
            36 => self.registers[36] = value as i16 as i32 as u32, // r33 (1x i16, sign-extend)
            37 => self.registers[37] = value,                      // trx (1x i32)
            38 => self.registers[38] = value,                      // try (1x i32)
            39 => self.registers[39] = value,                      // trz (1x i32)
            40 => self.registers[40] = value,                      // l11l12 (2x i16)
            41 => self.registers[41] = value,                      // l13l21 (2x i16)
            42 => self.registers[42] = value,                      // l22l23 (2x i16)
            43 => self.registers[43] = value,                      // l31l32 (2x i16)
            44 => self.registers[44] = value as i16 as i32 as u32, // l33 (1x i16, sign-extend)
            45 => self.registers[45] = value,                      // rbk (1x i32)
            46 => self.registers[46] = value,                      // gbk (1x i32)
            47 => self.registers[47] = value,                      // bbk (1x i32)
            48 => self.registers[48] = value,                      // lr1lr2 (2x i16)
            49 => self.registers[49] = value,                      // lr3lg1 (2x i16)
            50 => self.registers[50] = value,                      // lg2lg3 (2x i16)
            51 => self.registers[51] = value,                      // lb1lb2 (2x i16)
            52 => self.registers[52] = value as i16 as i32 as u32, // lb3 (1x i16, sign-extend)
            53 => self.registers[53] = value,                      // rfc (1x i32)
            54 => self.registers[54] = value,                      // gfc (1x i32)
            55 => self.registers[55] = value,                      // bfc (1x i32)
            56 => self.registers[56] = value,                      // ofx (1x i32)
            57 => self.registers[57] = value,                      // ofy (1x i32)
            58 => self.registers[58] = value & 0xFFFF,             // h (1x u16, zero-extend on write)
            59 => self.registers[59] = value as i16 as i32 as u32, // dqa (1x i16, sign-extend)
            60 => self.registers[60] = value,                      // dqb (1x i32)
            61 => self.registers[61] = value as i16 as i32 as u32, // zsf3 (1x i16, sign-extend)
            62 => self.registers[62] = value as i16 as i32 as u32, // zsf4 (1x i16, sign-extend)
            63 => {
                // flag - preserve bits 0-11 and bit 31, write bits 12-30
                self.registers[63] = (self.registers[63] & 0x80000FFF) | (value & 0x7FFFF000);
            }
            _ => panic!("Invalid GTE register index: {}", register),
        }
    }
}
