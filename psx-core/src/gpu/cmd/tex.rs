use proc_bitfield::bitfield;

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct DrawModeSettingCommand(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub texture_page_x_base: u32 @ 0..=3,
        pub texture_page_y_base_1: bool @ 4,
        pub semi_transparency: u32 @ 5..=6,
        pub texture_page_colors: u32 @ 7..=8,
        pub dither: bool @ 9,
        pub drawing_to_display_area: bool @ 10,
        pub texture_page_y_base_2: bool @ 11,
        pub textured_rectangle_x_flip: bool @ 12,
        pub textured_rectangle_y_flip: bool @ 13,
        pub command: u32 @ 24..=31,
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct TextureWindowSettingCommand(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub texture_window_x_mask: u32 @ 0..=4,
        pub texture_window_y_mask: u32 @ 5..=9,
        pub texture_window_x_offset: u32 @ 10..=14,
        pub texture_window_y_offset: u32 @ 15..=19,
        pub command: u32 @ 24..=31,
    }
}

