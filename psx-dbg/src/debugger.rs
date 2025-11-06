use crate::io::DebuggerEvent;
use crate::states::breakpoints::BreakpointsState;
use crate::states::cpu::CpuState;
use crate::states::gpu::GpuState;
use crate::states::mmu::MmuState;
use crate::states::trace::TraceState;
use crate::states::tty::TtyState;
use crossbeam_channel::{Receiver, Sender};
use psx_core::cpu::decoder::Instruction;
use psx_core::cpu::internal;
use psx_core::gpu::{VRAM_HEIGHT, VRAM_WIDTH};
use psx_core::psx::Psx;
use std::collections::{HashSet, VecDeque};

const GPU_UPDATE_INTERVAL: u32 = 100_000;

pub struct Debugger {
    pub psx: Psx,
    channel_send: Sender<DebuggerEvent>,
    channel_recv: Receiver<DebuggerEvent>,
    is_running: bool,
    ignore_errors: bool,
    trace: VecDeque<(u32, Instruction)>,
    breakpoints: HashSet<u32>,
    sideload_exe: Option<Vec<u8>>,
    cue_file: Option<Vec<u8>>,
    bin_file: Option<Vec<u8>>,
    bios: Vec<u8>,
    cycle_counter: u32,
}

impl Debugger {
    pub fn new(bios_path: String, channel_send: Sender<DebuggerEvent>, channel_recv: Receiver<DebuggerEvent>) -> Self {
        let bios =
            std::fs::read(&bios_path).unwrap_or_else(|e| panic!("Failed to read BIOS file '{}': {}", bios_path, e));

        Self {
            psx: Psx::new(&bios),
            channel_send,
            channel_recv,
            is_running: false,
            ignore_errors: true,
            trace: VecDeque::with_capacity(1000),
            breakpoints: HashSet::new(),
            sideload_exe: None,
            cue_file: None,
            bin_file: None,
            bios,
            cycle_counter: 0,
        }
    }

    pub fn with_sideloaded_exe(mut self, path: Option<String>) -> Self {
        if let Some(exe_path) = path {
            let exe_buffer = std::fs::read(&exe_path)
                .unwrap_or_else(|e| panic!("Failed to read sideloaded executable file '{}': {}", exe_path, e));
            self.sideload_exe = Some(exe_buffer.clone());
            self.psx.sideload_exe(exe_buffer);
        }

        self
    }

    pub fn with_cdrom_image(mut self, path: Option<String>) -> Self {
        if let Some(cdrom_path) = path {
            let cue_path = cdrom_path.clone();
            let bin_path = cue_path.replace(".cue", ".bin");

            let cue = std::fs::read(&cdrom_path)
                .unwrap_or_else(|e| panic!("Failed to read CD-ROM image file '{}': {}", cdrom_path, e));
            let bin = std::fs::read(&bin_path)
                .unwrap_or_else(|e| panic!("Failed to read CD-ROM image file '{}': {}", bin_path, e));

            self.cue_file = Some(cue.clone());
            self.bin_file = Some(bin.clone());

            self.psx.load_cdrom(cue, bin);
        }

        self
    }

    pub fn run(&mut self) {
        loop {
            self.process_events();

            if self.is_running {
                // Check for breakpoints
                if self.breakpoints.contains(&self.psx.cpu.pc) {
                    self.is_running = false;
                    self.channel_send
                        .send(DebuggerEvent::BreakpointHit(self.psx.cpu.pc))
                        .expect("Failed to send breakpoint hit event");
                    continue;
                }

                let old_pc = self.psx.cpu.pc;

                match self.psx.step() {
                    Ok(instr) => {
                        // Push to trace
                        self.trace.push_back((old_pc, instr));
                        if self.trace.len() > 1000 {
                            self.trace.pop_front();
                        }

                        self.cycle_counter += 1;
                        if self.cycle_counter >= GPU_UPDATE_INTERVAL {
                            self.cycle_counter = 0;

                            let (display_width, display_height) = self.psx.cpu.mmu.gpu.gp.resolution();
                            let gp1_status = self.psx.cpu.mmu.gpu.gp.status();

                            self.channel_send
                                .send(DebuggerEvent::GpuUpdated(GpuState {
                                    vram_frame: self.psx.cpu.mmu.gpu.internal_frame(),
                                    vram_width: VRAM_WIDTH,
                                    vram_height: VRAM_HEIGHT,
                                    display_frame: self.psx.cpu.mmu.gpu.display_frame(),
                                    display_width,
                                    display_height,
                                    gp1_status,
                                }))
                                .expect("Failed to send GPU update event");
                        }
                    }
                    Err(_) => {
                        // If step fails and we're not ignoring errors, stop running
                        if !self.ignore_errors {
                            self.is_running = false;
                            self.channel_send
                                .send(DebuggerEvent::Pause)
                                .expect("Failed to send pause event");
                        }
                        // Otherwise, just continue to the next iteration
                    }
                }
            }
        }
    }

