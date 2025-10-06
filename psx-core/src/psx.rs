use crate::cpu::Cpu;
use crate::cpu::decoder::Instruction;
use crate::exe::Exe;

const PSX_RESET_ADDRESS: u32 = 0xBFC0_0000;
const PSX_SIDELOAD_EXE_ADDRESS: u32 = 0x8003_0000;

pub struct Psx {
    pub cpu: Cpu,
    pub cycles: usize,
    sideload_exe: Option<Exe>,
}

impl Psx {
    pub fn new(bios: &[u8]) -> Self {
        let mut cpu = Cpu::new();
        cpu.mmu.load(PSX_RESET_ADDRESS, &bios);
        cpu.pc = PSX_RESET_ADDRESS;

        Self {
            cpu,
            cycles: 0,
            sideload_exe: None,
        }
    }

    pub fn sideload_exe(&mut self, exe_buffer: Vec<u8>) {
        self.sideload_exe = Some(Exe::parse(exe_buffer));
    }

    pub fn step(&mut self) -> Result<Instruction, ()> {
        if let Some(exe) = &self.sideload_exe
            && self.cpu.pc == PSX_SIDELOAD_EXE_ADDRESS
        {
            self.cpu.mmu.load(exe.entry_point, &exe.data);
            self.cpu.write_register(28, exe.initial_gp);
            self.cpu.write_register(29, exe.sp());
            self.cpu.write_register(30, exe.fp());
            self.cpu.pc = exe.entry_point;

            tracing::debug!(
                target: "psx_core::psx",
                "EXE license string: {}", exe.license
            );

            tracing::info!(
                target: "psx_core::psx",
                "Sideloaded EXE at {:08X} with entry point {:08X}",
                PSX_SIDELOAD_EXE_ADDRESS, exe.entry_point
            );
        }

        let instr = self.cpu.tick();

        for _ in 0..self.cpu.drain_cycles() {
            self.cpu.mmu.gpu.tick();
        }

        instr
    }
}
