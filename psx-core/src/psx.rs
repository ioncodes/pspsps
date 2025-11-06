use crate::cpu::Cpu;
use crate::cpu::decoder::Instruction;

const PSX_RESET_ADDRESS: u32 = 0xBFC0_0000;

pub struct Psx {
    pub cpu: Cpu,
}

impl Psx {
    pub fn new(bios: &[u8]) -> Self {
        let mut cpu = Cpu::new();
        cpu.mmu.load(PSX_RESET_ADDRESS, &bios);
        cpu.pc = PSX_RESET_ADDRESS;

        Self { cpu }
    }

    pub fn step(&mut self) -> Instruction {
        self.cpu.tick()
    }
}
