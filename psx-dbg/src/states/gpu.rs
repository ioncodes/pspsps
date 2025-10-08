#[derive(Clone, Default)]
pub struct GpuState {
    pub frame: Vec<(u8, u8, u8)>,
    pub width: usize,
    pub height: usize,
}
