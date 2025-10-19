use crate::cpu::Cpu;
use crate::cpu::cop::Cop;
use crate::cpu::cop::cop0::Exception;
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
    GreaterEqualZero,
    GreaterThanZero,
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
        ShiftDirection::Left => x.wrapping_shl(y),
        ShiftDirection::Right => {
            if TYPE == ShiftType::Logical {
                x.wrapping_shr(y)
            } else {
                (x as i32).wrapping_shr(y) as u32
            }
        }
    };

    let shift_amount = if VARIABLE {
        cpu.read_register(instr.rs()) // TODO: is this correct?
    } else {
        instr.shamt() as u32
    };
    let value = cpu.read_register(instr.rt());

    let result = shift(value, shift_amount);
    cpu.write_register(instr.rd(), result);

    cpu.add_cycles(1);
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
        BranchType::GreaterEqualZero => (x as i32) >= 0,
        BranchType::GreaterThanZero => (x as i32) > 0,
        BranchType::Unconditional => true, // Unconditional branches do not require comparison
    };

    // return address = PC + 8, where:
    // PC + 0 = current instruction
    // PC + 4 = next instruction (delay slot)
    // PC + 8 = instruction after the delay slot
    let return_address = cpu.pc + 8;

    // Cache rs, in case rs == rd as rs takes precedence over rd
    let rs_value = cpu.read_register(instr.rs());

    // Instructions like jal store the return address in reg 31 by default
    // however, for instructions like JALR the reg is explicit (and may be different)
    // This happens unconditionally (regardless of branch taken or not), even if there is an exception
    if LINK {
        let link_register = if LINK_REGISTER_DEFINED {
            instr.rd()
        } else {
            crate::regidx("$ra")
        };
        cpu.write_register(link_register, return_address);
    }

    let perform_branch = compare(rs_value, cpu.read_register(instr.rt()));

    if perform_branch {
        let branch_target = match ADDRESSING {
            BranchAddressing::AbsoluteImmediate => {
                (instr.address() << 2) | ((cpu.pc + 4) & 0xF000_0000)
            }
            BranchAddressing::AbsoluteRegister => {
                if !Mmu::is_word_aligned(rs_value) {
                    cpu.cause_exception(Exception::AddressErrorLoad, instr.is_delay_slot);
                    cpu.add_cycles(1);
                    return;
                }
                rs_value
            }
            BranchAddressing::RelativeOffset => {
                cpu.pc.wrapping_add_signed((instr.offset() as i32) << 2) + 4 // +4 to account for the delay slot
            }
        };

        cpu.set_delay_slot(branch_target);
    } else {
        // If the branch is not taken, we still need to set the delay slot
        // to the next instruction, which is PC + 4
        // with the branch target being PC + 8 (the instruction after the delay slot)
        // TODO: Verify if this is correct and does not cause weird side effects with the debugger or something
        cpu.set_delay_slot(cpu.pc + 8);
    }

    cpu.add_cycles(1);
}

