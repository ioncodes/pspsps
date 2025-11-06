#![feature(adt_const_params)]
#![feature(const_trait_impl)]
#![feature(const_cmp)]

pub mod cdrom;
pub mod cpu;
pub mod exe;
pub mod gpu;
pub mod irq;
pub mod mmu;
pub mod psx;
pub mod spu;
pub mod sio;
pub mod timer;

pub const fn regidx(register: &str) -> u8 {
    match register {
        "$zero" => 0,
        "$at" => 1,
        "$v0" => 2,
        "$v1" => 3,
        "$a0" => 4,
        "$a1" => 5,
        "$a2" => 6,
        "$a3" => 7,
        "$t0" => 8,
        "$t1" => 9,
        "$t2" => 10,
        "$t3" => 11,
        "$t4" => 12,
        "$t5" => 13,
        "$t6" => 14,
        "$t7" => 15,
        "$s0" => 16,
        "$s1" => 17,
        "$s2" => 18,
        "$s3" => 19,
        "$s4" => 20,
        "$s5" => 21,
        "$s6" => 22,
        "$s7" => 23,
        "$t8" => 24,
        "$t9" => 25,
        "$k0" => 26,
        "$k1" => 27,
        "$gp" => 28,
        "$sp" => 29,
        "$fp" => 30,
        "$ra" => 31,
        _ => unreachable!(),
    }
}

pub const fn gteidx(register: &str) -> u8 {
    match register {
        // Data registers (r0-r31)
        "v0_xy" => 0,
        "v0_z" => 1,
        "v1_xy" => 2,
        "v1_z" => 3,
        "v2_xy" => 4,
        "v2_z" => 5,
        "rgbc" => 6,
        "otz" => 7,
        "ir0" => 8,
        "ir1" => 9,
        "ir2" => 10,
        "ir3" => 11,
        "sxy0" => 12,
        "sxy1" => 13,
        "sxy2" => 14,
        "sxyp" => 15,
        "sz0" => 16,
        "sz1" => 17,
        "sz2" => 18,
        "sz3" => 19,
        "rgb0" => 20,
        "rgb1" => 21,
        "rgb2" => 22,
        "res1" => 23,
        "mac0" => 24,
        "mac1" => 25,
        "mac2" => 26,
        "mac3" => 27,
        "irgb" => 28,
        "orgb" => 29,
        "lzcs" => 30,
        "lzcr" => 31,
        // Control registers (r32-r63)
        "r11r12" => 32,
        "r13r21" => 33,
        "r22r23" => 34,
        "r31r32" => 35,
        "r33" => 36,
        "trx" => 37,
        "try" => 38,
        "trz" => 39,
        "l11l12" => 40,
        "l13l21" => 41,
        "l22l23" => 42,
        "l31l32" => 43,
        "l33" => 44,
        "rbk" => 45,
        "gbk" => 46,
        "bbk" => 47,
        "lr1lr2" => 48,
        "lr3lg1" => 49,
        "lg2lg3" => 50,
        "lb1lb2" => 51,
        "lb3" => 52,
        "rfc" => 53,
        "gfc" => 54,
        "bfc" => 55,
        "ofx" => 56,
        "ofy" => 57,
        "h" => 58,
        "dqa" => 59,
        "dqb" => 60,
        "zsf3" => 61,
        "zsf4" => 62,
        "flag" => 63,
        _ => unreachable!(),
    }
}

pub(crate) const fn calc_addr(base: u32, n: u32, size: u32, boundary: u32) -> (u32, u32) {
    let start = base + (n * boundary);
    let end = start + size - 1;
    (start, end)
}

#[macro_export]
macro_rules! define_addr {
    ($name:ident, $base:expr, $n:expr, $size:expr, $boundary:expr) => {
        pub const $name: (u32, u32) = crate::calc_addr($base, $n, $size, $boundary);
        paste::paste! {
            pub const [<$name _START>]: u32 = $name.0;
            pub const [<$name _END>]: u32 = $name.1;
        }
    };
}