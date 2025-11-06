use crate::cpu::Cpu;
use crate::cpu::decoder::Instruction;
use crate::mmu::Mmu;

#[derive(Debug, Clone)]
pub struct Psx {
    pub cpu: Cpu,
    pub mmu: Mmu,
}

impl Psx {
    pub fn new(bios: &[u8]) -> Self {
        let mut mmu = Mmu::new();
        mmu.load(&bios, 0);

        Self {
            cpu: Cpu::new(),
            mmu,
        }
    }

    pub fn step(&mut self) {
        let instr_raw = self.mmu.read_u32(self.cpu.pc);
        let instr = Instruction::decode(instr_raw);
        tracing::debug!(target: "psx_core::cpu", "{:08X}: {}", self.cpu.pc, instr);
        self.cpu.pc += 4;
        (instr.handler)(&instr, &mut self.cpu, &mut self.mmu);
    }
}