pub fn alu<const OPERATION: AluOperation, const UNSIGNED: bool, const IMMEDIATE: bool>(
    instr: &Instruction, cpu: &mut Cpu,
) {
    let x = cpu.read_register(instr.rs());
    let y = if IMMEDIATE {
        // Sign-extend for arithmetic operations, zero-extend for logical
        match OPERATION {
            AluOperation::Add | AluOperation::Sub | AluOperation::SetLessThan => {
                instr.immediate() as i16 as i32 as u32 // Sign-extend
            }
            _ => {
                instr.immediate() as u32 // Zero-extend for logical ops
            }
        }
    } else {
        cpu.read_register(instr.rt())
    };
    let dst = if IMMEDIATE { instr.rt() } else { instr.rd() };

    match OPERATION {
        AluOperation::Or => cpu.write_register(dst, x | y),
        AluOperation::And => cpu.write_register(dst, x & y),
        AluOperation::Xor => cpu.write_register(dst, x ^ y),
        AluOperation::Nor => cpu.write_register(dst, !(x | y)),
        AluOperation::Add if IMMEDIATE => {
            if UNSIGNED {
                cpu.write_register(dst, x.wrapping_add_signed(y as i16 as i32));
            } else {
                match (x as i32).checked_add(y as i16 as i32) {
                    Some(result) => cpu.write_register(dst, result as u32),
                    None => cpu.cause_exception(Exception::ArithmeticOverflow, instr.is_delay_slot),
                }
            }
        }
        AluOperation::Add if !IMMEDIATE => {
            if UNSIGNED {
                cpu.write_register(dst, x.wrapping_add(y));
            } else {
                match (x as i32).checked_add(y as i32) {
                    Some(result) => cpu.write_register(dst, result as u32),
                    None => cpu.cause_exception(Exception::ArithmeticOverflow, instr.is_delay_slot),
                }
            }
        }
        AluOperation::Sub if IMMEDIATE => {
            if UNSIGNED {
                cpu.write_register(dst, x.wrapping_sub_signed(y as i16 as i32));
            } else {
                match (x as i32).checked_sub(y as i16 as i32) {
                    Some(result) => cpu.write_register(dst, result as u32),
                    None => cpu.cause_exception(Exception::ArithmeticOverflow, instr.is_delay_slot),
                }
            }
        }
        AluOperation::Sub if !IMMEDIATE => {
            if UNSIGNED {
                cpu.write_register(dst, x.wrapping_sub(y));
            } else {
                match (x as i32).checked_sub(y as i32) {
                    Some(result) => cpu.write_register(dst, result as u32),
                    None => cpu.cause_exception(Exception::ArithmeticOverflow, instr.is_delay_slot),
                }
            }
        }
        AluOperation::Multiply => {
            let result = if UNSIGNED {
                (x as u32 as u64).wrapping_mul(y as u32 as u64)
            } else {
                (x as i32 as i64).wrapping_mul(y as i32 as i64) as u64
            };
            cpu.hi = (result >> 32) as u32;
            cpu.lo = result as u32;

            let cycles = if UNSIGNED {
                match x {
                    0x00000000..=0x000007FF => 6,
                    0x00000800..=0x000FFFFF => 9,
                    0x00100000..=0xFFFFFFFF => 12,
                }
            } else {
                match x {
                    0x00000000..=0x000007FF | 0xFFFFF800..=0xFFFFFFFF => 6,
                    0x00000800..=0x000FFFFF | 0xFFF00000..=0xFFFFF801 => 9,
                    0x00100000..=0x7FFFFFFF | 0x80000000..=0xFFF00001 => 12,
                }
            };

            cpu.add_cycles(cycles);
            return;
        }
        // https://gitlab.com/flio/rustation-ng/-/blob/master/src/psx/cpu.rs?ref_type=heads#L793
        AluOperation::Divide if UNSIGNED => {
            if y == 0 {
                cpu.lo = 0xFFFF_FFFF;
                cpu.hi = x;
            } else {
                cpu.lo = x.wrapping_div(y);
                cpu.hi = x.wrapping_rem(y);
            }
        }
        AluOperation::Divide if !UNSIGNED => {
            if y == 0 {
                if x as i32 >= 0 {
                    cpu.lo = 0xFFFF_FFFF;
                } else {
                    cpu.lo = 1;
                }
                cpu.hi = x;
            } else if x == 0x8000_0000 && y as i32 == -1 {
                cpu.lo = 0x8000_0000;
                cpu.hi = 0;
            } else {
                cpu.lo = (x as i32).wrapping_div(y as i32) as u32;
                cpu.hi = (x as i32).wrapping_rem(y as i32) as u32;
            }
        }
        AluOperation::SetLessThan => {
            let result = if UNSIGNED {
                x < y
            } else {
                (x as i32) < (y as i32)
            };

            cpu.write_register(dst, if result { 1 } else { 0 });
        }
        _ => todo!(
            "Implement ALU operation: {:?}, unsigned: {}, immediate: {}",
            OPERATION,
            UNSIGNED,
            IMMEDIATE
        ),
    }

    cpu.add_cycles(1);
}

