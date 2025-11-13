use crate::cpu::Cpu;
use crate::cpu::decoder::Instruction;

// UNR table for division operations - 257 entries
static UNR_TABLE: [u16; 257] = [
    0xFF, 0xFD, 0xFB, 0xF9, 0xF7, 0xF5, 0xF3, 0xF1, 0xEF, 0xEE, 0xEC, 0xEA, 0xE8, 0xE6, 0xE4, 0xE3, 0xE1, 0xDF, 0xDD,
    0xDC, 0xDA, 0xD8, 0xD6, 0xD5, 0xD3, 0xD1, 0xD0, 0xCE, 0xCD, 0xCB, 0xC9, 0xC8, 0xC6, 0xC5, 0xC3, 0xC1, 0xC0, 0xBE,
    0xBD, 0xBB, 0xBA, 0xB8, 0xB7, 0xB5, 0xB4, 0xB2, 0xB1, 0xB0, 0xAE, 0xAD, 0xAB, 0xAA, 0xA9, 0xA7, 0xA6, 0xA4, 0xA3,
    0xA2, 0xA0, 0x9F, 0x9E, 0x9C, 0x9B, 0x9A, 0x99, 0x97, 0x96, 0x95, 0x94, 0x92, 0x91, 0x90, 0x8F, 0x8D, 0x8C, 0x8B,
    0x8A, 0x89, 0x87, 0x86, 0x85, 0x84, 0x83, 0x82, 0x81, 0x7F, 0x7E, 0x7D, 0x7C, 0x7B, 0x7A, 0x79, 0x78, 0x77, 0x75,
    0x74, 0x73, 0x72, 0x71, 0x70, 0x6F, 0x6E, 0x6D, 0x6C, 0x6B, 0x6A, 0x69, 0x68, 0x67, 0x66, 0x65, 0x64, 0x63, 0x62,
    0x61, 0x60, 0x5F, 0x5E, 0x5D, 0x5D, 0x5C, 0x5B, 0x5A, 0x59, 0x58, 0x57, 0x56, 0x55, 0x54, 0x53, 0x53, 0x52, 0x51,
    0x50, 0x4F, 0x4E, 0x4D, 0x4D, 0x4C, 0x4B, 0x4A, 0x49, 0x48, 0x48, 0x47, 0x46, 0x45, 0x44, 0x43, 0x43, 0x42, 0x41,
    0x40, 0x3F, 0x3F, 0x3E, 0x3D, 0x3C, 0x3C, 0x3B, 0x3A, 0x39, 0x39, 0x38, 0x37, 0x36, 0x36, 0x35, 0x34, 0x33, 0x33,
    0x32, 0x31, 0x31, 0x30, 0x2F, 0x2E, 0x2E, 0x2D, 0x2C, 0x2C, 0x2B, 0x2A, 0x2A, 0x29, 0x28, 0x28, 0x27, 0x26, 0x26,
    0x25, 0x24, 0x24, 0x23, 0x22, 0x22, 0x21, 0x20, 0x20, 0x1F, 0x1E, 0x1E, 0x1D, 0x1D, 0x1C, 0x1B, 0x1B, 0x1A, 0x19,
    0x19, 0x18, 0x18, 0x17, 0x16, 0x16, 0x15, 0x15, 0x14, 0x14, 0x13, 0x12, 0x12, 0x11, 0x11, 0x10, 0x0F, 0x0F, 0x0E,
    0x0E, 0x0D, 0x0D, 0x0C, 0x0C, 0x0B, 0x0A, 0x0A, 0x09, 0x09, 0x08, 0x08, 0x07, 0x07, 0x06, 0x06, 0x05, 0x05, 0x04,
    0x04, 0x03, 0x03, 0x02, 0x02, 0x01, 0x01, 0x00, 0x00, 0x00,
];

// Count leading zeros for a given value with a specific bit count
fn count_leading_zeros(value: u32, bit_count: u32) -> u32 {
    let mut count = 0;
    for i in 0..bit_count {
        let mask = 1 << ((bit_count - 1) - i);
        if (value & mask) != 0 {
            break;
        }
        count += 1;
    }
    count
}

// GTE division with UNR table
fn gte_divide(cpu: &mut Cpu, numerator: u32, denominator: u32) -> u32 {
    if numerator >= denominator.saturating_mul(2) {
        // Division overflow
        let flag = cpu.cop2.flag();
        cpu.cop2.set_flag(flag | (1 << 17));
        return 0x1FFFF;
    }

    let shift = count_leading_zeros(denominator, 16);
    let r1 = ((denominator << shift) & 0x7FFF) as u16;
    let r2 = (UNR_TABLE[((r1 + 0x40) >> 7) as usize] + 0x101) as u32;
    let r3 = ((0x80u32.wrapping_sub(r2 * ((r1 as u32) + 0x8000))) >> 8) & 0x1FFFF;
    let reciprocal = ((r2 * r3) + 0x80) >> 8;

    let mut res = ((reciprocal as u64 * ((numerator << shift) as u64) + 0x8000) >> 16) as u32;

    // Some divisions result in values > 0x1FFFF but are saturated without setting FLAG
    if res > 0x1FFFF {
        let flag = cpu.cop2.flag();
        cpu.cop2.set_flag(flag | (1 << 17));
        res = 0x1FFFF;
    }

    res.min(0x1FFFF)
}

// Set MAC flag for 44-bit overflow detection
fn set_mac_flag(cpu: &mut Cpu, mac_index: usize, result: i64) {
    let flag = cpu.cop2.flag();

    let (positive_bit, negative_bit) = match mac_index {
        0 => (16, 15),
        1 => (30, 27),
        2 => (29, 26),
        3 => (28, 25),
        _ => return,
    };

    let mut new_flag = flag;

    if mac_index != 0 {
        // MAC1-3: check 44-bit overflow
        if result < -0x800_0000_0000i64 {
            new_flag |= 1 << negative_bit;
        } else if result > 0x7FF_FFFF_FFFFi64 {
            new_flag |= 1 << positive_bit;
        }
    } else {
        // MAC0: check 32-bit overflow
        if result < -0x80000000i64 {
            new_flag |= 1 << negative_bit;
        } else if result >= 0x80000000i64 {
            new_flag |= 1 << positive_bit;
        }
    }

    cpu.cop2.set_flag(new_flag);
}

// Limit A: MAC1-3 to IR1-3 with saturation
fn lim_a(cpu: &mut Cpu, result: i64, saturation_bit: u32, lm: bool) -> i16 {
    let flag = cpu.cop2.flag();
    let limit = if lm { 0 } else { 1 };

    let mut new_flag = flag;
    let mut value = result;

    if value < -0x8000 * limit {
        value = -0x8000 * limit;
        new_flag |= 1 << saturation_bit;
    } else if value > 0x7FFF {
        value = 0x7FFF;
        new_flag |= 1 << saturation_bit;
    }

    cpu.cop2.set_flag(new_flag);
    value as i16
}

// Limit A with shift fraction checking
fn lim_a_sf(cpu: &mut Cpu, result: i64, saturation_bit: u32, lm: bool, sf: u32) -> i16 {
    let flag = cpu.cop2.flag();
    let limit = if lm { 0 } else { 1 };
    let mut new_flag = flag;

    // Check overflow before shift
    if (result >> 12) < -0x8000 {
        new_flag |= 1 << saturation_bit;
    } else if (result >> 12) > 0x7FFF {
        new_flag |= 1 << saturation_bit;
    }

    let mut value = result;

    if value < -0x8000 * limit {
        value = -0x8000 * limit;
        if sf != 0 {
            new_flag |= 1 << saturation_bit;
        }
    } else if value > 0x7FFF {
        value = 0x7FFF;
        new_flag |= 1 << saturation_bit;
    }

    cpu.cop2.set_flag(new_flag);
    value as i16
}

