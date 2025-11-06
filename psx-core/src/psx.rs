use crate::cpu::Cpu;
use crate::cpu::decoder::Instruction;
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
        let instr_raw = self.mmu.read_u32(self.cpu.pc);
        let instr = Instruction::decode(instr_raw);
        tracing::debug!(target: "psx_core::cpu", "{:08X}: [{:08X}] {: <30} {{ {:?} }}", self.cpu.pc, instr_raw, format!("{}", instr), instr);
        self.cpu.pc += 4;
        (instr.handler)(&instr, &mut self.cpu, &mut self.mmu);
    }
}
