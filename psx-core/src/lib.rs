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