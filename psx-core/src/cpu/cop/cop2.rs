use crate::cpu::cop::Cop;

#[derive(Clone, Copy)]
pub struct Cop2 {
    registers: [u32; 32],
}

impl Cop2 {
    pub fn new() -> Self {
        Self {
            registers: [0xFF; 32],
        }
    }
}

impl Cop for Cop2 {
    #[inline(always)]
    fn read_register(&self, register: u32) -> u32 {
        tracing::error!(target: "psx_core::cpu", register, "Attempted to read unimplemented COP2 register");
        self.registers[register as usize]
    }

    #[inline(always)]
    fn write_register(&mut self, register: u32, value: u32) {
        tracing::error!(target: "psx_core::cpu", register, value = format!("{:08X}", value), "Attempted to write to unimplemented COP2 register");
        self.registers[register as usize] = value;
    }
}
