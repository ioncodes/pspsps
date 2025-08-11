use crate::cpu::Cpu;
use crate::cpu::decoder::Instruction;
use std::marker::ConstParamTy;

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum ShiftType {
    Logical,
    Arithmetic,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum ShiftDirection {
    Left,
    Right,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum BranchType {
    Unconditional,
    Equal,
    NotEqual,
    LessEqualZero,
    LessThanZero,
    BranchGreaterEqualZero,
    BranchGreaterThanZero,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum AluOperation {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Nor,
    Multiply,
    Divide,
    SetLessThan,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum MemoryAccessType {
    Load,
    Store,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum MemoryTransferSize {
    Byte,
    HalfWord,
    Word,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum MemoryAccessPortion {
    Full,
    Left,
    Right,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum MultiplyMoveDirection {
    ToRegister,
    FromRegister,
}

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum MultiplyMoveRegister {
    Hi,
    Lo,
}

pub fn shift<const DIRECTION: ShiftDirection, const TYPE: ShiftType, const VARIABLE: bool>(
    _instr: &Instruction, _cpu: &mut Cpu,
) {
    todo!(
        "Implement shift operation with direction: {:?}, type: {:?}, variable: {}",
        DIRECTION,
        TYPE,
        VARIABLE
    );
}

pub fn branch<const LINK: bool, const REGISTER: bool, const TYPE: BranchType>(
    _instr: &Instruction, _cpu: &mut Cpu,
) {
    todo!(
        "Implement branch operation with link: {}, register: {}, type: {:?}",
        LINK,
        REGISTER,
        TYPE
    );
}

pub fn alu<const OPERATION: AluOperation, const UNSIGNED: bool, const IMMEDIATE: bool>(
    instr: &Instruction, cpu: &mut Cpu,
) {
    let x = cpu.registers[instr.rs() as usize];
    let y = if IMMEDIATE {
        instr.immediate() as u32
    } else {
        cpu.registers[instr.rt() as usize]
    };
    let dst = if IMMEDIATE { instr.rt() } else { instr.rd() } as usize;

    match OPERATION {
        AluOperation::Or => cpu.registers[dst] = x | y,
        AluOperation::And => cpu.registers[dst] = x & y,
        AluOperation::Xor => cpu.registers[dst] = x ^ y,
        AluOperation::Nor => cpu.registers[dst] = !(x | y),
        AluOperation::Add => cpu.registers[dst] = x.wrapping_add(y),
        AluOperation::Sub => cpu.registers[dst] = x.wrapping_sub(y),
        _ => todo!(
            "Implement ALU operation: {:?}, unsigned: {}, immediate: {}",
            OPERATION,
            UNSIGNED,
            IMMEDIATE
        ),
    }
}

pub fn load_store<
    const IS_LUI: bool,
    const TYPE: MemoryAccessType,
    const TRANSFER_SIZE: MemoryTransferSize,
    const PORTION: MemoryAccessPortion,
>(
    instr: &Instruction, cpu: &mut Cpu,
) {
    if IS_LUI {
        let imm = instr.immediate() as u32;
        cpu.registers[instr.rt() as usize] = imm << 16;
        return;
    }

    todo!(
        "Implement load/store operation with type: {:?}, transfer size: {:?}, portion: {:?}",
        TYPE,
        TRANSFER_SIZE,
        PORTION
    );
}

pub fn move_multiply<
    const DIRECTION: MultiplyMoveDirection,
    const REGISTER: MultiplyMoveRegister,
>(
    _instr: &Instruction, _cpu: &mut Cpu,
) {
    todo!(
        "Implement move multiply operation with direction: {:?}, register: {:?}",
        DIRECTION,
        REGISTER
    );
}

pub fn cop(_instr: &Instruction, _cpu: &mut Cpu) {
    todo!("Implement COP");
}

pub fn system_call(_instr: &Instruction, _cpu: &mut Cpu) {
    todo!("Implement system call");
}

pub fn debug_break(_instr: &Instruction, _cpu: &mut Cpu) {
    todo!("Implement break");
}
