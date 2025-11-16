#[derive(Clone, Default)]
pub struct IrqState {
    pub i_stat: u32,
    pub i_mask: u32,
}
