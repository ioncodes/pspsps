pub mod breakpoints;
pub mod cdrom;
pub mod cpu;
pub mod dma;
pub mod gpu;
pub mod irq;
pub mod mmu;
pub mod timers;
pub mod trace;
pub mod tty;

pub struct State {
    pub cpu: cpu::CpuState,
    pub previous_cpu: cpu::CpuState,
    pub mmu: mmu::MmuState,
    pub tty: tty::TtyState,
    pub trace: trace::TraceState,
    pub breakpoints: breakpoints::BreakpointsState,
    pub gpu: gpu::GpuState,
    pub timers: timers::TimersState,
    pub cdrom: cdrom::CdromState,
    pub dma: dma::DmaState,
    pub irq: irq::IrqState,
    pub is_running: bool,
    pub ignore_errors: bool,
    pub should_update_previous_cpu: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            cpu: cpu::CpuState::default(),
            previous_cpu: cpu::CpuState::default(),
            mmu: mmu::MmuState::default(),
            tty: tty::TtyState::default(),
            trace: trace::TraceState::default(),
            breakpoints: breakpoints::BreakpointsState::default(),
            gpu: gpu::GpuState::default(),
            timers: timers::TimersState::default(),
            cdrom: cdrom::CdromState::default(),
            dma: dma::DmaState::default(),
            irq: irq::IrqState::default(),
            is_running: false,
            ignore_errors: true,
            should_update_previous_cpu: false,
        }
    }
}
