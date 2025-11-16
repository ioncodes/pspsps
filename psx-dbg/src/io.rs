use crate::states::breakpoints::BreakpointsState;
use crate::states::cdrom::CdromState;
use crate::states::cpu::CpuState;
use crate::states::dma::DmaState;
use crate::states::gpu::GpuState;
use crate::states::irq::IrqState;
use crate::states::mmu::MmuState;
use crate::states::timers::TimersState;
use crate::states::trace::TraceState;
use crate::states::tty::TtyState;
use psx_core::sio::joy::ControllerState;

pub enum DebuggerEvent {
    Step,
    Run,
    Pause,
    Paused,
    Unpaused,
    Reset,
    AddBreakpoint(u32),
    RemoveBreakpoint(u32),
    ClearBreakpoints,
    UpdateCpu,
    UpdateMmu,
    UpdateTrace,
    UpdateTty,
    UpdateTimers,
    UpdateCdrom,
    UpdateDma,
    UpdateIrq,
    UpdateController(ControllerState),
    SetIgnoreErrors(bool),
    BreakpointHit(u32),
    BreakpointsUpdated(BreakpointsState),
    TraceUpdated(TraceState),
    TtyUpdated(TtyState),
    CpuUpdated(CpuState),
    MmuUpdated(MmuState),
    GpuUpdated(GpuState),
    TimersUpdated(TimersState),
    CdromUpdated(CdromState),
    DmaUpdated(DmaState),
    IrqUpdated(IrqState),
}
