#[derive(Clone, Default)]
pub struct GpuState {
    pub vram_frame: Vec<(u8, u8, u8)>,
    pub vram_width: usize,
    pub vram_height: usize,
    pub display_frame: Vec<(u8, u8, u8)>,
    pub display_width: usize,
    pub display_height: usize,
}
