use psx_core::cpu::decoder::Instruction;
use std::collections::VecDeque;

pub struct TraceState {
    pub instructions: VecDeque<(u32, Instruction)>,
}

impl Default for TraceState {
    fn default() -> Self {
        Self {
            instructions: VecDeque::with_capacity(1000),
        }
    }
}
