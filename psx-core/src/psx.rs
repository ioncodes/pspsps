use crate::cpu::Cpu;
use crate::cpu::decoder::Instruction;
use crate::exe::Exe;
use crate::sio::joy::ControllerState;

pub const PSX_RESET_ADDRESS: u32 = 0xBFC0_0000;
pub const PSX_SIDELOAD_EXE_ADDRESS: u32 = 0x8003_0000;

pub const CPU_CLOCK: usize = 33_868_800;
pub const NTSC_VBLANK_CYCLES: usize = 33_868_800 / 60;
pub const PAL_VBLANK_CYCLES: usize = 33_868_800 / 50;
pub const NTSC_VBLANK_DURATION: usize = NTSC_VBLANK_CYCLES / 2; // ~282,240 cycles
pub const PAL_VBLANK_DURATION: usize = PAL_VBLANK_CYCLES / 2; // ~338,688 cycles

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

    pub fn load_cdrom(&mut self, cue_buffer: Vec<u8>, bin_buffer: Vec<u8>) {
        self.cpu.mmu.cdrom.insert_disk(cue_buffer, bin_buffer);
    }

    pub fn set_controller_state(&mut self, state: ControllerState) {
        self.cpu.mmu.sio.set_controller_state(state);
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
                address = format!("{:08X}", PSX_SIDELOAD_EXE_ADDRESS), 
                entrypoint = format!("{:08X}", exe.entry_point),
                "Sideloaded EXE"
            );
        }

        let instr = self.cpu.tick();

        let cycles = self.cpu.drain_cycles();
        self.cycles += cycles;

        for _ in 0..cycles {
            self.cpu.mmu.gpu.tick();
        }

        self.cpu.mmu.cdrom.tick(cycles);
        self.cpu.mmu.irq.status.set_cdrom(self.cpu.mmu.cdrom.check_and_clear_irq());

        self.cpu.mmu.sio.tick(cycles);
        if self.cpu.mmu.sio.should_trigger_irq() {
            self.cpu.mmu.irq.status.set_controller_and_memory_card(true);
        }

        self.cpu.mmu.perform_dma_transfers();

        if self.cycles >= NTSC_VBLANK_DURATION {
            self.cpu.mmu.irq.status.set_vblank(false);
            self.cpu
                .mmu
                .gpu
                .gp
                .gp1_status
                .set_drawing_even_odd_lines_in_interlace_mode(false);
        }

        if self.cycles >= NTSC_VBLANK_CYCLES {
            self.cycles = 0;

            self.cpu.mmu.irq.status.set_vblank(true);
            self.cpu
                .mmu
                .gpu
                .gp
                .gp1_status
                .set_drawing_even_odd_lines_in_interlace_mode(true);

            tracing::trace!(target: "psx_core::psx", "VBLANK period reached, setting I_STAT bit");
        }

        instr
    }
}