// Limit B: MAC0 to IR0 (0-255)
fn lim_b(cpu: &mut Cpu, result: i32, saturation_bit: u32) -> u8 {
    let flag = cpu.cop2.flag();
    let mut new_flag = flag;
    let mut value = result;

    if value < 0 {
        value = 0;
        new_flag |= 1 << saturation_bit;
    } else if value > 0xFF {
        value = 0xFF;
        new_flag |= 1 << saturation_bit;
    }

    cpu.cop2.set_flag(new_flag);
    value as u8
}

// Limit C: MAC1-3 to color FIFO (0-65535)
fn lim_c(cpu: &mut Cpu, result: i32) -> u16 {
    let flag = cpu.cop2.flag();
    let mut value = result;

    if value < 0 {
        value = 0;
        cpu.cop2.set_flag(flag | (1 << 18));
    } else if value > 0xFFFF {
        value = 0xFFFF;
        cpu.cop2.set_flag(flag | (1 << 18));
    }

    value as u16
}

// Limit E: Z-value limiting (0-4096)
fn lim_e(cpu: &mut Cpu, result: i64) -> u16 {
    let flag = cpu.cop2.flag();
    let mut value = result;

    if value < 0 {
        value = 0;
        cpu.cop2.set_flag(flag | (1 << 12));
    } else if value > 0x1000 {
        value = 0x1000;
        cpu.cop2.set_flag(flag | (1 << 12));
    }

    value as u16
}

// Set SXY flag for screen coordinate saturation
fn set_sxy_flag(cpu: &mut Cpu, result: i32, is_y: bool) {
    let flag = cpu.cop2.flag();

    if result < -0x400 || result > 0x3FF {
        let bit = if is_y { 13 } else { 14 };
        cpu.cop2.set_flag(flag | (1 << bit));
    }
}

// Push color to FIFO
fn push_color_fifo(cpu: &mut Cpu, r: i32, g: i32, b: i32, code: u8) {
    let r = lim_b(cpu, r, 21);
    let g = lim_b(cpu, g, 20);
    let b = lim_b(cpu, b, 19);

    let rgb1 = cpu.cop2.rgb1();
    let rgb2 = cpu.cop2.rgb2();

    cpu.cop2.set_rgb0(rgb1.0, rgb1.1, rgb1.2, rgb1.3);
    cpu.cop2.set_rgb1(rgb2.0, rgb2.1, rgb2.2, rgb2.3);
    cpu.cop2.set_rgb2(r, g, b, code);
}

// Clamp i64 value to range
fn clamp_i64(value: i64, low: i64, high: i64) -> i64 {
    if value < low {
        low
    } else if value > high {
        high
    } else {
        value
    }
}

// Vector type for GTE operations
#[derive(Clone)]
struct Vec3 {
    x: i64,
    y: i64,
    z: i64,
}

impl Vec3 {
    fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    fn from_v0(cpu: &Cpu) -> Self {
        let (x, y) = cpu.cop2.v0_xy();
        let z = cpu.cop2.v0_z();
        Self::new(x as i64, y as i64, z as i64)
    }

    fn from_v1(cpu: &Cpu) -> Self {
        let (x, y) = cpu.cop2.v1_xy();
        let z = cpu.cop2.v1_z();
        Self::new(x as i64, y as i64, z as i64)
    }

    fn from_v2(cpu: &Cpu) -> Self {
        let (x, y) = cpu.cop2.v2_xy();
        let z = cpu.cop2.v2_z();
        Self::new(x as i64, y as i64, z as i64)
    }

    fn from_ir(cpu: &Cpu) -> Self {
        Self::new(cpu.cop2.ir1() as i64, cpu.cop2.ir2() as i64, cpu.cop2.ir3() as i64)
    }

    fn from_translation(cpu: &Cpu) -> Self {
        Self::new(
            (cpu.cop2.tr_x() as i64) << 12,
            (cpu.cop2.tr_y() as i64) << 12,
            (cpu.cop2.tr_z() as i64) << 12,
        )
    }

    fn from_background(cpu: &Cpu) -> Self {
        Self::new(
            (cpu.cop2.rbk() as i64) << 12,
            (cpu.cop2.gbk() as i64) << 12,
            (cpu.cop2.bbk() as i64) << 12,
        )
    }

    fn from_far_color(cpu: &Cpu) -> Self {
        Self::new(
            (cpu.cop2.rfc() as i64) << 12,
            (cpu.cop2.gfc() as i64) << 12,
            (cpu.cop2.bfc() as i64) << 12,
        )
    }

    fn from_rgbc(cpu: &Cpu) -> Self {
        let (r, g, b, _) = cpu.cop2.rgbc();
        Self::new((r as i64) << 16, (g as i64) << 16, (b as i64) << 16)
    }

    fn from_rgbc_scaled(cpu: &Cpu) -> Self {
        let (r, g, b, _) = cpu.cop2.rgbc();
        Self::new((r as i64) << 4, (g as i64) << 4, (b as i64) << 4)
    }

    fn from_rgb0(cpu: &Cpu) -> Self {
        let (r, g, b, _) = cpu.cop2.rgb0();
        Self::new((r as i64) << 16, (g as i64) << 16, (b as i64) << 16)
    }

    fn zero() -> Self {
        Self::new(0, 0, 0)
    }

    fn shift(&self, sf: u32) -> Self {
        if sf != 0 {
            Self::new(self.x >> sf, self.y >> sf, self.z >> sf)
        } else {
            self.clone()
        }
    }

    fn scale(&self, factor: i64) -> Self {
        Self::new(self.x * factor, self.y * factor, self.z * factor)
    }

    fn add(&self, other: &Vec3) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    fn sub(&self, other: &Vec3) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    fn multiply_components(&self, x: i64, y: i64, z: i64) -> Self {
        Self::new(self.x * x, self.y * y, self.z * z)
    }
}

// Matrix type for GTE operations
struct Matrix {
    m: [i64; 9],
}

impl Matrix {
    fn from_rotation(cpu: &Cpu) -> Self {
        let (r11, r12) = cpu.cop2.r11r12();
        let (r13, r21) = cpu.cop2.r13r21();
        let (r22, r23) = cpu.cop2.r22r23();
        let (r31, r32) = cpu.cop2.r31r32();
        let r33 = cpu.cop2.r33();

        Self {
            m: [
                r11 as i64, r12 as i64, r13 as i64, r21 as i64, r22 as i64, r23 as i64, r31 as i64, r32 as i64,
                r33 as i64,
            ],
        }
    }

    fn from_light(cpu: &Cpu) -> Self {
        let (l11, l12) = cpu.cop2.l11l12();
        let (l13, l21) = cpu.cop2.l13l21();
        let (l22, l23) = cpu.cop2.l22l23();
        let (l31, l32) = cpu.cop2.l31l32();
        let l33 = cpu.cop2.l33();

        Self {
            m: [
                l11 as i64, l12 as i64, l13 as i64, l21 as i64, l22 as i64, l23 as i64, l31 as i64, l32 as i64,
                l33 as i64,
            ],
        }
    }

    fn from_color(cpu: &Cpu) -> Self {
        let (lr1, lr2) = cpu.cop2.lr1lr2();
        let (lr3, lg1) = cpu.cop2.lr3lg1();
        let (lg2, lg3) = cpu.cop2.lg2lg3();
        let (lb1, lb2) = cpu.cop2.lb1lb2();
        let lb3 = cpu.cop2.lb3();

        Self {
            m: [
                lr1 as i64, lr2 as i64, lr3 as i64, lg1 as i64, lg2 as i64, lg3 as i64, lb1 as i64, lb2 as i64,
                lb3 as i64,
            ],
        }
    }

