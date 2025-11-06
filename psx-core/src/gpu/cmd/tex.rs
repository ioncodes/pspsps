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

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct DrawingAreaTopLeftCommand(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub x1: u32 @ 0..=9,
        pub y1: u32 @ 10..=18,
        pub command: u32 @ 24..=31,
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct DrawingAreaBottomRightCommand(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub x2: u32 @ 0..=9,
        pub y2: u32 @ 10..=18,
        pub command: u32 @ 24..=31,
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct DrawingOffsetCommand(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub x_offset: u32 @ 0..=10,
        pub y_offset: u32 @ 11..=21,
        pub command: u32 @ 24..=31,
    }
}

impl DrawingOffsetCommand {
    pub fn x_offset_signed(&self) -> i32 {
        let val = self.x_offset();
        if val & 0x400 != 0 {
            (val | 0xFFFFF800) as i32
        } else {
            val as i32
        }
    }

    pub fn y_offset_signed(&self) -> i32 {
        let val = self.y_offset();
        if val & 0x400 != 0 {
            (val | 0xFFFFF800) as i32
        } else {
            val as i32
        }
    }
}

