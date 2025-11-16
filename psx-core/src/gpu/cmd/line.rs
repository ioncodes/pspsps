use proc_bitfield::bitfield;

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct DrawLineCommand(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub color: u32 @ 0..=23,
        pub semi_transparent: bool @ 25,
        pub polyline: bool @ 27,
        pub gouraud: bool @ 28,
        pub command: u32 @ 29..=31,
    }
}

impl DrawLineCommand {
    /// For a flat-shaded single line: 2 vertices
    /// For a flat-shaded polyline: N vertices (terminated by 0x55555555 or 0x50005000)
    /// For a Gouraud-shaded single line: 2 vertices with colors
    /// For a Gouraud-shaded polyline: N vertices with colors

    /// Get the index of the next vertex in the command buffer
    /// vertex_num: 0 for first vertex (in command word), 1+ for subsequent vertices
    #[inline(always)]
    pub fn vertex_idx(&self, vertex_num: usize) -> usize {
        if vertex_num == 0 {
            return 0; // First vertex position is in command word bits 0-23 (misusing color field)
        }

        // Each subsequent vertex is 1 or 2 words depending on Gouraud shading
        let mut idx = 0;
        for _ in 0..vertex_num {
            if self.gouraud() {
                idx += 1; // color word
            }
            idx += 1; // vertex word
        }

        idx
    }

    /// Get the index of the color word for a vertex (for Gouraud shading)
    #[inline(always)]
    pub fn color_idx(&self, vertex_num: usize) -> usize {
        debug_assert!(vertex_num > 0, "Vertex 0 color is in command word");
        debug_assert!(self.gouraud(), "Color index only valid for Gouraud shading");

        // Each vertex has color then position
        let mut idx = 0;
        for _ in 0..vertex_num {
            idx += 2; // skip previous color and vertex
        }

        idx
    }
}
