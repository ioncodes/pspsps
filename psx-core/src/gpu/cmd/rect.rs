use proc_bitfield::bitfield;

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct DrawRectangleCommand(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub color: u32 @ 0..=23,
        pub raw_texture: bool @ 24,
        pub semi_transparent: bool @ 25,
        pub textured: bool @ 26,
        pub size: u32 @ 27..=28,
        pub command: u32 @ 29..=31,
    }
}

impl DrawRectangleCommand {
    #[inline(always)]
    pub fn vertex_idx(&self) -> usize {
        0
    }

    #[inline(always)]
    pub fn uv_idx(&self) -> usize {
        1
    }

    #[inline(always)]
    pub fn size_idx(&self) -> usize {
        if self.textured() { 2 } else { 1 }
    }
}
