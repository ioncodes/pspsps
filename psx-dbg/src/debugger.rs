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
    trace: VecDeque<(u32, Instruction)>,
    breakpoints: HashSet<u32>,
    sideload_exe: Option<Vec<u8>>,
    cycle_counter: u32,
}

static BIOS: &[u8] = include_bytes!("../../bios/SCPH1000.BIN");

impl Debugger {
    pub fn new(channel_send: Sender<DebuggerEvent>, channel_recv: Receiver<DebuggerEvent>) -> Self {
        Self {
            psx: Psx::new(BIOS),
            channel_send,
            channel_recv,
            is_running: false,
            trace: VecDeque::with_capacity(1000),
            breakpoints: HashSet::new(),
            sideload_exe: None,
            cycle_counter: 0,
        }
    }

    pub fn sideload_exe(&mut self, exe_buffer: Vec<u8>) {
        self.sideload_exe = Some(exe_buffer); // Store the sideloaded executable, used for reset
        self.psx.sideload_exe(self.sideload_exe.clone().unwrap());
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

                if let Ok(instr) = self.psx.step() {
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
                        let fifo_len = self.psx.cpu.mmu.gpu.gp.fifo_len();

                        self.channel_send
                            .send(DebuggerEvent::GpuUpdated(GpuState {
                                vram_frame: self.psx.cpu.mmu.gpu.internal_frame(),
                                vram_width: VRAM_WIDTH,
                                vram_height: VRAM_HEIGHT,
                                display_frame: self.psx.cpu.mmu.gpu.display_frame(),
                                display_width,
                                display_height,
                                horizontal_resolution: display_width,
                                vertical_resolution: display_height,
                                gp1_status,
                                fifo_len,
                            }))
                            .expect("Failed to send GPU update event");
                    }
                } else {
                    // If step fails, we stop running
                    self.is_running = false;
                    self.channel_send
                        .send(DebuggerEvent::Pause)
                        .expect("Failed to send pause event");
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
                    self.psx = Psx::new(BIOS);
                    self.psx.sideload_exe(
                        self.sideload_exe
                            .take()
                            .unwrap_or_else(|| panic!("No sideloaded executable found")),
                    );

                    self.is_running = false;
                    self.trace.clear();

                    psx_core::cpu::internal::tty_buffer()
                        .lock()
                        .unwrap()
                        .clear();
                    psx_core::cpu::internal::tty_buffer()
                        .lock()
                        .unwrap()
                        .clear();

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
                _ => {}
            }
        }
    }
}
