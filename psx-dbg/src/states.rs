pub mod breakpoints;
pub mod cpu;
pub mod gpu;
pub mod mmu;
pub mod trace;
pub mod tty;

pub struct State {
    pub cpu: cpu::CpuState,
    pub mmu: mmu::MmuState,
    pub tty: tty::TtyState,
    pub trace: trace::TraceState,
    pub breakpoints: breakpoints::BreakpointsState,
    pub gpu: gpu::GpuState,
    pub is_running: bool,
    pub ignore_errors: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            cpu: cpu::CpuState::default(),
            mmu: mmu::MmuState::default(),
            tty: tty::TtyState::default(),
            trace: trace::TraceState::default(),
            breakpoints: breakpoints::BreakpointsState::default(),
            gpu: gpu::GpuState::default(),
            is_running: false,
            ignore_errors: true,
        }
    }
}