pub fn load_store<
    const IS_LUI: bool,
    const TYPE: MemoryAccessType,
    const TRANSFER_SIZE: MemoryTransferSize,
    const PORTION: MemoryAccessPortion,
    const SIGNED: bool,
>(
    instr: &Instruction, cpu: &mut Cpu,
) {
    if IS_LUI {
        let imm = instr.immediate() as u32;
        cpu.write_register(instr.rt(), imm << 16);
        cpu.add_cycles(1);
        return;
    }

    let base = cpu.read_register(instr.rs());
    let offset = instr.offset();
    let vaddr = base.wrapping_add_signed(offset as i32);

    match TRANSFER_SIZE {
        MemoryTransferSize::Byte if TYPE == MemoryAccessType::Load => {
            let value = if SIGNED {
                cpu.read_u8(vaddr) as i8 as i32 as u32 // Sign-extend
            } else {
                cpu.read_u8(vaddr) as u32 // Zero-extend
            };
            cpu.schedule_load(instr.rt(), value);
            cpu.add_cycles(2);
        }
        MemoryTransferSize::Byte if TYPE == MemoryAccessType::Store => {
            cpu.write_u8(vaddr, (cpu.read_register(instr.rt()) & 0xFF) as u8);
            cpu.add_cycles(1);
        }
        MemoryTransferSize::HalfWord if TYPE == MemoryAccessType::Load => {
            if vaddr % 2 != 0 {
                cpu.cause_exception(Exception::AddressErrorLoad, instr.is_delay_slot);
                return;
            }

            let value = if SIGNED {
                cpu.read_u16(vaddr) as i16 as i32 as u32 // Sign-extend
            } else {
                cpu.read_u16(vaddr) as u32 // Zero-extend
            };
            cpu.schedule_load(instr.rt(), value);
            cpu.add_cycles(2);
        }
        MemoryTransferSize::HalfWord if TYPE == MemoryAccessType::Store => {
            if vaddr % 2 != 0 {
                cpu.cause_exception(Exception::AddressErrorStore, instr.is_delay_slot);
                return;
            }

            cpu.write_u16(vaddr, (cpu.read_register(instr.rt()) & 0xFFFF) as u16);
            cpu.add_cycles(1);
        }
        MemoryTransferSize::Word
            if TYPE == MemoryAccessType::Load && PORTION == MemoryAccessPortion::Full =>
        {
            if vaddr % 4 != 0 {
                cpu.cause_exception(Exception::AddressErrorLoad, instr.is_delay_slot);
                return;
            }

            let value = cpu.read_u32(vaddr);
            cpu.schedule_load(instr.rt(), value);
            cpu.add_cycles(2);
        }
        MemoryTransferSize::Word
            if TYPE == MemoryAccessType::Store && PORTION == MemoryAccessPortion::Full =>
        {
            if vaddr % 4 != 0 {
                cpu.cause_exception(Exception::AddressErrorStore, instr.is_delay_slot);
                return;
            }

            cpu.write_u32(vaddr, cpu.read_register(instr.rt()));
            cpu.add_cycles(1);
        }
        MemoryTransferSize::Word
            if TYPE == MemoryAccessType::Load && PORTION != MemoryAccessPortion::Full =>
        {
            // LWL, LWR, SWL, SWR do *not* throw exceptions for address alignment!

            // LWL: "It reads bytes only from the word in memory which contains the specified starting byte."
            // this many bytes in word boundary -> subtract from/add to vaddr when reading
            let bytes_to_read = match PORTION {
                MemoryAccessPortion::Left => Mmu::word_align(vaddr) + 1,
                MemoryAccessPortion::Right => 4 - Mmu::word_align(vaddr),
                _ => unreachable!(),
            };
            let mut register_value = cpu.read_register_with_pending_load(instr.rt()); // LWL/LWR can read from pending load delays

            for idx in 0..bytes_to_read {
                let shift = match PORTION {
                    MemoryAccessPortion::Left => 8 * (3 - idx),
                    MemoryAccessPortion::Right => 8 * idx,
                    _ => unreachable!(),
                };
                let mask = 0xFFu32 << shift;

                let vaddr = match PORTION {
                    MemoryAccessPortion::Left => vaddr - idx,
                    MemoryAccessPortion::Right => vaddr + idx,
                    _ => unreachable!(),
                };
                let value = cpu.read_u8(vaddr);

                register_value = (register_value & !mask) | ((value as u32) << shift);
            }

            cpu.schedule_load(instr.rt(), register_value);
            cpu.add_cycles(2);
        }
        MemoryTransferSize::Word
            if TYPE == MemoryAccessType::Store && PORTION != MemoryAccessPortion::Full =>
        {
            let bytes_to_write = match PORTION {
                MemoryAccessPortion::Left => Mmu::word_align(vaddr) + 1,
                MemoryAccessPortion::Right => 4 - Mmu::word_align(vaddr),
                _ => unreachable!(),
            };
            let register_value = cpu.read_register(instr.rt());

            for idx in 0..bytes_to_write {
                let shift = match PORTION {
                    MemoryAccessPortion::Left => 8 * (3 - idx),
                    MemoryAccessPortion::Right => 8 * idx,
                    _ => unreachable!(),
                };
                let vaddr = match PORTION {
                    MemoryAccessPortion::Left => vaddr - idx,
                    MemoryAccessPortion::Right => vaddr + idx,
                    _ => unreachable!(),
                };
                cpu.write_u8(vaddr, (register_value >> shift) as u8);
            }

            cpu.add_cycles(2);
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
            MultiplyMoveRegister::Hi => cpu.hi = cpu.read_register(instr.rs()),
            MultiplyMoveRegister::Lo => cpu.lo = cpu.read_register(instr.rs()),
        },
        MultiplyMoveDirection::FromRegister => match REGISTER {
            MultiplyMoveRegister::Hi => cpu.write_register(instr.rd(), cpu.hi),
            MultiplyMoveRegister::Lo => cpu.write_register(instr.rd(), cpu.lo),
        },
    }

    cpu.add_cycles(1);
}

