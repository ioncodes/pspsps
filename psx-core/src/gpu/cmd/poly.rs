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
    pub fn vertex_count(&self) -> usize {
        if self.vertices_count() { 4 } else { 3 }
    }

    // with these helpers we could technically cast the bool into a number and add it directly
    // for a bit of optimization, but the compiler should be able to take care of that
    // TODO: also, i think we can do this completely branchless. figure that out eventually

    #[inline(always)]
    pub fn vertex_idx(&self, vertex_num: usize) -> usize {
        if vertex_num == 0 {
            return 0;
        }

        // we need to account for interleaved words for each vertex
        let mut idx = 0;
        for _ in 0..vertex_num {
            if self.textured() {
                idx += 1; // skip UV
            }
            if self.gouraud() {
                idx += 1; // skip color
            }
            idx += 1; // vertex itself
        }

        idx
    }

    #[inline(always)]
    pub fn uv_idx(&self, vertex_num: usize) -> usize {
        if vertex_num == 0 {
            return 1; // uv always after vertex 0
        }

        let mut idx = 3;
        for _ in 1..vertex_num {
            if self.gouraud() {
                idx += 1; // skip color
            }
            idx += 2; // skip vertex and uv
        }

        if self.gouraud() {
            idx += 1;
        }

        idx
    }

    #[inline(always)]
    pub fn color_idx(&self, vertex_num: usize) -> usize {
        debug_assert!(vertex_num > 0, "Vertex 0 color is in command word");

        let mut idx = 1;
        if self.textured() {
            idx += 1; // skip UV0 if textured
        }

        for _ in 1..vertex_num {
            idx += 2; // skip color and vertex

            if self.textured() {
                idx += 1; // skip UV
            }
        }

        idx
    }
}