    fn process_events(&mut self) {
        while let Ok(event) = self.channel_recv.try_recv() {
            match event {
                DebuggerEvent::Step => {
                    if self.is_running {
                        self.is_running = false;
                        self.channel_send
                            .send(DebuggerEvent::Paused)
                            .expect("Failed to send paused event");
                    } else {
                        let _ = self.psx.step();
                    }
                }
                DebuggerEvent::Run => {
                    self.is_running = true;
                    self.channel_send
                        .send(DebuggerEvent::Unpaused)
                        .expect("Failed to send update CPU event");
                }
                DebuggerEvent::Pause => {
                    self.is_running = false;
                    self.channel_send
                        .send(DebuggerEvent::Paused)
                        .expect("Failed to send paused event");
                }
                DebuggerEvent::UpdateCpu => {
                    self.channel_send
                        .send(DebuggerEvent::CpuUpdated(CpuState {
                            pc: self.psx.cpu.pc,
                            registers: self.psx.cpu.registers.clone(),
                            cop0: self.psx.cpu.cop0,
                            hi: self.psx.cpu.hi,
                            lo: self.psx.cpu.lo,
                        }))
                        .unwrap();
                }
                DebuggerEvent::UpdateMmu => {
                    self.channel_send
                        .send(DebuggerEvent::MmuUpdated(MmuState {
                            data: self.psx.cpu.mmu.memory.clone(),
                        }))
                        .unwrap();
                }
                DebuggerEvent::UpdateTrace => {
                    self.channel_send
                        .send(DebuggerEvent::TraceUpdated(TraceState {
                            instructions: self.trace.clone(),
                        }))
                        .unwrap();
                }
                DebuggerEvent::UpdateTty => {
                    self.channel_send
                        .send(DebuggerEvent::TtyUpdated(TtyState {
                            buffer: internal::tty_buffer().lock().unwrap().clone(),
                        }))
                        .unwrap();
                }
                DebuggerEvent::AddBreakpoint(addr) => {
                    self.breakpoints.insert(addr);
                    self.channel_send
                        .send(DebuggerEvent::BreakpointsUpdated(BreakpointsState {
                            breakpoints: self.breakpoints.clone(),
                        }))
                        .unwrap();
                }
                DebuggerEvent::RemoveBreakpoint(addr) => {
                    self.breakpoints.remove(&addr);
                    self.channel_send
                        .send(DebuggerEvent::BreakpointsUpdated(BreakpointsState {
                            breakpoints: self.breakpoints.clone(),
                        }))
                        .unwrap();
                }
                DebuggerEvent::ClearBreakpoints => {
                    self.breakpoints.clear();
                    self.channel_send
                        .send(DebuggerEvent::BreakpointsUpdated(BreakpointsState {
                            breakpoints: self.breakpoints.clone(),
                        }))
                        .unwrap();
                }
                DebuggerEvent::Reset => {
                    self.psx = Psx::new(&self.bios);

                    if let Some(exe_buffer) = &self.sideload_exe {
                        self.psx.sideload_exe(exe_buffer.clone());
                    }

                    if let (Some(cue), Some(bin)) = (&self.cue_file, &self.bin_file) {
                        self.psx.load_cdrom(cue.clone(), bin.clone());
                    }

                    self.is_running = false;
                    self.trace.clear();

                    psx_core::cpu::internal::tty_buffer().lock().unwrap().clear();
                    psx_core::cpu::internal::tty_buffer().lock().unwrap().clear();

                    self.channel_send
                        .send(DebuggerEvent::CpuUpdated(CpuState {
                            pc: self.psx.cpu.pc,
                            registers: self.psx.cpu.registers.clone(),
                            cop0: self.psx.cpu.cop0,
                            hi: self.psx.cpu.hi,
                            lo: self.psx.cpu.lo,
                        }))
                        .unwrap();
                    self.channel_send
                        .send(DebuggerEvent::MmuUpdated(MmuState {
                            data: self.psx.cpu.mmu.memory.clone(),
                        }))
                        .unwrap();
                    self.channel_send
                        .send(DebuggerEvent::TraceUpdated(TraceState {
                            instructions: self.trace.clone(),
                        }))
                        .unwrap();
                    self.channel_send
                        .send(DebuggerEvent::TtyUpdated(TtyState {
                            buffer: internal::tty_buffer().lock().unwrap().clone(),
                        }))
                        .unwrap();
                    self.channel_send
                        .send(DebuggerEvent::Paused)
                        .expect("Failed to send paused event");
                }
                DebuggerEvent::UpdateController(controller_state) => {
                    self.psx.set_controller_state(controller_state);
                }
                DebuggerEvent::SetIgnoreErrors(ignore) => {
                    self.ignore_errors = ignore;
                }
                _ => {}
            }
        }
    }
}
