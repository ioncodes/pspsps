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
use std::io::Read;

const GPU_UPDATE_INTERVAL: u32 = 100_000;

fn load_rom(rom_path: &str) -> Result<Vec<u8>, String> {
    let path = std::path::Path::new(rom_path);

    // Check if it's a zip file
    if path.extension().and_then(|s| s.to_str()) == Some("zip") {
        tracing::info!("Detected ZIP file, extracting first .bin file...");

        let file = std::fs::File::open(rom_path).map_err(|e| format!("Failed to open ZIP file: {}", e))?;

        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

        // Get all .bin files and sort them
        let mut bin_files: Vec<String> = archive
            .file_names()
            .filter(|name| name.to_lowercase().ends_with(".bin"))
            .map(|s| s.to_string())
            .collect();

        bin_files.sort();

        if bin_files.is_empty() {
            return Err("No .bin files found in ZIP archive".to_string());
        }

        let first_bin = &bin_files[0];
        tracing::info!("Extracting: {}", first_bin);

        let mut bin_file = archive
            .by_name(first_bin)
            .map_err(|e| format!("Failed to extract {}: {}", first_bin, e))?;

        let mut rom_data = Vec::new();
        bin_file
            .read_to_end(&mut rom_data)
            .map_err(|e| format!("Failed to read {}: {}", first_bin, e))?;

        tracing::info!("Extracted {} bytes from {}", rom_data.len(), first_bin);

        Ok(rom_data)
    } else {
        // Not a zip, read directly
        std::fs::read(rom_path).map_err(|e| format!("Failed to read ROM file: {}", e))
    }
}

pub struct Debugger {
    pub psx: Psx,
    channel_send: Sender<DebuggerEvent>,
    channel_recv: Receiver<DebuggerEvent>,
    is_running: bool,
    ignore_errors: bool,
    trace: VecDeque<(u32, Instruction)>,
    breakpoints: HashSet<u32>,
    sideload_exe: Option<Vec<u8>>,
    bin_file: Option<Vec<u8>>,
    bios: Vec<u8>,
    cycle_counter: u32,
    frame_count: usize,
    fps_timer: std::time::Instant,
    current_fps: f64,
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
            bin_file: None,
            bios,
            cycle_counter: 0,
            frame_count: 0,
            fps_timer: std::time::Instant::now(),
            current_fps: 0.0,
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
            let bin = load_rom(&cdrom_path)
                .unwrap_or_else(|e| panic!("Failed to load CD-ROM image file '{}': {}", cdrom_path, e));
            self.bin_file = Some(bin.clone());
            self.psx.load_cdrom(bin);
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
                    Ok((instr, frame_complete)) => {
                        // Push to trace
                        self.trace.push_back((old_pc, instr));
                        if self.trace.len() > 1000 {
                            self.trace.pop_front();
                        }

                        // Update FPS tracking
                        if frame_complete {
                            self.frame_count += 1;
                            let elapsed = self.fps_timer.elapsed().as_secs_f64();
                            if elapsed >= 1.0 {
                                self.current_fps = self.frame_count as f64 / elapsed;
                                self.frame_count = 0;
                                self.fps_timer = std::time::Instant::now();
                            }
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
                                    fps: self.current_fps,
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
                            cop2: self.psx.cpu.cop2,
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

                    if let Some(bin) = &self.bin_file {
                        self.psx.load_cdrom(bin.clone());
                    }

                    self.is_running = false;
                    self.trace.clear();
                    self.frame_count = 0;
                    self.fps_timer = std::time::Instant::now();
                    self.current_fps = 0.0;

                    psx_core::cpu::internal::tty_buffer().lock().unwrap().clear();
                    psx_core::cpu::internal::tty_buffer().lock().unwrap().clear();

                    self.channel_send
                        .send(DebuggerEvent::CpuUpdated(CpuState {
                            pc: self.psx.cpu.pc,
                            registers: self.psx.cpu.registers.clone(),
                            cop0: self.psx.cpu.cop0,
                            cop2: self.psx.cpu.cop2,
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
