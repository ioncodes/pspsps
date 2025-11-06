use egui::Ui;
use psx_core::psx::Psx;
use psx_core::cpu::decoder::Instruction;
use std::collections::{HashSet, VecDeque};

pub mod breakpoints;
pub mod cpu;
pub mod mmu;
pub mod trace;
pub mod tty;

pub use breakpoints::BreakpointsWidget;
pub use cpu::CpuWidget;
pub use mmu::MmuWidget;
pub use trace::TraceWidget;
pub use tty::TtyWidget;

pub trait Widget {
    fn title(&self) -> &str;
    fn ui(&mut self, ui: &mut Ui, shared_context: &mut SharedContext);
}

pub struct SharedContext<'a> {
    pub psx: &'a mut Psx,
    pub is_running: &'a mut bool,

    // Breakpoints
    pub breakpoints: &'a mut HashSet<u32>,
    pub breakpoint_hit: &'a mut bool,
    pub show_in_disassembly: &'a mut Option<u32>,
    
    // Trace
    pub trace_buffer: &'a mut VecDeque<(u32, Instruction)>,
}
