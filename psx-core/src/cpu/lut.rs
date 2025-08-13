use crate::cpu::decoder::Instruction;
use crate::cpu::handlers;

macro_rules! instruction {
    ($name:ident, $opcode_type:ident, $handler:expr) => {
        Instruction {
            opcode: crate::cpu::decoder::Opcode::$name,
            raw: 0,
            opcode_type: crate::cpu::decoder::InstructionType::$opcode_type,
            handler: $handler,
        }
    };
    ($name:ident($param:expr), $opcode_type:ident, $handler:expr) => {
        Instruction {
            opcode: crate::cpu::decoder::Opcode::Opcode::$name($param),
            raw: 0,
            opcode_type: crate::cpu::decoder::InstructionType::$opcode_type,
            handler: $handler,
        }
    };
}

// opcode = 0x00
pub static MIPS_RTYPE_LUT: [Instruction; 64] = [
    /* 0x00 */
    instruction!(
        ShiftLeftLogical,
        RType,
        handlers::shift::<
            { handlers::ShiftDirection::Left },
            { handlers::ShiftType::Logical },
            false,
        >
    ),
    /* 0x01 */ Instruction::invalid(),
    /* 0x02 */
    instruction!(
        ShiftRightLogical,
        RType,
        handlers::shift::<
            { handlers::ShiftDirection::Right },
            { handlers::ShiftType::Logical },
            false,
        >
    ),
    /* 0x03 */
    instruction!(
        ShiftRightArithmetic,
        RType,
        handlers::shift::<
            { handlers::ShiftDirection::Right },
            { handlers::ShiftType::Arithmetic },
            false,
        >
    ),
    /* 0x04 */
    instruction!(
        ShiftLeftLogicalVariable,
        RType,
        handlers::shift::<{ handlers::ShiftDirection::Left }, { handlers::ShiftType::Logical }, true>
    ),
    /* 0x05 */ Instruction::invalid(),
    /* 0x06 */
    instruction!(
        ShiftRightLogicalVariable,
        RType,
        handlers::shift::<
            { handlers::ShiftDirection::Right },
            { handlers::ShiftType::Logical },
            true,
        >
    ),
    /* 0x07 */
    instruction!(
        ShiftRightArithmeticVariable,
        RType,
        handlers::shift::<
            { handlers::ShiftDirection::Right },
            { handlers::ShiftType::Arithmetic },
            true,
        >
    ),
    /* 0x08 */
    instruction!(
        JumpRegister,
        RType,
        handlers::branch::<
            false,
            false,
            { handlers::BranchType::Unconditional },
            { handlers::BranchAddressing::AbsoluteRegister },
        >
    ),
    /* 0x09 */
    instruction!(
        JumpAndLinkRegister,
        RType,
        handlers::branch::<
            true,
            true,
            { handlers::BranchType::Unconditional },
            { handlers::BranchAddressing::AbsoluteRegister },
        >
    ),
    /* 0x0A */ Instruction::invalid(),
    /* 0x0B */ Instruction::invalid(),
    /* 0x0C */ instruction!(SystemCall, RType, handlers::system_call),
    /* 0x0D */ instruction!(Break, RType, handlers::debug_break),
    /* 0x0E */ Instruction::invalid(),
    /* 0x0F */ Instruction::invalid(),
    /* 0x10 */
    instruction!(
        MoveFromHi,
        RType,
        handlers::move_multiply::<
            { handlers::MultiplyMoveDirection::FromRegister },
            { handlers::MultiplyMoveRegister::Hi },
        >
    ),
    /* 0x11 */
    instruction!(
        MoveToHi,
        RType,
        handlers::move_multiply::<
            { handlers::MultiplyMoveDirection::ToRegister },
            { handlers::MultiplyMoveRegister::Hi },
        >
    ),
    /* 0x12 */
    instruction!(
        MoveFromLo,
        RType,
        handlers::move_multiply::<
            { handlers::MultiplyMoveDirection::FromRegister },
            { handlers::MultiplyMoveRegister::Lo },
        >
    ),
    /* 0x13 */
    instruction!(
        MoveToLo,
        RType,
        handlers::move_multiply::<
            { handlers::MultiplyMoveDirection::ToRegister },
            { handlers::MultiplyMoveRegister::Lo },
        >
    ),
    /* 0x14 */ Instruction::invalid(),
    /* 0x15 */ Instruction::invalid(),
    /* 0x16 */ Instruction::invalid(),
    /* 0x17 */ Instruction::invalid(),
    /* 0x18 */
    instruction!(
        Multiply,
        RType,
        handlers::alu::<{ handlers::AluOperation::Multiply }, false, false>
    ),
    /* 0x19 */
    instruction!(
        MultiplyUnsigned,
        RType,
        handlers::alu::<{ handlers::AluOperation::Multiply }, true, false>
    ),
    /* 0x1A */
    instruction!(
        Divide,
        RType,
        handlers::alu::<{ handlers::AluOperation::Divide }, false, false>
    ),
    /* 0x1B */
    instruction!(
        DivideUnsigned,
        RType,
        handlers::alu::<{ handlers::AluOperation::Divide }, true, false>
    ),
    /* 0x1C */ Instruction::invalid(),
    /* 0x1D */ Instruction::invalid(),
    /* 0x1E */ Instruction::invalid(),
    /* 0x1F */ Instruction::invalid(),
    /* 0x20 */
    instruction!(
        Add,
        RType,
        handlers::alu::<{ handlers::AluOperation::Add }, false, false>
    ),
    /* 0x21 */
    instruction!(
        AddUnsigned,
        RType,
        handlers::alu::<{ handlers::AluOperation::Add }, true, false>
    ),
    /* 0x22 */
    instruction!(
        Sub,
        RType,
        handlers::alu::<{ handlers::AluOperation::Sub }, false, false>
    ),
    /* 0x23 */
    instruction!(
        SubUnsigned,
        RType,
        handlers::alu::<{ handlers::AluOperation::Sub }, true, false>
    ),
    /* 0x24 */
    instruction!(
        And,
        RType,
        handlers::alu::<{ handlers::AluOperation::And }, false, false>
    ),
    /* 0x25 */
    instruction!(
        Or,
        RType,
        handlers::alu::<{ handlers::AluOperation::Or }, false, false>
    ),
    /* 0x26 */
    instruction!(
        Xor,
        RType,
        handlers::alu::<{ handlers::AluOperation::Xor }, false, false>
    ),
    /* 0x27 */
    instruction!(
        Nor,
        RType,
        handlers::alu::<{ handlers::AluOperation::Nor }, false, false>
    ),
    /* 0x28 */ Instruction::invalid(),
    /* 0x29 */ Instruction::invalid(),
    /* 0x2A */
    instruction!(
        SetLessThan,
        RType,
        handlers::alu::<{ handlers::AluOperation::SetLessThan }, false, false>
    ),
    /* 0x2B */
    instruction!(
        SetLessThanUnsigned,
        RType,
        handlers::alu::<{ handlers::AluOperation::SetLessThan }, true, false>
    ),
    /* 0x2C */ Instruction::invalid(),
    /* 0x2D */ Instruction::invalid(),
    /* 0x2E */ Instruction::invalid(),
    /* 0x2F */ Instruction::invalid(),
    /* 0x30 */ Instruction::invalid(),
    /* 0x31 */ Instruction::invalid(),
    /* 0x32 */ Instruction::invalid(),
    /* 0x33 */ Instruction::invalid(),
    /* 0x34 */ Instruction::invalid(),
    /* 0x35 */ Instruction::invalid(),
    /* 0x36 */ Instruction::invalid(),
    /* 0x37 */ Instruction::invalid(),
    /* 0x38 */ Instruction::invalid(),
    /* 0x39 */ Instruction::invalid(),
    /* 0x3A */ Instruction::invalid(),
    /* 0x3B */ Instruction::invalid(),
    /* 0x3C */ Instruction::invalid(),
    /* 0x3D */ Instruction::invalid(),
    /* 0x3E */ Instruction::invalid(),
    /* 0x3F */ Instruction::invalid(),
];

