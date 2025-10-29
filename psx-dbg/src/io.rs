use crate::states::breakpoints::BreakpointsState;
use crate::states::cpu::CpuState;
use crate::states::gpu::GpuState;
use crate::states::mmu::MmuState;
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
    UpdateController(ControllerState),
    BreakpointHit(u32),
    BreakpointsUpdated(BreakpointsState),
    TraceUpdated(TraceState),
    TtyUpdated(TtyState),
    CpuUpdated(CpuState),
    MmuUpdated(MmuState),
    GpuUpdated(GpuState),
}