    fn from_reserved(cpu: &Cpu) -> Self {
        let (_, r13) = cpu.cop2.r13r21();
        let (r22, _) = cpu.cop2.r22r23();
        let (r, _, _, _) = cpu.cop2.rgbc();
        let ir0 = cpu.cop2.ir0();

        Self {
            m: [
                -((r as i64) << 4),
                (r as i64) << 4,
                ir0 as i64,
                r13 as i64,
                r13 as i64,
                r13 as i64,
                r22 as i64,
                r22 as i64,
                r22 as i64,
            ],
        }
    }

    fn multiply(&self, v: &Vec3, tr: &Vec3, cpu: &mut Cpu) -> Vec3 {
        let sum_x = tr.x + self.m[0] * v.x + self.m[1] * v.y + self.m[2] * v.z;
        let sum_y = tr.y + self.m[3] * v.x + self.m[4] * v.y + self.m[5] * v.z;
        let sum_z = tr.z + self.m[6] * v.x + self.m[7] * v.y + self.m[8] * v.z;

        // Set MAC flags for intermediate calculations
        let mac1_1 = tr.x + self.m[0] * v.x;
        let mac1_2 = mac1_1 + self.m[1] * v.y;
        let mac1_3 = mac1_2 + self.m[2] * v.z;
        set_mac_flag(cpu, 1, mac1_1);
        set_mac_flag(cpu, 1, mac1_2);
        set_mac_flag(cpu, 1, mac1_3);

        let mac2_1 = tr.y + self.m[3] * v.x;
        let mac2_2 = mac2_1 + self.m[4] * v.y;
        let mac2_3 = mac2_2 + self.m[5] * v.z;
        set_mac_flag(cpu, 2, mac2_1);
        set_mac_flag(cpu, 2, mac2_2);
        set_mac_flag(cpu, 2, mac2_3);

        let mac3_1 = tr.z + self.m[6] * v.x;
        let mac3_2 = mac3_1 + self.m[7] * v.y;
        let mac3_3 = mac3_2 + self.m[8] * v.z;
        set_mac_flag(cpu, 3, mac3_1);
        set_mac_flag(cpu, 3, mac3_2);
        set_mac_flag(cpu, 3, mac3_3);

        Vec3::new(sum_x, sum_y, sum_z)
    }
}

// GTE instruction handlers

pub fn gte_rtps(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    let tv = Vec3::from_translation(cpu);
    let mv = Vec3::from_v0(cpu);
    let mat = Matrix::from_rotation(cpu);

    let vector = mat.multiply(&mv, &tv, cpu);
    let result = vector.shift(sf);

    let mac1 = result.x as i32;
    let mac2 = result.y as i32;
    let mac3 = result.z as i32;

    let ir1 = lim_a(cpu, mac1 as i64, 24, lm);
    let ir2 = lim_a(cpu, mac2 as i64, 23, lm);
    let ir3 = lim_a_sf(cpu, mac3 as i64, 22, lm, sf);

    set_mac_flag(cpu, 1, vector.x);
    set_mac_flag(cpu, 2, vector.y);
    set_mac_flag(cpu, 3, vector.z);

    let sz = lim_c(cpu, (result.z >> (12 - sf)) as i32);
    let division = gte_divide(cpu, cpu.cop2.h() as u32, sz as u32);

    let sx = (division as i64) * (ir1 as i64) + (cpu.cop2.ofx() as i64);
    let sy = (division as i64) * (ir2 as i64) + (cpu.cop2.ofy() as i64);
    let p = (division as i64) * (cpu.cop2.dqa() as i64) + (cpu.cop2.dqb() as i64);

    set_mac_flag(cpu, 0, sx);
    set_mac_flag(cpu, 0, sy);
    set_mac_flag(cpu, 0, p);

    set_sxy_flag(cpu, (sx >> 16) as i32, false);
    set_sxy_flag(cpu, (sy >> 16) as i32, true);

    let sx2 = clamp_i64(sx >> 16, -0x400, 0x3FF) as i16;
    let sy2 = clamp_i64(sy >> 16, -0x400, 0x3FF) as i16;
    let ir0 = lim_e(cpu, p >> 12);

    cpu.cop2.set_ir0(ir0 as i16);
    cpu.cop2.set_ir1(ir1);
    cpu.cop2.set_ir2(ir2);
    cpu.cop2.set_ir3(ir3);

    cpu.cop2.set_mac0(sx as i32);
    cpu.cop2.set_mac1(mac1);
    cpu.cop2.set_mac2(mac2);
    cpu.cop2.set_mac3(mac3);

    // Push through SXY FIFO
    let (sx1, sy1) = cpu.cop2.sxy1();
    let (sx2_old, sy2_old) = cpu.cop2.sxy2();
    cpu.cop2.set_sxy0(sx1, sy1);
    cpu.cop2.set_sxy1(sx2_old, sy2_old);
    cpu.cop2.set_sxy2(sx2, sy2);

    // Push through SZ FIFO
    let sz1 = cpu.cop2.sz1();
    let sz2 = cpu.cop2.sz2();
    let sz3 = cpu.cop2.sz3();
    cpu.cop2.set_sz0(sz1);
    cpu.cop2.set_sz1(sz2);
    cpu.cop2.set_sz2(sz3);
    cpu.cop2.set_sz3(sz);
}

pub fn gte_nclip(_instr: &Instruction, cpu: &mut Cpu) {
    cpu.cop2.set_flag(0);

    let (sx0, sy0) = cpu.cop2.sxy0();
    let (sx1, sy1) = cpu.cop2.sxy1();
    let (sx2, sy2) = cpu.cop2.sxy2();

    let sx0 = sx0 as i64;
    let sy0 = sy0 as i64;
    let sx1 = sx1 as i64;
    let sy1 = sy1 as i64;
    let sx2 = sx2 as i64;
    let sy2 = sy2 as i64;

    let opz = sx0 * sy1 + sx1 * sy2 + sx2 * sy0 - sx0 * sy2 - sx1 * sy0 - sx2 * sy1;

    set_mac_flag(cpu, 0, opz);
    cpu.cop2.set_mac0(opz as i32);
}

pub fn gte_op(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    let ir1 = cpu.cop2.ir1() as i64;
    let ir2 = cpu.cop2.ir2() as i64;
    let ir3 = cpu.cop2.ir3() as i64;

    let (r11, _) = cpu.cop2.r11r12();
    let (_, r22) = cpu.cop2.r22r23();
    let r33 = cpu.cop2.r33();

    let d1 = r11 as i64;
    let d2 = r22 as i64;
    let d3 = r33 as i64;

    let mac1 = ir3 * d2 - ir2 * d3;
    let mac2 = ir1 * d3 - ir3 * d1;
    let mac3 = ir2 * d1 - ir1 * d2;

    set_mac_flag(cpu, 1, mac1);
    set_mac_flag(cpu, 2, mac2);
    set_mac_flag(cpu, 3, mac3);

    let mac1 = mac1 >> sf;
    let mac2 = mac2 >> sf;
    let mac3 = mac3 >> sf;

    let ir1_new = lim_a(cpu, mac1, 24, lm);
    let ir2_new = lim_a(cpu, mac2, 23, lm);
    let ir3_new = lim_a(cpu, mac3, 22, lm);

    cpu.cop2.set_ir1(ir1_new);
    cpu.cop2.set_ir2(ir2_new);
    cpu.cop2.set_ir3(ir3_new);
    cpu.cop2.set_mac1(mac1 as i32);
    cpu.cop2.set_mac2(mac2 as i32);
    cpu.cop2.set_mac3(mac3 as i32);
}