// opcode = 0x01 - REGIMM instructions (uses rt field bits 20-16, so 32 possible values)
pub static MIPS_REGIMM_LUT: [Instruction; 32] = [
    /* 0x00 */
    instruction!(
        BranchLessThanZero,
        IType,
        handlers::branch::<
            false,
            false,
            { handlers::BranchType::LessThanZero },
            { handlers::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x01 */
    instruction!(
        BranchGreaterEqualZero,
        IType,
        handlers::branch::<
            false,
            false,
            { handlers::BranchType::BranchGreaterEqualZero },
            { handlers::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x02 */ Instruction::invalid(),
    /* 0x03 */ Instruction::invalid(),
    /* 0x04 */ Instruction::invalid(),
    /* 0x05 */ Instruction::invalid(),
    /* 0x06 */ Instruction::invalid(),
    /* 0x07 */ Instruction::invalid(),
    /* 0x08 */ Instruction::invalid(),
    /* 0x09 */ Instruction::invalid(),
    /* 0x0A */ Instruction::invalid(),
    /* 0x0B */ Instruction::invalid(),
    /* 0x0C */ Instruction::invalid(),
    /* 0x0D */ Instruction::invalid(),
    /* 0x0E */ Instruction::invalid(),
    /* 0x0F */ Instruction::invalid(),
    /* 0x10 */
    instruction!(
        BranchLessThanZeroAndLink,
        IType,
        handlers::branch::<
            true,
            true,
            { handlers::BranchType::LessThanZero },
            { handlers::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x11 */
    instruction!(
        BranchGreaterEqualZeroAndLink,
        IType,
        handlers::branch::<
            true,
            true,
            { handlers::BranchType::BranchGreaterEqualZero },
            { handlers::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x12 */ Instruction::invalid(),
    /* 0x13 */ Instruction::invalid(),
    /* 0x14 */ Instruction::invalid(),
    /* 0x15 */ Instruction::invalid(),
    /* 0x16 */ Instruction::invalid(),
    /* 0x17 */ Instruction::invalid(),
    /* 0x18 */ Instruction::invalid(),
    /* 0x19 */ Instruction::invalid(),
    /* 0x1A */ Instruction::invalid(),
    /* 0x1B */ Instruction::invalid(),
    /* 0x1C */ Instruction::invalid(),
    /* 0x1D */ Instruction::invalid(),
    /* 0x1E */ Instruction::invalid(),
    /* 0x1F */ Instruction::invalid(),
];

// opcode = anything else
pub static MIPS_OTHER_LUT: [Instruction; 64] = [
    /* 0x00 */ Instruction::invalid(), // SPECIAL
    /* 0x01 */ Instruction::invalid(), // REGIMM
    /* 0x02 */
    instruction!(
        Jump,
        JType,
        handlers::branch::<
            false,
            false,
            { handlers::BranchType::Unconditional },
            { handlers::BranchAddressing::AbsoluteImmediate },
        >
    ),
    /* 0x03 */
    instruction!(
        JumpAndLink,
        JType,
        handlers::branch::<
            true,
            false,
            { handlers::BranchType::Unconditional },
            { handlers::BranchAddressing::AbsoluteImmediate },
        >
    ),
    /* 0x04 */
    instruction!(
        BranchEqual,
        IType,
        handlers::branch::<
            false,
            false,
            { handlers::BranchType::Equal },
            { handlers::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x05 */
    instruction!(
        BranchNotEqual,
        IType,
        handlers::branch::<
            false,
            false,
            { handlers::BranchType::NotEqual },
            { handlers::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x06 */
    instruction!(
        BranchLessEqualZero,
        IType,
        handlers::branch::<
            false,
            false,
            { handlers::BranchType::LessEqualZero },
            { handlers::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x07 */
    instruction!(
        BranchGreaterThanZero,
        IType,
        handlers::branch::<
            false,
            false,
            { handlers::BranchType::BranchGreaterThanZero },
            { handlers::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x08 */
    instruction!(
        AddImmediate,
        IType,
        handlers::alu::<{ handlers::AluOperation::Add }, false, true>
    ),
    /* 0x09 */
    instruction!(
        AddImmediateUnsigned,
        IType,
        handlers::alu::<{ handlers::AluOperation::Add }, true, true>
    ),
    /* 0x0A */
    instruction!(
        SetLessThanImmediate,
        IType,
        handlers::alu::<{ handlers::AluOperation::SetLessThan }, false, true>
    ),
    /* 0x0B */
    instruction!(
        SetLessThanImmediateUnsigned,
        IType,
        handlers::alu::<{ handlers::AluOperation::SetLessThan }, true, true>
    ),
    /* 0x0C */
    instruction!(
        AndImmediate,
        IType,
        handlers::alu::<{ handlers::AluOperation::And }, false, true>
    ),
    /* 0x0D */
    instruction!(
        OrImmediate,
        IType,
        handlers::alu::<{ handlers::AluOperation::Or }, false, true>
    ),
    /* 0x0E */
    instruction!(
        XorImmediate,
        IType,
        handlers::alu::<{ handlers::AluOperation::Xor }, false, true>
    ),
    /* 0x0F */
    instruction!(
        LoadUpperImmediate,
        IType,
        handlers::load_store::<
            true,
            { handlers::MemoryAccessType::Load },
            { handlers::MemoryTransferSize::Word },
            { handlers::MemoryAccessPortion::Full },
        >
    ),
    /* 0x10 */ Instruction::invalid(),
    /* 0x11 */ Instruction::invalid(),
    /* 0x12 */ Instruction::invalid(),
    /* 0x13 */ Instruction::invalid(),
    /* 0x14 */ Instruction::invalid(),
    /* 0x15 */ Instruction::invalid(),
    /* 0x16 */ Instruction::invalid(),
    /* 0x17 */ Instruction::invalid(),
    /* 0x18 */ Instruction::invalid(), // COP0
    /* 0x19 */ Instruction::invalid(), // No FPU for PSX
    /* 0x1A */ Instruction::invalid(),
    /* 0x1B */ Instruction::invalid(),
    /* 0x1C */ Instruction::invalid(),
    /* 0x1D */ Instruction::invalid(),
    /* 0x1E */ Instruction::invalid(),
    /* 0x1F */ Instruction::invalid(),
    /* 0x20 */
    instruction!(
        LoadByte,
        IType,
        handlers::load_store::<
            false,
            { handlers::MemoryAccessType::Load },
            { handlers::MemoryTransferSize::Byte },
            { handlers::MemoryAccessPortion::Full },
        >
    ),
    /* 0x21 */
    instruction!(
        LoadHalfword,
        IType,
        handlers::load_store::<
            false,
            { handlers::MemoryAccessType::Load },
            { handlers::MemoryTransferSize::HalfWord },
            { handlers::MemoryAccessPortion::Full },
        >
    ),
    /* 0x22 */
    instruction!(
        LoadWordLeft,
        IType,
        handlers::load_store::<
            false,
            { handlers::MemoryAccessType::Load },
            { handlers::MemoryTransferSize::Word },
            { handlers::MemoryAccessPortion::Left },
        >
    ),
    /* 0x23 */
    instruction!(
        LoadWord,
        IType,
        handlers::load_store::<
            false,
            { handlers::MemoryAccessType::Load },
            { handlers::MemoryTransferSize::Word },
            { handlers::MemoryAccessPortion::Full },
        >
    ),
    /* 0x24 */
    instruction!(
        LoadByteUnsigned,
        IType,
        handlers::load_store::<
            false,
            { handlers::MemoryAccessType::Load },
            { handlers::MemoryTransferSize::Byte },
            { handlers::MemoryAccessPortion::Full },
        >
    ),
    /* 0x25 */
    instruction!(
        LoadHalfwordUnsigned,
        IType,
        handlers::load_store::<
            false,
            { handlers::MemoryAccessType::Load },
            { handlers::MemoryTransferSize::HalfWord },
            { handlers::MemoryAccessPortion::Full },
        >
    ),
    /* 0x26 */
    instruction!(
        LoadWordRight,
        IType,
        handlers::load_store::<
            false,
            { handlers::MemoryAccessType::Load },
            { handlers::MemoryTransferSize::Word },
            { handlers::MemoryAccessPortion::Right },
        >
    ),
    /* 0x27 */ Instruction::invalid(),
    /* 0x28 */
    instruction!(
        StoreByte,
        IType,
        handlers::load_store::<
            false,
            { handlers::MemoryAccessType::Store },
            { handlers::MemoryTransferSize::Byte },
            { handlers::MemoryAccessPortion::Full },
        >
    ),
    /* 0x29 */
    instruction!(
        StoreHalfword,
        IType,
        handlers::load_store::<
            false,
            { handlers::MemoryAccessType::Store },
            { handlers::MemoryTransferSize::HalfWord },
            { handlers::MemoryAccessPortion::Full },
        >
    ),
    /* 0x2A */
    instruction!(
        StoreWordLeft,
        IType,
        handlers::load_store::<
            false,
            { handlers::MemoryAccessType::Store },
            { handlers::MemoryTransferSize::Word },
            { handlers::MemoryAccessPortion::Left },
        >
    ),
    /* 0x2B */
    instruction!(
        StoreWord,
        IType,
        handlers::load_store::<
            false,
            { handlers::MemoryAccessType::Store },
            { handlers::MemoryTransferSize::Word },
            { handlers::MemoryAccessPortion::Full },
        >
    ),
    /* 0x2C */ Instruction::invalid(),
    /* 0x2D */ Instruction::invalid(),
    /* 0x2E */
    instruction!(
        StoreWordRight,
        IType,
        handlers::load_store::<
            false,
            { handlers::MemoryAccessType::Store },
            { handlers::MemoryTransferSize::Word },
            { handlers::MemoryAccessPortion::Right },
        >
    ),
    /* 0x2F */ Instruction::invalid(),
    /* 0x30 */ Instruction::invalid(),
    /* 0x31 */ Instruction::invalid(),
    /* 0x32 */ Instruction::invalid(),
    /* 0x33 */ Instruction::invalid(),
    /* 0x34 */ Instruction::invalid(),
    /* 0x35 */ Instruction::invalid(),
    /* 0x36 */ Instruction::invalid(),
    /* 0x37 */ Instruction::invalid(),
    /* 0x38 */ Instruction::invalid(),
    /* 0x39 */ Instruction::invalid(),
    /* 0x3A */ Instruction::invalid(),
    /* 0x3B */ Instruction::invalid(),
    /* 0x3C */ Instruction::invalid(),
    /* 0x3D */ Instruction::invalid(),
    /* 0x3E */ Instruction::invalid(),
    /* 0x3F */ Instruction::invalid(),
];

pub static REGISTER_NAME_LUT: [&str; 32] = [
    "$zero", "$at", "$v0", "$v1", "$a0", "$a1", "$a2", "$a3", "$t0", "$t1", "$t2", "$t3", "$t4",
    "$t5", "$t6", "$t7", "$s0", "$s1", "$s2", "$s3", "$s4", "$s5", "$s6", "$s7", "$t8", "$t9",
    "$k0", "$k1", "$gp", "$sp", "$fp", "$ra",
];

pub static COP_REGISTER_NAME_LUT: [&str; 16] = [
    "???", "???", "???", "BPC", "???", "BDA", "TAR", "DCIC", "BadA", "BDAM", "???", "BPCM", "SR",
    "CAUSE", "EPC", "PRID",
];
