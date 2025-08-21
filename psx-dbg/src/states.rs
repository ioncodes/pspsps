use psx_core::mmu::Addressable;

pub mod breakpoints;
pub mod cpu;
pub mod mmu;
pub mod trace;
pub mod tty;

pub struct State {
    pub cpu: cpu::CpuState,
    pub mmu: mmu::MmuState,
    pub tty: tty::TtyState,
    pub trace: trace::TraceState,
    pub breakpoints: breakpoints::BreakpointsState,
    pub is_running: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            cpu: cpu::CpuState::default(),
            mmu: mmu::MmuState::default(),
            tty: tty::TtyState::default(),
            trace: trace::TraceState::default(),
            breakpoints: breakpoints::BreakpointsState::default(),
            is_running: false,
        }
    }
}