pub fn gte_dpcs(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    let far_color = Vec3::from_far_color(cpu);
    let mac_color = Vec3::from_rgbc(cpu);

    let subtracted = far_color.sub(&mac_color);

    set_mac_flag(cpu, 1, subtracted.x);
    set_mac_flag(cpu, 2, subtracted.y);
    set_mac_flag(cpu, 3, subtracted.z);

    let mut limit = subtracted.shift(sf);

    let ir1 = lim_a(cpu, limit.x, 24, false);
    let ir2 = lim_a(cpu, limit.y, 23, false);
    let ir3 = lim_a(cpu, limit.z, 22, false);

    limit.x = ir1 as i64;
    limit.y = ir2 as i64;
    limit.z = ir3 as i64;

    limit = limit.scale(cpu.cop2.ir0() as i64).add(&mac_color);

    set_mac_flag(cpu, 1, limit.x);
    set_mac_flag(cpu, 2, limit.y);
    set_mac_flag(cpu, 3, limit.z);

    limit = limit.shift(sf);

    let ir1_final = lim_a(cpu, limit.x, 24, lm);
    let ir2_final = lim_a(cpu, limit.y, 23, lm);
    let ir3_final = lim_a(cpu, limit.z, 22, lm);

    let (_, _, _, code) = cpu.cop2.rgbc();
    push_color_fifo(
        cpu,
        (limit.x >> 4) as i32,
        (limit.y >> 4) as i32,
        (limit.z >> 4) as i32,
        code,
    );

    cpu.cop2.set_ir1(ir1_final);
    cpu.cop2.set_ir2(ir2_final);
    cpu.cop2.set_ir3(ir3_final);
    cpu.cop2.set_mac1(limit.x as i32);
    cpu.cop2.set_mac2(limit.y as i32);
    cpu.cop2.set_mac3(limit.z as i32);
}

pub fn gte_intpl(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    let far_color = Vec3::from_far_color(cpu);
    let mac_color = Vec3::new(
        (cpu.cop2.ir1() as i64) << 12,
        (cpu.cop2.ir2() as i64) << 12,
        (cpu.cop2.ir3() as i64) << 12,
    );

    let subtracted = far_color.sub(&mac_color);

    set_mac_flag(cpu, 1, subtracted.x);
    set_mac_flag(cpu, 2, subtracted.y);
    set_mac_flag(cpu, 3, subtracted.z);

    let mut limit = subtracted.shift(sf);

    let ir1 = lim_a(cpu, limit.x, 24, false);
    let ir2 = lim_a(cpu, limit.y, 23, false);
    let ir3 = lim_a(cpu, limit.z, 22, false);

    limit.x = ir1 as i64;
    limit.y = ir2 as i64;
    limit.z = ir3 as i64;

    limit = limit.scale(cpu.cop2.ir0() as i64).add(&mac_color);

    set_mac_flag(cpu, 1, limit.x);
    set_mac_flag(cpu, 2, limit.y);
    set_mac_flag(cpu, 3, limit.z);

    limit = limit.shift(sf);

    let ir1_final = lim_a(cpu, limit.x, 24, lm);
    let ir2_final = lim_a(cpu, limit.y, 23, lm);
    let ir3_final = lim_a(cpu, limit.z, 22, lm);

    let (_, _, _, code) = cpu.cop2.rgbc();
    push_color_fifo(
        cpu,
        (limit.x >> 4) as i32,
        (limit.y >> 4) as i32,
        (limit.z >> 4) as i32,
        code,
    );

    cpu.cop2.set_ir1(ir1_final);
    cpu.cop2.set_ir2(ir2_final);
    cpu.cop2.set_ir3(ir3_final);
    cpu.cop2.set_mac1(limit.x as i32);
    cpu.cop2.set_mac2(limit.y as i32);
    cpu.cop2.set_mac3(limit.z as i32);
}

pub fn gte_mvmva(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();
    let mx = instr.gte_mx();
    let vx = instr.gte_v();
    let tx = instr.gte_cv();

    cpu.cop2.set_flag(0);

    let v = match vx {
        0 => Vec3::from_v0(cpu),
        1 => Vec3::from_v1(cpu),
        2 => Vec3::from_v2(cpu),
        3 => Vec3::from_ir(cpu),
        _ => Vec3::zero(),
    };

    let t = match tx {
        0 => Vec3::from_translation(cpu),
        1 => Vec3::from_background(cpu),
        2 => Vec3::from_far_color(cpu),
        _ => Vec3::zero(),
    };

    let m = match mx {
        0 => Matrix::from_rotation(cpu),
        1 => Matrix::from_light(cpu),
        2 => Matrix::from_color(cpu),
        3 => Matrix::from_reserved(cpu),
        _ => Matrix::from_rotation(cpu),
    };

    // Special buggy behavior for tx=2
    if tx == 2 {
        let result_x = t.x + m.m[0] * v.x;
        let result_y = t.y + m.m[3] * v.x;
        let result_z = t.z + m.m[6] * v.x;

        set_mac_flag(cpu, 1, result_x);
        set_mac_flag(cpu, 2, result_y);
        set_mac_flag(cpu, 3, result_z);

        lim_a(cpu, result_x >> sf, 24, false);
        lim_a(cpu, result_y >> sf, 23, false);
        lim_a(cpu, result_z >> sf, 22, false);

        let mut result_x = m.m[1] * v.y;
        result_x = result_x + m.m[2] * v.z;
        set_mac_flag(cpu, 1, result_x);

        let mut result_y = m.m[4] * v.y;
        result_y = result_y + m.m[5] * v.z;
        set_mac_flag(cpu, 2, result_y);

        let mut result_z = m.m[7] * v.y;
        result_z = result_z + m.m[8] * v.z;
        set_mac_flag(cpu, 3, result_z);

        let result_x = result_x >> sf;
        let result_y = result_y >> sf;
        let result_z = result_z >> sf;

        let ir1 = lim_a(cpu, result_x, 24, lm);
        let ir2 = lim_a(cpu, result_y, 23, lm);
        let ir3 = lim_a(cpu, result_z, 22, lm);

        cpu.cop2.set_ir1(ir1);
        cpu.cop2.set_ir2(ir2);
        cpu.cop2.set_ir3(ir3);
        cpu.cop2.set_mac1(result_x as i32);
        cpu.cop2.set_mac2(result_y as i32);
        cpu.cop2.set_mac3(result_z as i32);
    } else {
        let result = m.multiply(&v, &t, cpu);
        let result = result.shift(sf);

        let ir1 = lim_a(cpu, result.x, 24, lm);
        let ir2 = lim_a(cpu, result.y, 23, lm);
        let ir3 = lim_a(cpu, result.z, 22, lm);

        cpu.cop2.set_ir1(ir1);
        cpu.cop2.set_ir2(ir2);
        cpu.cop2.set_ir3(ir3);
        cpu.cop2.set_mac1(result.x as i32);
        cpu.cop2.set_mac2(result.y as i32);
        cpu.cop2.set_mac3(result.z as i32);
    }
}

