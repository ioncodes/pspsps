#[derive(Clone)]
pub struct CpuState {
    pub pc: u32,
    pub registers: [u32; 32],
    pub hi: u32,
    pub lo: u32,
    pub cop0: psx_core::cpu::cop::cop0::Cop0,
    pub cop2: psx_core::cpu::cop::cop2::Cop2,
}

impl Default for CpuState {
    fn default() -> Self {
        Self {
            pc: 0,
            registers: [0; 32],
            hi: 0,
            lo: 0,
            cop0: psx_core::cpu::cop::cop0::Cop0::new(),
            cop2: psx_core::cpu::cop::cop2::Cop2::new(),
        }
    }
}
