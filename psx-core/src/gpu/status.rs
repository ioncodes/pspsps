use proc_bitfield::bitfield;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoMode {
    Ntsc,
    Pal,
}

impl From<bool> for VideoMode {
    fn from(value: bool) -> Self {
        match value {
            false => VideoMode::Ntsc,
            true => VideoMode::Pal,
        }
    }
}

impl From<VideoMode> for bool {
    fn from(mode: VideoMode) -> bool {
        match mode {
            VideoMode::Ntsc => false,
            VideoMode::Pal => true,
        }
    }
}

impl std::fmt::Display for VideoMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoMode::Ntsc => write!(f, "NTSC"),
            VideoMode::Pal => write!(f, "PAL"),
        }
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Default)]
    pub struct StatusRegister(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub drawing_to_display_area: bool @ 10,
        pub horizontal_resolution2: bool @ 16,
        pub horizontal_resolution1: u8 @ 17..=18,
        pub vertical_resolution: bool @ 19,
        pub video_mode: bool [get VideoMode, set VideoMode] @ 20,
        pub vertical_interlace: bool @ 22,
        pub display_enable: bool @ 23,
        pub data_request: bool @ 25,
        pub ready_to_receive_cmd_word: bool @ 26,
        pub ready_to_send_vram_to_cpu: bool @ 27,
        pub ready_to_receive_dma_block: bool @ 28,
        pub dma_direction: u8 @ 29..=30,
        pub drawing_even_odd_lines_in_interlace_mode: bool @ 31,
    }
}

impl StatusRegister {
    pub fn hres(&self) -> u32 {
        match (self.horizontal_resolution1(), self.horizontal_resolution2()) {
            (0, false) => 256,
            (1, false) => 320,
            (_, true) => 368,
            (2, false) => 512,
            (3, false) => 640,
            _ => unreachable!(),
        }
    }

    pub fn vres(&self) -> u32 {
        match (self.vertical_resolution(), self.vertical_interlace()) {
            (false, false) => 240,
            (_, true) => 480,
            _ => unreachable!(),
        }
    }
}