pub fn gte_ncds(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    let v = Vec3::from_v0(cpu);
    let bk = Vec3::from_background(cpu);
    let llm = Matrix::from_light(cpu);
    let lcm = Matrix::from_color(cpu);
    let rgbc = Vec3::from_rgbc_scaled(cpu);
    let far_color = Vec3::from_far_color(cpu);

    let light_result = llm.multiply(&v, &Vec3::zero(), cpu);
    let light_result = light_result.shift(sf);

    let ir1 = lim_a(cpu, light_result.x, 24, lm);
    let ir2 = lim_a(cpu, light_result.y, 23, lm);
    let ir3 = lim_a(cpu, light_result.z, 22, lm);

    let ir = Vec3::new(ir1 as i64, ir2 as i64, ir3 as i64);
    let bk_light_result = lcm.multiply(&ir, &bk, cpu);
    let bk_light_result = bk_light_result.shift(sf);

    let ir1 = lim_a(cpu, bk_light_result.x, 24, lm);
    let ir2 = lim_a(cpu, bk_light_result.y, 23, lm);
    let ir3 = lim_a(cpu, bk_light_result.z, 22, lm);

    let color_vector = Vec3::new((ir1 as i64) * rgbc.x, (ir2 as i64) * rgbc.y, (ir3 as i64) * rgbc.z);

    set_mac_flag(cpu, 1, color_vector.x);
    set_mac_flag(cpu, 2, color_vector.y);
    set_mac_flag(cpu, 3, color_vector.z);

    let subtracted = far_color.sub(&color_vector);

    set_mac_flag(cpu, 1, subtracted.x);
    set_mac_flag(cpu, 2, subtracted.y);
    set_mac_flag(cpu, 3, subtracted.z);

    let mut limit = subtracted.shift(sf);

    let ir1 = lim_a(cpu, limit.x, 24, false);
    let ir2 = lim_a(cpu, limit.y, 23, false);
    let ir3 = lim_a(cpu, limit.z, 22, false);

    limit.x = ir1 as i64;
    limit.y = ir2 as i64;
    limit.z = ir3 as i64;

    limit = limit.scale(cpu.cop2.ir0() as i64);

    set_mac_flag(cpu, 1, limit.x);
    set_mac_flag(cpu, 2, limit.y);
    set_mac_flag(cpu, 3, limit.z);

    limit = limit.add(&color_vector);

    set_mac_flag(cpu, 1, limit.x);
    set_mac_flag(cpu, 2, limit.y);
    set_mac_flag(cpu, 3, limit.z);

    limit = limit.shift(sf);

    let ir1_final = lim_a(cpu, limit.x, 24, lm);
    let ir2_final = lim_a(cpu, limit.y, 23, lm);
    let ir3_final = lim_a(cpu, limit.z, 22, lm);

    cpu.cop2.set_mac1(limit.x as i32);
    cpu.cop2.set_mac2(limit.y as i32);
    cpu.cop2.set_mac3(limit.z as i32);
    cpu.cop2.set_ir1(ir1_final);
    cpu.cop2.set_ir2(ir2_final);
    cpu.cop2.set_ir3(ir3_final);

    let (_, _, _, code) = cpu.cop2.rgbc();
    push_color_fifo(
        cpu,
        (limit.x >> 4) as i32,
        (limit.y >> 4) as i32,
        (limit.z >> 4) as i32,
        code,
    );
}

pub fn gte_cdp(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    let bk = Vec3::from_background(cpu);
    let lcm = Matrix::from_color(cpu);
    let ir = Vec3::from_ir(cpu);
    let rgbc = Vec3::from_rgbc_scaled(cpu);
    let far_color = Vec3::from_far_color(cpu);

    let bk_light_result = lcm.multiply(&ir, &bk, cpu);
    let bk_light_result = bk_light_result.shift(sf);

    let ir1 = lim_a(cpu, bk_light_result.x, 24, lm);
    let ir2 = lim_a(cpu, bk_light_result.y, 23, lm);
    let ir3 = lim_a(cpu, bk_light_result.z, 22, lm);

    let color_vector = Vec3::new((ir1 as i64) * rgbc.x, (ir2 as i64) * rgbc.y, (ir3 as i64) * rgbc.z);

    set_mac_flag(cpu, 1, color_vector.x);
    set_mac_flag(cpu, 2, color_vector.y);
    set_mac_flag(cpu, 3, color_vector.z);

    let subtracted = far_color.sub(&color_vector);

    set_mac_flag(cpu, 1, subtracted.x);
    set_mac_flag(cpu, 2, subtracted.y);
    set_mac_flag(cpu, 3, subtracted.z);

    let mut limit = subtracted.shift(sf);

    let ir1 = lim_a(cpu, limit.x, 24, false);
    let ir2 = lim_a(cpu, limit.y, 23, false);
    let ir3 = lim_a(cpu, limit.z, 22, false);

    limit.x = ir1 as i64;
    limit.y = ir2 as i64;
    limit.z = ir3 as i64;

    limit = limit.scale(cpu.cop2.ir0() as i64);

    set_mac_flag(cpu, 1, limit.x);
    set_mac_flag(cpu, 2, limit.y);
    set_mac_flag(cpu, 3, limit.z);

    limit = limit.add(&color_vector);

    set_mac_flag(cpu, 1, limit.x);
    set_mac_flag(cpu, 2, limit.y);
    set_mac_flag(cpu, 3, limit.z);

    limit = limit.shift(sf);

    let ir1_final = lim_a(cpu, limit.x, 24, lm);
    let ir2_final = lim_a(cpu, limit.y, 23, lm);
    let ir3_final = lim_a(cpu, limit.z, 22, lm);

    cpu.cop2.set_mac1(limit.x as i32);
    cpu.cop2.set_mac2(limit.y as i32);
    cpu.cop2.set_mac3(limit.z as i32);
    cpu.cop2.set_ir1(ir1_final);
    cpu.cop2.set_ir2(ir2_final);
    cpu.cop2.set_ir3(ir3_final);

    let (_, _, _, code) = cpu.cop2.rgbc();
    push_color_fifo(
        cpu,
        (limit.x >> 4) as i32,
        (limit.y >> 4) as i32,
        (limit.z >> 4) as i32,
        code,
    );
}

pub fn gte_ncdt(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    for i in 0..3 {
        let v = match i {
            0 => Vec3::from_v0(cpu),
            1 => Vec3::from_v1(cpu),
            2 => Vec3::from_v2(cpu),
            _ => Vec3::zero(),
        };

        let bk = Vec3::from_background(cpu);
        let llm = Matrix::from_light(cpu);
        let lcm = Matrix::from_color(cpu);
        let rgbc = Vec3::from_rgbc_scaled(cpu);
        let far_color = Vec3::from_far_color(cpu);

        let light_result = llm.multiply(&v, &Vec3::zero(), cpu);
        let light_result = light_result.shift(sf);

        let ir1 = lim_a(cpu, light_result.x, 24, lm);
        let ir2 = lim_a(cpu, light_result.y, 23, lm);
        let ir3 = lim_a(cpu, light_result.z, 22, lm);

        let ir = Vec3::new(ir1 as i64, ir2 as i64, ir3 as i64);
        let bk_light_result = lcm.multiply(&ir, &bk, cpu);

        set_mac_flag(cpu, 1, bk_light_result.x);
        set_mac_flag(cpu, 2, bk_light_result.y);
        set_mac_flag(cpu, 3, bk_light_result.z);

        let bk_light_result = bk_light_result.shift(sf);

        let ir1 = lim_a(cpu, bk_light_result.x, 24, lm);
        let ir2 = lim_a(cpu, bk_light_result.y, 23, lm);
        let ir3 = lim_a(cpu, bk_light_result.z, 22, lm);

        let color_vector = Vec3::new((ir1 as i64) * rgbc.x, (ir2 as i64) * rgbc.y, (ir3 as i64) * rgbc.z);

        set_mac_flag(cpu, 1, color_vector.x);
        set_mac_flag(cpu, 2, color_vector.y);
        set_mac_flag(cpu, 3, color_vector.z);

        let subtracted = far_color.sub(&color_vector);

        set_mac_flag(cpu, 1, subtracted.x);
        set_mac_flag(cpu, 2, subtracted.y);
        set_mac_flag(cpu, 3, subtracted.z);

        let mut limit = subtracted.shift(sf);

        let ir1 = lim_a(cpu, limit.x, 24, false);
        let ir2 = lim_a(cpu, limit.y, 23, false);
        let ir3 = lim_a(cpu, limit.z, 22, false);

        limit.x = ir1 as i64;
        limit.y = ir2 as i64;
        limit.z = ir3 as i64;

        limit = limit.scale(cpu.cop2.ir0() as i64);

        set_mac_flag(cpu, 1, limit.x);
        set_mac_flag(cpu, 2, limit.y);
        set_mac_flag(cpu, 3, limit.z);

        limit = limit.add(&color_vector);

        set_mac_flag(cpu, 1, limit.x);
        set_mac_flag(cpu, 2, limit.y);
        set_mac_flag(cpu, 3, limit.z);

        limit = limit.shift(sf);

        let ir1_final = lim_a(cpu, limit.x, 24, lm);
        let ir2_final = lim_a(cpu, limit.y, 23, lm);
        let ir3_final = lim_a(cpu, limit.z, 22, lm);

        cpu.cop2.set_mac1(limit.x as i32);
        cpu.cop2.set_mac2(limit.y as i32);
        cpu.cop2.set_mac3(limit.z as i32);
        cpu.cop2.set_ir1(ir1_final);
        cpu.cop2.set_ir2(ir2_final);
        cpu.cop2.set_ir3(ir3_final);

        let (_, _, _, code) = cpu.cop2.rgbc();
        push_color_fifo(
            cpu,
            (limit.x >> 4) as i32,
            (limit.y >> 4) as i32,
            (limit.z >> 4) as i32,
            code,
        );
    }
}

