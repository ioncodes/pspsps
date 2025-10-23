use proc_bitfield::bitfield;

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct DrawPolygonCommand(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub color: u32 @ 0..=23,
        pub raw_texture: bool @ 24,
        pub semi_transparent: bool @ 25,
        pub textured: bool @ 26,
        pub vertices_count: bool @ 27,
        pub gouraud: bool @ 28,
        pub command: u32 @ 29..=31,
    }
}

impl DrawPolygonCommand {
    #[inline(always)]
    pub fn gouraud_color_idx(&self) -> usize {
        0
    }

    #[inline(always)]
    pub fn vertex_idx(&self) -> usize {
        // TODO: fix, needs to be based on vertex count aka vertices_count
        if self.gouraud() { 1 } else { 0 }
    }

    #[inline(always)]
    pub fn uv_idx(&self) -> usize {
        // TODO: fix, needs to be based on vertex count aka vertices_count
        if self.gouraud() { 2 } else { 1 }
    }
}
