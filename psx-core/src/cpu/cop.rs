// pub trait CoProcessor {
//     fn read_register(&self, register: u32) -> u32;
//     fn write_register(&mut self, register: u32, value: u32);
// }

// pub struct Cop<Registers: CoProcessor> {
//     pub registers: Registers,
// }

// pub struct Cop0 {
//     pub bpc: u32,   // Breakpoint Control
//     pub cause: u32, // Cause register
// }

// impl CoProcessor for Cop0 {
//     #[inline(always)]
//     fn read_register(&self, register: u32) -> u32 {}

//     #[inline(always)]
//     fn write_register(&mut self, register: u32, value: u32) {
//         // Implement writing to COP0 registers
//     }
// }

use proc_bitfield::bitfield;

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct StatusRegister(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub current_interrupt_enable: bool @ 0,
        pub current_mode: bool @ 1,
        pub previous_interrupt_enable: bool @ 2,
        pub previous_mode: bool @ 3,
        pub old_interrupt_enable: bool @ 4,
        pub old_mode: bool @ 5,
        pub interrupt_mask: u32 @ 8..=15,
        pub isolate_cache: bool @ 16,
        pub swapped_cache: bool @ 17,
        pub cop0_enable: bool @ 28,
        pub cop1_enable: bool @ 29,
        pub cop2_enable: bool @ 30,
        pub cop3_enable: bool @ 31,
    }
}
