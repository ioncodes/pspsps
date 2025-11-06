use crate::cpu::Cpu;
use crate::cpu::cop::Cop;
use crate::cpu::cop::cop0::COP0_EXCEPTION_CODE_SYSCALL;
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
pub enum BranchAddressing {
    AbsoluteImmediate,
    RelativeOffset,
    AbsoluteRegister,
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

#[derive(Debug, ConstParamTy, PartialEq, Eq)]
pub enum CopOperation {
    MoveTo,
    MoveFrom,
    MoveControlTo,
    MoveControlFrom,
    ReturnFromException,
}

pub fn shift<const DIRECTION: ShiftDirection, const TYPE: ShiftType, const VARIABLE: bool>(
    instr: &Instruction, cpu: &mut Cpu,
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

    let shift_amount = if VARIABLE {
        cpu.registers[instr.rs() as usize] // TODO: is this correct?
    } else {
        instr.shamt() as u32
    };
    let value = cpu.registers[instr.rt() as usize];

    let result = shift(value, shift_amount);
    cpu.registers[instr.rd() as usize] = result;
}

pub fn branch<
    const LINK: bool,
    const LINK_REGISTER_DEFINED: bool,
    const TYPE: BranchType,
    const ADDRESSING: BranchAddressing,
>(
    instr: &Instruction, cpu: &mut Cpu,
) {
    let compare = |x: u32, y: u32| match TYPE {
        BranchType::Equal => x == y,
        BranchType::NotEqual => x != y,
        BranchType::LessThanZero => (x as i32) < 0,
        BranchType::LessEqualZero => (x as i32) <= 0,
        BranchType::BranchGreaterEqualZero => (x as i32) >= 0,
        BranchType::BranchGreaterThanZero => (x as i32) > 0,
        _ => true, // Unconditional branches do not require comparison
    };

    let perform_branch = compare(
        cpu.registers[instr.rs() as usize],
        cpu.registers[instr.rt() as usize],
    );

    if perform_branch {
        // return address = PC + 8, where:
        // PC + 0 = current instruction
        // PC + 4 = next instruction (delay slot)
        // PC + 8 = instruction after the delay slot
        let return_address = cpu.pc + 8;

        // Instructions like jal store the return address in reg 31 by default
        // however, for instructions like JALR the reg is explicit (and may be different)
        if LINK && !LINK_REGISTER_DEFINED {
            cpu.registers[31] = return_address;
        }

        let branch_target = match ADDRESSING {
            BranchAddressing::AbsoluteImmediate => {
                (instr.address() << 2) | ((cpu.pc + 4) & 0xF000_0000)
            }
            BranchAddressing::AbsoluteRegister => cpu.registers[instr.rs() as usize],
            BranchAddressing::RelativeOffset => {
                cpu.pc.wrapping_add_signed((instr.offset() as i32) << 2) + 4 // +4 to account for the delay slot
            }
        };

        if LINK && LINK_REGISTER_DEFINED {
            cpu.registers[instr.rd() as usize] = return_address;
        }

        cpu.set_delay_slot(branch_target);
    }
}

pub fn alu<const OPERATION: AluOperation, const UNSIGNED: bool, const IMMEDIATE: bool>(
    instr: &Instruction, cpu: &mut Cpu,
) {
    // TODO: UNSIGNED = no exception

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
        AluOperation::Add if IMMEDIATE => {
            cpu.registers[dst] = x.wrapping_add_signed(y as i16 as i32)
        }
        AluOperation::Add if !IMMEDIATE => cpu.registers[dst] = x.wrapping_add(y),
        AluOperation::Sub if IMMEDIATE => {
            cpu.registers[dst] = x.wrapping_sub_signed(y as i16 as i32)
        }
        AluOperation::Sub if !IMMEDIATE => cpu.registers[dst] = x.wrapping_sub(y),
        AluOperation::Multiply => {
            let result = (x as i32).wrapping_mul(y as i32) as i64;
            cpu.hi = (result >> 32) as u32;
            cpu.lo = (result & 0xFFFFFFFF) as u32;
        }
        AluOperation::Divide => {
            if y == 0 {
                cpu.hi = 0;
                cpu.lo = 0;
            } else {
                cpu.lo = x.wrapping_div(y);
                cpu.hi = x.wrapping_rem(y);
            }
        }
        AluOperation::SetLessThan => {
            cpu.registers[dst] = if (x as i32) < (y as i32) { 1 } else { 0 }
        }
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

    let base = cpu.registers[instr.rs() as usize];
    let offset = instr.offset();
    let vaddr = base.wrapping_add_signed(offset as i32);

    match TRANSFER_SIZE {
        MemoryTransferSize::Byte if TYPE == MemoryAccessType::Load => {
            cpu.registers[instr.rt() as usize] = cpu.read_u8(vaddr) as u32;
        }
        MemoryTransferSize::Byte if TYPE == MemoryAccessType::Store => {
            cpu.write_u8(vaddr, (cpu.registers[instr.rt() as usize] & 0xFF) as u8);
        }
        MemoryTransferSize::HalfWord if TYPE == MemoryAccessType::Load => {
            cpu.registers[instr.rt() as usize] = cpu.read_u16(vaddr) as u32;
        }
        MemoryTransferSize::HalfWord if TYPE == MemoryAccessType::Store => {
            cpu.write_u16(vaddr, (cpu.registers[instr.rt() as usize] & 0xFFFF) as u16);
        }
        MemoryTransferSize::Word if TYPE == MemoryAccessType::Load => {
            cpu.registers[instr.rt() as usize] = cpu.read_u32(vaddr);
        }
        MemoryTransferSize::Word if TYPE == MemoryAccessType::Store => {
            cpu.write_u32(vaddr, cpu.registers[instr.rt() as usize]);
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
    instr: &Instruction, cpu: &mut Cpu,
) {
    match DIRECTION {
        MultiplyMoveDirection::ToRegister => match REGISTER {
            MultiplyMoveRegister::Hi => cpu.hi = cpu.registers[instr.rs() as usize],
            MultiplyMoveRegister::Lo => cpu.lo = cpu.registers[instr.rs() as usize],
        },
        MultiplyMoveDirection::FromRegister => match REGISTER {
            MultiplyMoveRegister::Hi => cpu.registers[instr.rd() as usize] = cpu.hi,
            MultiplyMoveRegister::Lo => cpu.registers[instr.rd() as usize] = cpu.lo,
        },
    }
}

pub fn cop<const OPERATION: CopOperation>(instr: &Instruction, cpu: &mut Cpu) {
    match OPERATION {
        CopOperation::MoveTo | CopOperation::MoveControlTo => {
            cpu.cop0
                .write_register(instr.rd() as u32, cpu.registers[instr.rt() as usize]);
        }
        CopOperation::MoveFrom | CopOperation::MoveControlFrom => {
            cpu.registers[instr.rt() as usize] = cpu.cop0.read_register(instr.rd() as u32);
        }
        CopOperation::ReturnFromException => {
            cpu.restore_from_exception();
        }
    }
}

pub fn system_call(_instr: &Instruction, cpu: &mut Cpu) {
    let function_number = cpu.registers[4]; // BIOS function number is in $a0 (reg 4)
    tracing::debug!(target: "psx_core::cpu", "syscall({:08X})", function_number);
    cpu.cause_exception(COP0_EXCEPTION_CODE_SYSCALL);
}

pub fn debug_break(_instr: &Instruction, _cpu: &mut Cpu) {
    todo!("Implement break");
}