pub fn gte_nccs(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    let v = Vec3::from_v0(cpu);
    let bk = Vec3::from_background(cpu);
    let llm = Matrix::from_light(cpu);
    let lcm = Matrix::from_color(cpu);
    let rgbc = Vec3::from_rgbc_scaled(cpu);

    let light_result = llm.multiply(&v, &Vec3::zero(), cpu);
    let light_result = light_result.shift(sf);

    let ir1 = lim_a(cpu, light_result.x, 24, lm);
    let ir2 = lim_a(cpu, light_result.y, 23, lm);
    let ir3 = lim_a(cpu, light_result.z, 22, lm);

    let ir = Vec3::new(ir1 as i64, ir2 as i64, ir3 as i64);
    let bk_light_result = lcm.multiply(&ir, &bk, cpu);
    let bk_light_result = bk_light_result.shift(sf);

    let ir1 = lim_a(cpu, bk_light_result.x, 24, lm);
    let ir2 = lim_a(cpu, bk_light_result.y, 23, lm);
    let ir3 = lim_a(cpu, bk_light_result.z, 22, lm);

    let color_vector = Vec3::new((ir1 as i64) * rgbc.x, (ir2 as i64) * rgbc.y, (ir3 as i64) * rgbc.z);

    set_mac_flag(cpu, 1, color_vector.x);
    set_mac_flag(cpu, 2, color_vector.y);
    set_mac_flag(cpu, 3, color_vector.z);

    let limit = color_vector.shift(sf);

    let ir1_final = lim_a(cpu, limit.x, 24, lm);
    let ir2_final = lim_a(cpu, limit.y, 23, lm);
    let ir3_final = lim_a(cpu, limit.z, 22, lm);

    cpu.cop2.set_mac1(limit.x as i32);
    cpu.cop2.set_mac2(limit.y as i32);
    cpu.cop2.set_mac3(limit.z as i32);
    cpu.cop2.set_ir1(ir1_final);
    cpu.cop2.set_ir2(ir2_final);
    cpu.cop2.set_ir3(ir3_final);

    let (_, _, _, code) = cpu.cop2.rgbc();
    push_color_fifo(
        cpu,
        (limit.x >> 4) as i32,
        (limit.y >> 4) as i32,
        (limit.z >> 4) as i32,
        code,
    );
}

pub fn gte_cc(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    let bk = Vec3::from_background(cpu);
    let lcm = Matrix::from_color(cpu);
    let ir = Vec3::from_ir(cpu);
    let rgbc = Vec3::from_rgbc_scaled(cpu);

    let bk_light_result = lcm.multiply(&ir, &bk, cpu);
    let bk_light_result = bk_light_result.shift(sf);

    let ir1 = lim_a(cpu, bk_light_result.x, 24, lm);
    let ir2 = lim_a(cpu, bk_light_result.y, 23, lm);
    let ir3 = lim_a(cpu, bk_light_result.z, 22, lm);

    let color_vector = Vec3::new((ir1 as i64) * rgbc.x, (ir2 as i64) * rgbc.y, (ir3 as i64) * rgbc.z);

    set_mac_flag(cpu, 1, color_vector.x);
    set_mac_flag(cpu, 2, color_vector.y);
    set_mac_flag(cpu, 3, color_vector.z);

    let limit = color_vector.shift(sf);

    let ir1_final = lim_a(cpu, limit.x, 24, lm);
    let ir2_final = lim_a(cpu, limit.y, 23, lm);
    let ir3_final = lim_a(cpu, limit.z, 22, lm);

    cpu.cop2.set_mac1(limit.x as i32);
    cpu.cop2.set_mac2(limit.y as i32);
    cpu.cop2.set_mac3(limit.z as i32);
    cpu.cop2.set_ir1(ir1_final);
    cpu.cop2.set_ir2(ir2_final);
    cpu.cop2.set_ir3(ir3_final);

    let (_, _, _, code) = cpu.cop2.rgbc();
    push_color_fifo(
        cpu,
        (limit.x >> 4) as i32,
        (limit.y >> 4) as i32,
        (limit.z >> 4) as i32,
        code,
    );
}

pub fn gte_ncs(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    let v = Vec3::from_v0(cpu);
    let bk = Vec3::from_background(cpu);
    let llm = Matrix::from_light(cpu);
    let lcm = Matrix::from_color(cpu);

    let light_result = llm.multiply(&v, &Vec3::zero(), cpu);
    let light_result = light_result.shift(sf);

    let ir1 = lim_a(cpu, light_result.x, 24, lm);
    let ir2 = lim_a(cpu, light_result.y, 23, lm);
    let ir3 = lim_a(cpu, light_result.z, 22, lm);

    let ir = Vec3::new(ir1 as i64, ir2 as i64, ir3 as i64);
    let bk_light_result = lcm.multiply(&ir, &bk, cpu);
    let bk_light_result = bk_light_result.shift(sf);

    let ir1_final = lim_a(cpu, bk_light_result.x, 24, lm);
    let ir2_final = lim_a(cpu, bk_light_result.y, 23, lm);
    let ir3_final = lim_a(cpu, bk_light_result.z, 22, lm);

    cpu.cop2.set_mac1(bk_light_result.x as i32);
    cpu.cop2.set_mac2(bk_light_result.y as i32);
    cpu.cop2.set_mac3(bk_light_result.z as i32);
    cpu.cop2.set_ir1(ir1_final);
    cpu.cop2.set_ir2(ir2_final);
    cpu.cop2.set_ir3(ir3_final);

    let (_, _, _, code) = cpu.cop2.rgbc();
    push_color_fifo(
        cpu,
        (bk_light_result.x >> 4) as i32,
        (bk_light_result.y >> 4) as i32,
        (bk_light_result.z >> 4) as i32,
        code,
    );
}

