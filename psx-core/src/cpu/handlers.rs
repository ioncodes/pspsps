use crate::cpu::Cpu;
use crate::cpu::decoder::Instruction;
use crate::mmu::Mmu;
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
    instr: &Instruction, cpu: &mut Cpu, _mmu: &mut Mmu,
) {
    // TODO: op 0 technically is a NOP

    let shift = |x: u32, y: u32| match DIRECTION {
        ShiftDirection::Left => x << y,
        ShiftDirection::Right => {
            if TYPE == ShiftType::Logical {
                x >> y
            } else {
                (x as i32 >> y) as u32
            }
        }
    };

    let shift_amount = if VARIABLE { instr.rs() } else { instr.sa() } as u32;
    let value = cpu.registers[instr.rt() as usize];

    let result = shift(value, shift_amount);
    cpu.registers[instr.rd() as usize] = result;
}

pub fn branch<const LINK: bool, const REGISTER: bool, const TYPE: BranchType>(
    _instr: &Instruction, _cpu: &mut Cpu, _mmu: &mut Mmu,
) {
    todo!(
        "Implement branch operation with link: {}, register: {}, type: {:?}",
        LINK,
        REGISTER,
        TYPE
    );
}

pub fn alu<const OPERATION: AluOperation, const UNSIGNED: bool, const IMMEDIATE: bool>(
    instr: &Instruction, cpu: &mut Cpu, _mmu: &mut Mmu,
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
    instr: &Instruction, cpu: &mut Cpu, mmu: &mut Mmu,
) {
    if IS_LUI {
        let imm = instr.immediate() as u32;
        cpu.registers[instr.rt() as usize] = imm << 16;
        return;
    }

    let base = cpu.registers[instr.rs() as usize];
    let offset = instr.offset();
    let vaddr = base.wrapping_add_signed(offset as i32);

    match TRANSFER_SIZE {
        MemoryTransferSize::Byte if TYPE == MemoryAccessType::Load => {
            cpu.registers[instr.rt() as usize] = mmu.read_u8(vaddr) as u32;
        }
        MemoryTransferSize::Byte if TYPE == MemoryAccessType::Store => {
            mmu.write_u8(vaddr, (cpu.registers[instr.rt() as usize] & 0xFF) as u8);
        }
        MemoryTransferSize::HalfWord if TYPE == MemoryAccessType::Load => {
            cpu.registers[instr.rt() as usize] = mmu.read_u16(vaddr) as u32;
        }
        MemoryTransferSize::HalfWord if TYPE == MemoryAccessType::Store => {
            mmu.write_u16(vaddr, (cpu.registers[instr.rt() as usize] & 0xFFFF) as u16);
        }
        MemoryTransferSize::Word if TYPE == MemoryAccessType::Load => {
            cpu.registers[instr.rt() as usize] = mmu.read_u32(vaddr);
        }
        MemoryTransferSize::Word if TYPE == MemoryAccessType::Store => {
            mmu.write_u32(vaddr, cpu.registers[instr.rt() as usize]);
        }
        _ => todo!(
            "Implement load/store operation with type: {:?}, transfer size: {:?}, portion: {:?}",
            TYPE,
            TRANSFER_SIZE,
            PORTION
        ),
    }
}

pub fn move_multiply<
    const DIRECTION: MultiplyMoveDirection,
    const REGISTER: MultiplyMoveRegister,
>(
    _instr: &Instruction, _cpu: &mut Cpu, _mmu: &mut Mmu,
) {
    todo!(
        "Implement move multiply operation with direction: {:?}, register: {:?}",
        DIRECTION,
        REGISTER
    );
}

pub fn cop(_instr: &Instruction, _cpu: &mut Cpu, _mmu: &mut Mmu) {
    todo!("Implement COP");
}

pub fn system_call(_instr: &Instruction, _cpu: &mut Cpu, _mmu: &mut Mmu) {
    todo!("Implement system call");
}

pub fn debug_break(_instr: &Instruction, _cpu: &mut Cpu, _mmu: &mut Mmu) {
    todo!("Implement break");
}
