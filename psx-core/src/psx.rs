use crate::cpu::Cpu;
use crate::mmu::Mmu;

const PSX_RESET_ADDRESS: u32 = 0xBFC0_0000;

pub struct Psx {
    pub cpu: Cpu,
    pub mmu: Mmu,
}

impl Psx {
    pub fn new(bios: &[u8]) -> Self {
        let mut mmu = Mmu::new();
        mmu.load(&bios, PSX_RESET_ADDRESS);

        let mut cpu = Cpu::new();
        cpu.pc = PSX_RESET_ADDRESS;

        Self { cpu, mmu }
    }

    pub fn step(&mut self) {
        self.cpu.tick(&mut self.mmu);
    }
}
