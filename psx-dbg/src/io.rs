use psx_core::cpu::decoder::Instruction;
use std::collections::VecDeque;

use crate::states::breakpoints::BreakpointsState;
use crate::states::cpu::CpuState;
use crate::states::mmu::MmuState;
use crate::states::trace::TraceState;
use crate::states::tty::TtyState;

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
    BreakpointHit(u32),
    BreakpointsUpdated(BreakpointsState),
    TraceUpdated(TraceState),
    TtyUpdated(TtyState),
    CpuUpdated(CpuState),
    MmuUpdated(MmuState),
}