pub fn gte_nct(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    for i in 0..3 {
        let v = match i {
            0 => Vec3::from_v0(cpu),
            1 => Vec3::from_v1(cpu),
            2 => Vec3::from_v2(cpu),
            _ => Vec3::zero(),
        };

        let bk = Vec3::from_background(cpu);
        let llm = Matrix::from_light(cpu);
        let lcm = Matrix::from_color(cpu);

        let light_result = llm.multiply(&v, &Vec3::zero(), cpu);
        let light_result = light_result.shift(sf);

        let ir1 = lim_a(cpu, light_result.x, 24, lm);
        let ir2 = lim_a(cpu, light_result.y, 23, lm);
        let ir3 = lim_a(cpu, light_result.z, 22, lm);

        let ir = Vec3::new(ir1 as i64, ir2 as i64, ir3 as i64);
        let bk_light_result = lcm.multiply(&ir, &bk, cpu);
        let bk_light_result = bk_light_result.shift(sf);

        let ir1_final = lim_a(cpu, bk_light_result.x, 24, lm);
        let ir2_final = lim_a(cpu, bk_light_result.y, 23, lm);
        let ir3_final = lim_a(cpu, bk_light_result.z, 22, lm);

        cpu.cop2.set_mac1(bk_light_result.x as i32);
        cpu.cop2.set_mac2(bk_light_result.y as i32);
        cpu.cop2.set_mac3(bk_light_result.z as i32);
        cpu.cop2.set_ir1(ir1_final);
        cpu.cop2.set_ir2(ir2_final);
        cpu.cop2.set_ir3(ir3_final);

        let (_, _, _, code) = cpu.cop2.rgbc();
        push_color_fifo(
            cpu,
            (bk_light_result.x >> 4) as i32,
            (bk_light_result.y >> 4) as i32,
            (bk_light_result.z >> 4) as i32,
            code,
        );
    }
}

pub fn gte_sqr(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };

    cpu.cop2.set_flag(0);

    let ir = Vec3::from_ir(cpu);
    let result = ir.multiply_components(ir.x, ir.y, ir.z);

    let mac1 = (result.x >> sf) as i32;
    let mac2 = (result.y >> sf) as i32;
    let mac3 = (result.z >> sf) as i32;

    let ir1 = lim_a(cpu, mac1 as i64, 24, false);
    let ir2 = lim_a(cpu, mac2 as i64, 23, false);
    let ir3 = lim_a(cpu, mac3 as i64, 22, false);

    cpu.cop2.set_mac1(mac1);
    cpu.cop2.set_mac2(mac2);
    cpu.cop2.set_mac3(mac3);
    cpu.cop2.set_ir1(ir1);
    cpu.cop2.set_ir2(ir2);
    cpu.cop2.set_ir3(ir3);
}

pub fn gte_dcpl(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    let far_color = Vec3::from_far_color(cpu);
    let rgbc = Vec3::from_rgbc_scaled(cpu);
    let ir = Vec3::from_ir(cpu);

    let mac_color = Vec3::new(ir.x * rgbc.x, ir.y * rgbc.y, ir.z * rgbc.z);
    let subtracted = far_color.sub(&mac_color);

    set_mac_flag(cpu, 1, subtracted.x);
    set_mac_flag(cpu, 2, subtracted.y);
    set_mac_flag(cpu, 3, subtracted.z);

    let mut limit = subtracted.shift(sf);

    let ir1 = lim_a(cpu, limit.x, 24, false);
    let ir2 = lim_a(cpu, limit.y, 23, false);
    let ir3 = lim_a(cpu, limit.z, 22, false);

    limit.x = ir1 as i64;
    limit.y = ir2 as i64;
    limit.z = ir3 as i64;

    limit = limit.scale(cpu.cop2.ir0() as i64).add(&mac_color);

    set_mac_flag(cpu, 1, limit.x);
    set_mac_flag(cpu, 2, limit.y);
    set_mac_flag(cpu, 3, limit.z);

    limit = limit.shift(sf);

    let ir1_final = lim_a(cpu, limit.x, 24, lm);
    let ir2_final = lim_a(cpu, limit.y, 23, lm);
    let ir3_final = lim_a(cpu, limit.z, 22, lm);

    let (_, _, _, code) = cpu.cop2.rgbc();
    push_color_fifo(
        cpu,
        (limit.x >> 4) as i32,
        (limit.y >> 4) as i32,
        (limit.z >> 4) as i32,
        code,
    );

    cpu.cop2.set_ir1(ir1_final);
    cpu.cop2.set_ir2(ir2_final);
    cpu.cop2.set_ir3(ir3_final);
    cpu.cop2.set_mac1(limit.x as i32);
    cpu.cop2.set_mac2(limit.y as i32);
    cpu.cop2.set_mac3(limit.z as i32);
}

pub fn gte_dpct(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    for _ in 0..3 {
        let far_color = Vec3::from_far_color(cpu);
        let mac_color = Vec3::from_rgb0(cpu);

        let subtracted = far_color.sub(&mac_color);

        set_mac_flag(cpu, 1, subtracted.x);
        set_mac_flag(cpu, 2, subtracted.y);
        set_mac_flag(cpu, 3, subtracted.z);

        let mut limit = subtracted.shift(sf);

        let ir1 = lim_a(cpu, limit.x, 24, false);
        let ir2 = lim_a(cpu, limit.y, 23, false);
        let ir3 = lim_a(cpu, limit.z, 22, false);

        limit.x = ir1 as i64;
        limit.y = ir2 as i64;
        limit.z = ir3 as i64;

        limit = limit.scale(cpu.cop2.ir0() as i64).add(&mac_color);

        set_mac_flag(cpu, 1, limit.x);
        set_mac_flag(cpu, 2, limit.y);
        set_mac_flag(cpu, 3, limit.z);

        limit = limit.shift(sf);

        let ir1_final = lim_a(cpu, limit.x, 24, lm);
        let ir2_final = lim_a(cpu, limit.y, 23, lm);
        let ir3_final = lim_a(cpu, limit.z, 22, lm);

        let (_, _, _, code) = cpu.cop2.rgbc();
        push_color_fifo(
            cpu,
            (limit.x >> 4) as i32,
            (limit.y >> 4) as i32,
            (limit.z >> 4) as i32,
            code,
        );

        cpu.cop2.set_ir1(ir1_final);
        cpu.cop2.set_ir2(ir2_final);
        cpu.cop2.set_ir3(ir3_final);
        cpu.cop2.set_mac1(limit.x as i32);
        cpu.cop2.set_mac2(limit.y as i32);
        cpu.cop2.set_mac3(limit.z as i32);
    }
}

pub fn gte_avsz3(_instr: &Instruction, cpu: &mut Cpu) {
    cpu.cop2.set_flag(0);

    let zsf3 = cpu.cop2.zsf3() as i64;
    let sz1 = cpu.cop2.sz1() as i64;
    let sz2 = cpu.cop2.sz2() as i64;
    let sz3 = cpu.cop2.sz3() as i64;

    let mac0 = zsf3 * (sz1 + sz2 + sz3);

    set_mac_flag(cpu, 0, mac0);

    let otz = lim_c(cpu, (mac0 >> 12) as i32);

    cpu.cop2.set_mac0(mac0 as i32);
    cpu.cop2.set_otz(otz);
}

pub fn gte_avsz4(_instr: &Instruction, cpu: &mut Cpu) {
    cpu.cop2.set_flag(0);

    let zsf4 = cpu.cop2.zsf4() as i64;
    let sz0 = cpu.cop2.sz0() as i64;
    let sz1 = cpu.cop2.sz1() as i64;
    let sz2 = cpu.cop2.sz2() as i64;
    let sz3 = cpu.cop2.sz3() as i64;

    let mac0 = zsf4 * (sz0 + sz1 + sz2 + sz3);

    set_mac_flag(cpu, 0, mac0);

    let otz = lim_c(cpu, (mac0 >> 12) as i32);

    cpu.cop2.set_mac0(mac0 as i32);
    cpu.cop2.set_otz(otz);
}

