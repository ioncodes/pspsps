use crate::io::DebuggerEvent;
use crate::states::breakpoints::BreakpointsState;
use crate::states::cpu::CpuState;
use crate::states::mmu::MmuState;
use crate::states::trace::TraceState;
use crate::states::tty::TtyState;
use crossbeam_channel::{Receiver, Sender};
use psx_core::cpu::decoder::Instruction;
use psx_core::cpu::internal;
use psx_core::psx::Psx;
use std::collections::{HashSet, VecDeque};

pub struct Debugger {
    pub psx: Psx,
    channel_send: Sender<DebuggerEvent>,
    channel_recv: Receiver<DebuggerEvent>,
    is_running: bool,
    trace: VecDeque<(u32, Instruction)>,
    breakpoints: HashSet<u32>,
}

static BIOS: &[u8] = include_bytes!("../../bios/SCPH1000.BIN");

impl Debugger {
    pub fn new(channel_send: Sender<DebuggerEvent>, channel_recv: Receiver<DebuggerEvent>) -> Self {
        let mut psx = Psx::new(BIOS);
        psx.sideload_exe(include_bytes!("../../external/amidog/psxtest_cpu.exe").to_vec());

        Self {
            psx,
            channel_send,
            channel_recv,
            is_running: false,
            trace: VecDeque::with_capacity(1000),
            breakpoints: HashSet::new(),
        }
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
                            buffer: internal::TTY_BUFFER.lock().unwrap().clone(),
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
                        include_bytes!("../../external/amidog/psxtest_cpu.exe").to_vec(),
                    );

                    self.is_running = false;
                    self.trace.clear();

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
                            buffer: internal::TTY_BUFFER.lock().unwrap().clone(),
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
