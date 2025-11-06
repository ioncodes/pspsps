use crate::cpu::{Cpu, decoder::Instruction};

pub fn unimplemented_gte(instr: &Instruction, cpu: &mut Cpu) {
    todo!("GTE instruction dispatched: {}", instr);
}