pub fn gte_rtpt(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    for i in 0..3 {
        let tv = Vec3::from_translation(cpu);
        let mv = match i {
            0 => Vec3::from_v0(cpu),
            1 => Vec3::from_v1(cpu),
            2 => Vec3::from_v2(cpu),
            _ => Vec3::zero(),
        };
        let mat = Matrix::from_rotation(cpu);

        let vector = mat.multiply(&mv, &tv, cpu);
        let result = vector.shift(sf);

        let mac1 = result.x as i32;
        let mac2 = result.y as i32;
        let mac3 = result.z as i32;

        let ir1 = lim_a(cpu, mac1 as i64, 24, lm);
        let ir2 = lim_a(cpu, mac2 as i64, 23, lm);
        let ir3 = lim_a_sf(cpu, mac3 as i64, 22, lm, sf);

        set_mac_flag(cpu, 1, vector.x);
        set_mac_flag(cpu, 2, vector.y);
        set_mac_flag(cpu, 3, vector.z);

        let sz = lim_c(cpu, (result.z >> (12 - sf)) as i32);
        let division = gte_divide(cpu, cpu.cop2.h() as u32, sz as u32);

        let sx = (division as i64) * (ir1 as i64) + (cpu.cop2.ofx() as i64);
        let sy = (division as i64) * (ir2 as i64) + (cpu.cop2.ofy() as i64);
        let p = (division as i64) * (cpu.cop2.dqa() as i64) + (cpu.cop2.dqb() as i64);

        set_mac_flag(cpu, 0, sx);
        set_mac_flag(cpu, 0, sy);
        if i == 2 {
            set_mac_flag(cpu, 0, p);
        }

        set_sxy_flag(cpu, (sx >> 16) as i32, false);
        set_sxy_flag(cpu, (sy >> 16) as i32, true);

        let sx2 = clamp_i64(sx >> 16, -0x400, 0x3FF) as i16;
        let sy2 = clamp_i64(sy >> 16, -0x400, 0x3FF) as i16;

        if i == 2 {
            let ir0 = lim_e(cpu, p >> 12);
            cpu.cop2.set_ir0(ir0 as i16);
        }

        cpu.cop2.set_ir1(ir1);
        cpu.cop2.set_ir2(ir2);
        cpu.cop2.set_ir3(ir3);

        cpu.cop2.set_mac0(sx as i32);
        cpu.cop2.set_mac1(mac1);
        cpu.cop2.set_mac2(mac2);
        cpu.cop2.set_mac3(mac3);

        // Push through SXY FIFO
        let (sx1, sy1) = cpu.cop2.sxy1();
        let (sx2_old, sy2_old) = cpu.cop2.sxy2();
        cpu.cop2.set_sxy0(sx1, sy1);
        cpu.cop2.set_sxy1(sx2_old, sy2_old);
        cpu.cop2.set_sxy2(sx2, sy2);

        // Push through SZ FIFO
        let sz1 = cpu.cop2.sz1();
        let sz2 = cpu.cop2.sz2();
        let sz3 = cpu.cop2.sz3();
        cpu.cop2.set_sz0(sz1);
        cpu.cop2.set_sz1(sz2);
        cpu.cop2.set_sz2(sz3);
        cpu.cop2.set_sz3(sz);
    }
}

pub fn gte_gpf(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    let ir = Vec3::from_ir(cpu);
    let limit = ir.scale(cpu.cop2.ir0() as i64);

    set_mac_flag(cpu, 1, limit.x);
    set_mac_flag(cpu, 2, limit.y);
    set_mac_flag(cpu, 3, limit.z);

    let limit = limit.shift(sf);

    let ir1 = lim_a(cpu, limit.x, 24, lm);
    let ir2 = lim_a(cpu, limit.y, 23, lm);
    let ir3 = lim_a(cpu, limit.z, 22, lm);

    let (_, _, _, code) = cpu.cop2.rgbc();
    push_color_fifo(
        cpu,
        (limit.x >> 4) as i32,
        (limit.y >> 4) as i32,
        (limit.z >> 4) as i32,
        code,
    );

    cpu.cop2.set_ir1(ir1);
    cpu.cop2.set_ir2(ir2);
    cpu.cop2.set_ir3(ir3);
    cpu.cop2.set_mac1(limit.x as i32);
    cpu.cop2.set_mac2(limit.y as i32);
    cpu.cop2.set_mac3(limit.z as i32);
}

pub fn gte_gpl(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    let mac = Vec3::new(
        (cpu.cop2.mac1() as i64) << sf,
        (cpu.cop2.mac2() as i64) << sf,
        (cpu.cop2.mac3() as i64) << sf,
    );

    let ir = Vec3::from_ir(cpu);
    let limit = ir.scale(cpu.cop2.ir0() as i64).add(&mac);

    set_mac_flag(cpu, 1, limit.x);
    set_mac_flag(cpu, 2, limit.y);
    set_mac_flag(cpu, 3, limit.z);

    let limit = limit.shift(sf);

    let ir1 = lim_a(cpu, limit.x, 24, lm);
    let ir2 = lim_a(cpu, limit.y, 23, lm);
    let ir3 = lim_a(cpu, limit.z, 22, lm);

    let (_, _, _, code) = cpu.cop2.rgbc();
    push_color_fifo(
        cpu,
        (limit.x >> 4) as i32,
        (limit.y >> 4) as i32,
        (limit.z >> 4) as i32,
        code,
    );

    cpu.cop2.set_ir1(ir1);
    cpu.cop2.set_ir2(ir2);
    cpu.cop2.set_ir3(ir3);
    cpu.cop2.set_mac1(limit.x as i32);
    cpu.cop2.set_mac2(limit.y as i32);
    cpu.cop2.set_mac3(limit.z as i32);
}

pub fn gte_ncct(instr: &Instruction, cpu: &mut Cpu) {
    let sf = if instr.gte_sf() { 12 } else { 0 };
    let lm = instr.gte_lm();

    cpu.cop2.set_flag(0);

    for i in 0..3 {
        let v = match i {
            0 => Vec3::from_v0(cpu),
            1 => Vec3::from_v1(cpu),
            2 => Vec3::from_v2(cpu),
            _ => Vec3::zero(),
        };

        let bk = Vec3::from_background(cpu);
        let llm = Matrix::from_light(cpu);
        let lcm = Matrix::from_color(cpu);
        let rgbc = Vec3::from_rgbc_scaled(cpu);

        let light_result = llm.multiply(&v, &Vec3::zero(), cpu);
        let light_result = light_result.shift(sf);

        let ir1 = lim_a(cpu, light_result.x, 24, lm);
        let ir2 = lim_a(cpu, light_result.y, 23, lm);
        let ir3 = lim_a(cpu, light_result.z, 22, lm);

        let ir = Vec3::new(ir1 as i64, ir2 as i64, ir3 as i64);
        let bk_light_result = lcm.multiply(&ir, &bk, cpu);
        let bk_light_result = bk_light_result.shift(sf);

        let ir1 = lim_a(cpu, bk_light_result.x, 24, lm);
        let ir2 = lim_a(cpu, bk_light_result.y, 23, lm);
        let ir3 = lim_a(cpu, bk_light_result.z, 22, lm);

        let color_vector = Vec3::new((ir1 as i64) * rgbc.x, (ir2 as i64) * rgbc.y, (ir3 as i64) * rgbc.z);

        set_mac_flag(cpu, 1, color_vector.x);
        set_mac_flag(cpu, 2, color_vector.y);
        set_mac_flag(cpu, 3, color_vector.z);

        let limit = color_vector.shift(sf);

        let ir1_final = lim_a(cpu, limit.x, 24, lm);
        let ir2_final = lim_a(cpu, limit.y, 23, lm);
        let ir3_final = lim_a(cpu, limit.z, 22, lm);

        cpu.cop2.set_mac1(limit.x as i32);
        cpu.cop2.set_mac2(limit.y as i32);
        cpu.cop2.set_mac3(limit.z as i32);
        cpu.cop2.set_ir1(ir1_final);
        cpu.cop2.set_ir2(ir2_final);
        cpu.cop2.set_ir3(ir3_final);

        let (_, _, _, code) = cpu.cop2.rgbc();
        push_color_fifo(
            cpu,
            (limit.x >> 4) as i32,
            (limit.y >> 4) as i32,
            (limit.z >> 4) as i32,
            code,
        );
    }
}
