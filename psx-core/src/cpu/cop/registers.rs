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
        pub boot_exception_vector_location: bool @ 22, // 0 = RAM, 1 = ROM
        pub cop0_enable: bool @ 28,
        pub cop1_enable: bool @ 29,
        pub cop2_enable: bool @ 30,
        pub cop3_enable: bool @ 31,
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct CauseRegister(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub exception_code: u32 @ 2..=6,
        pub software_interrupts: u32 @ 8..=9,
        pub interrupt_pending: u32 @ 10..=15,
        pub coprocessor_exception: u32 @ 28..=29,
        pub branch_taken: bool @ 30,
        pub branch_delay: bool @ 31,
    }
}