pub fn cop<const OPERATION: CopOperation>(instr: &Instruction, cpu: &mut Cpu) {
    match OPERATION {
        CopOperation::MoveTo | CopOperation::MoveControlTo => {
            cpu.cop0
                .write_register(instr.rd() as u32, cpu.read_register(instr.rt()));
            cpu.add_cycles(1);
        }
        CopOperation::MoveFrom | CopOperation::MoveControlFrom => {
            cpu.schedule_load(instr.rt(), cpu.cop0.read_register(instr.rd() as u32));
            cpu.add_cycles(2);
        }
        CopOperation::ReturnFromException => {
            cpu.restore_from_exception();
            cpu.add_cycles(1);
        }
    }
}

pub fn system_call(instr: &Instruction, cpu: &mut Cpu) {
    let function_number = cpu.read_register(crate::regidx("$a0")); // BIOS function number is in $a0 (reg 4)
    tracing::debug!(target: "psx_core::cpu", "syscall({:08X})", function_number);
    cpu.cause_exception(Exception::Syscall, instr.is_delay_slot);
    cpu.add_cycles(1);
}

pub fn debug_break(instr: &Instruction, cpu: &mut Cpu) {
    cpu.cause_exception(Exception::Breakpoint, instr.is_delay_slot);
    cpu.add_cycles(1);
}
