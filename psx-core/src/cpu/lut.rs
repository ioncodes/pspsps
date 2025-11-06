use crate::cpu::decoder::Instruction;
use crate::cpu::interpreter;

macro_rules! instruction {
    ($name:ident, $opcode_type:ident, $handler:expr) => {
        Instruction {
            opcode: crate::cpu::decoder::Opcode::$name,
            raw: 0,
            opcode_type: crate::cpu::decoder::InstructionType::$opcode_type,
            handler: $handler,
            is_delay_slot: false,
        }
    };
    ($name:ident($param:expr), $opcode_type:ident, $handler:expr) => {
        Instruction {
            opcode: crate::cpu::decoder::Opcode::Opcode::$name($param),
            raw: 0,
            opcode_type: crate::cpu::decoder::InstructionType::$opcode_type,
            handler: $handler,
            is_delay_slot: false,
        }
    };
}

// opcode = 0x00
pub static MIPS_RTYPE_LUT: [Instruction; 64] = [
    /* 0x00 */
    instruction!(
        ShiftLeftLogical,
        RType,
        interpreter::cpu::shift::<{ interpreter::ShiftDirection::Left }, { interpreter::ShiftType::Logical }, false>
    ),
    /* 0x01 */ Instruction::invalid(),
    /* 0x02 */
    instruction!(
        ShiftRightLogical,
        RType,
        interpreter::cpu::shift::<{ interpreter::ShiftDirection::Right }, { interpreter::ShiftType::Logical }, false>
    ),
    /* 0x03 */
    instruction!(
        ShiftRightArithmetic,
        RType,
        interpreter::cpu::shift::<{ interpreter::ShiftDirection::Right }, { interpreter::ShiftType::Arithmetic }, false>
    ),
    /* 0x04 */
    instruction!(
        ShiftLeftLogicalVariable,
        RType,
        interpreter::cpu::shift::<{ interpreter::ShiftDirection::Left }, { interpreter::ShiftType::Logical }, true>
    ),
    /* 0x05 */ Instruction::invalid(),
    /* 0x06 */
    instruction!(
        ShiftRightLogicalVariable,
        RType,
        interpreter::cpu::shift::<{ interpreter::ShiftDirection::Right }, { interpreter::ShiftType::Logical }, true>
    ),
    /* 0x07 */
    instruction!(
        ShiftRightArithmeticVariable,
        RType,
        interpreter::cpu::shift::<{ interpreter::ShiftDirection::Right }, { interpreter::ShiftType::Arithmetic }, true>
    ),
    /* 0x08 */
    instruction!(
        JumpRegister,
        RType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::Unconditional },
            { interpreter::BranchAddressing::AbsoluteRegister },
        >
    ),
    /* 0x09 */
    instruction!(
        JumpAndLinkRegister,
        RType,
        interpreter::cpu::branch::<
            true,
            true,
            { interpreter::BranchType::Unconditional },
            { interpreter::BranchAddressing::AbsoluteRegister },
        >
    ),
    /* 0x0A */ Instruction::invalid(),
    /* 0x0B */ Instruction::invalid(),
    /* 0x0C */ instruction!(SystemCall, RType, interpreter::cpu::system_call),
    /* 0x0D */ instruction!(Break, RType, interpreter::cpu::debug_break),
    /* 0x0E */ Instruction::invalid(),
    /* 0x0F */ Instruction::invalid(),
    /* 0x10 */
    instruction!(
        MoveFromHi,
        RType,
        interpreter::cpu::move_multiply::<
            { interpreter::MultiplyMoveDirection::FromRegister },
            { interpreter::MultiplyMoveRegister::Hi },
        >
    ),
    /* 0x11 */
    instruction!(
        MoveToHi,
        RType,
        interpreter::cpu::move_multiply::<
            { interpreter::MultiplyMoveDirection::ToRegister },
            { interpreter::MultiplyMoveRegister::Hi },
        >
    ),
    /* 0x12 */
    instruction!(
        MoveFromLo,
        RType,
        interpreter::cpu::move_multiply::<
            { interpreter::MultiplyMoveDirection::FromRegister },
            { interpreter::MultiplyMoveRegister::Lo },
        >
    ),
    /* 0x13 */
    instruction!(
        MoveToLo,
        RType,
        interpreter::cpu::move_multiply::<
            { interpreter::MultiplyMoveDirection::ToRegister },
            { interpreter::MultiplyMoveRegister::Lo },
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
        interpreter::cpu::alu::<{ interpreter::AluOperation::Multiply }, false, false>
    ),
    /* 0x19 */
    instruction!(
        MultiplyUnsigned,
        RType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::Multiply }, true, false>
    ),
    /* 0x1A */
    instruction!(
        Divide,
        RType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::Divide }, false, false>
    ),
    /* 0x1B */
    instruction!(
        DivideUnsigned,
        RType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::Divide }, true, false>
    ),
    /* 0x1C */ Instruction::invalid(),
    /* 0x1D */ Instruction::invalid(),
    /* 0x1E */ Instruction::invalid(),
    /* 0x1F */ Instruction::invalid(),
    /* 0x20 */
    instruction!(
        Add,
        RType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::Add }, false, false>
    ),
    /* 0x21 */
    instruction!(
        AddUnsigned,
        RType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::Add }, true, false>
    ),
    /* 0x22 */
    instruction!(
        Sub,
        RType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::Sub }, false, false>
    ),
    /* 0x23 */
    instruction!(
        SubUnsigned,
        RType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::Sub }, true, false>
    ),
    /* 0x24 */
    instruction!(
        And,
        RType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::And }, false, false>
    ),
    /* 0x25 */
    instruction!(
        Or,
        RType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::Or }, false, false>
    ),
    /* 0x26 */
    instruction!(
        Xor,
        RType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::Xor }, false, false>
    ),
    /* 0x27 */
    instruction!(
        Nor,
        RType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::Nor }, false, false>
    ),
    /* 0x28 */ Instruction::invalid(),
    /* 0x29 */ Instruction::invalid(),
    /* 0x2A */
    instruction!(
        SetLessThan,
        RType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::SetLessThan }, false, false>
    ),
    /* 0x2B */
    instruction!(
        SetLessThanUnsigned,
        RType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::SetLessThan }, true, false>
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
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::LessThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x01 */
    instruction!(
        BranchGreaterEqualZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::GreaterEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x02 */
    instruction!(
        BranchLessThanZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::LessThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x03 */
    instruction!(
        BranchGreaterEqualZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::GreaterEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x04 */
    instruction!(
        BranchLessThanZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::LessThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x05 */
    instruction!(
        BranchGreaterEqualZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::GreaterEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x06 */
    instruction!(
        BranchLessThanZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::LessThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x07 */
    instruction!(
        BranchGreaterEqualZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::GreaterEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x08 */
    instruction!(
        BranchLessThanZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::LessThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x09 */
    instruction!(
        BranchGreaterEqualZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::GreaterEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x0A */
    instruction!(
        BranchLessThanZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::LessThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x0B */
    instruction!(
        BranchGreaterEqualZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::GreaterEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x0C */
    instruction!(
        BranchLessThanZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::LessThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x0D */
    instruction!(
        BranchGreaterEqualZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::GreaterEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x0E */
    instruction!(
        BranchLessThanZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::LessThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x0F */
    instruction!(
        BranchGreaterEqualZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::GreaterEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x10 */
    instruction!(
        BranchLessThanZeroAndLink,
        IType,
        interpreter::cpu::branch::<
            true,
            false,
            { interpreter::BranchType::LessThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x11 */
    instruction!(
        BranchGreaterEqualZeroAndLink,
        IType,
        interpreter::cpu::branch::<
            true,
            false,
            { interpreter::BranchType::GreaterEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x12 */
    instruction!(
        BranchLessThanZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::LessThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x13 */
    instruction!(
        BranchGreaterEqualZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::GreaterEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x14 */
    instruction!(
        BranchLessThanZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::LessThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x15 */
    instruction!(
        BranchGreaterEqualZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::GreaterEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x16 */
    instruction!(
        BranchLessThanZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::LessThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x17 */
    instruction!(
        BranchGreaterEqualZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::GreaterEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x18 */
    instruction!(
        BranchLessThanZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::LessThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x19 */
    instruction!(
        BranchGreaterEqualZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::GreaterEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x1A */
    instruction!(
        BranchLessThanZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::LessThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x1B */
    instruction!(
        BranchGreaterEqualZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::GreaterEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x1C */
    instruction!(
        BranchLessThanZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::LessThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x1D */
    instruction!(
        BranchGreaterEqualZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::GreaterEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x1E */
    instruction!(
        BranchLessThanZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::LessThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x1F */
    instruction!(
        BranchGreaterEqualZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::GreaterEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
];

// opcode = anything else
pub static MIPS_OTHER_LUT: [Instruction; 64] = [
    /* 0x00 */ Instruction::invalid(), // SPECIAL
    /* 0x01 */ Instruction::invalid(), // REGIMM
    /* 0x02 */
    instruction!(
        Jump,
        JType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::Unconditional },
            { interpreter::BranchAddressing::AbsoluteImmediate },
        >
    ),
    /* 0x03 */
    instruction!(
        JumpAndLink,
        JType,
        interpreter::cpu::branch::<
            true,
            false,
            { interpreter::BranchType::Unconditional },
            { interpreter::BranchAddressing::AbsoluteImmediate },
        >
    ),
    /* 0x04 */
    instruction!(
        BranchEqual,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::Equal },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x05 */
    instruction!(
        BranchNotEqual,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::NotEqual },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x06 */
    instruction!(
        BranchLessEqualZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::LessEqualZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x07 */
    instruction!(
        BranchGreaterThanZero,
        IType,
        interpreter::cpu::branch::<
            false,
            false,
            { interpreter::BranchType::GreaterThanZero },
            { interpreter::BranchAddressing::RelativeOffset },
        >
    ),
    /* 0x08 */
    instruction!(
        AddImmediate,
        IType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::Add }, false, true>
    ),
    /* 0x09 */
    instruction!(
        AddImmediateUnsigned,
        IType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::Add }, true, true>
    ),
    /* 0x0A */
    instruction!(
        SetLessThanImmediate,
        IType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::SetLessThan }, false, true>
    ),
    /* 0x0B */
    instruction!(
        SetLessThanImmediateUnsigned,
        IType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::SetLessThan }, true, true>
    ),
    /* 0x0C */
    instruction!(
        AndImmediate,
        IType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::And }, false, true>
    ),
    /* 0x0D */
    instruction!(
        OrImmediate,
        IType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::Or }, false, true>
    ),
    /* 0x0E */
    instruction!(
        XorImmediate,
        IType,
        interpreter::cpu::alu::<{ interpreter::AluOperation::Xor }, false, true>
    ),
    /* 0x0F */
    instruction!(
        LoadUpperImmediate,
        IType,
        interpreter::cpu::load_store::<
            true,
            { interpreter::MemoryAccessType::Load },
            { interpreter::MemoryTransferSize::Word },
            { interpreter::MemoryAccessPortion::Full },
            false,
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
        interpreter::cpu::load_store::<
            false,
            { interpreter::MemoryAccessType::Load },
            { interpreter::MemoryTransferSize::Byte },
            { interpreter::MemoryAccessPortion::Full },
            true,
        >
    ),
    /* 0x21 */
    instruction!(
        LoadHalfword,
        IType,
        interpreter::cpu::load_store::<
            false,
            { interpreter::MemoryAccessType::Load },
            { interpreter::MemoryTransferSize::HalfWord },
            { interpreter::MemoryAccessPortion::Full },
            true,
        >
    ),
    /* 0x22 */
    instruction!(
        LoadWordLeft,
        IType,
        interpreter::cpu::load_store::<
            false,
            { interpreter::MemoryAccessType::Load },
            { interpreter::MemoryTransferSize::Word },
            { interpreter::MemoryAccessPortion::Left },
            false,
        >
    ),
    /* 0x23 */
    instruction!(
        LoadWord,
        IType,
        interpreter::cpu::load_store::<
            false,
            { interpreter::MemoryAccessType::Load },
            { interpreter::MemoryTransferSize::Word },
            { interpreter::MemoryAccessPortion::Full },
            false,
        >
    ),
    /* 0x24 */
    instruction!(
        LoadByteUnsigned,
        IType,
        interpreter::cpu::load_store::<
            false,
            { interpreter::MemoryAccessType::Load },
            { interpreter::MemoryTransferSize::Byte },
            { interpreter::MemoryAccessPortion::Full },
            false,
        >
    ),
    /* 0x25 */
    instruction!(
        LoadHalfwordUnsigned,
        IType,
        interpreter::cpu::load_store::<
            false,
            { interpreter::MemoryAccessType::Load },
            { interpreter::MemoryTransferSize::HalfWord },
            { interpreter::MemoryAccessPortion::Full },
            false,
        >
    ),
    /* 0x26 */
    instruction!(
        LoadWordRight,
        IType,
        interpreter::cpu::load_store::<
            false,
            { interpreter::MemoryAccessType::Load },
            { interpreter::MemoryTransferSize::Word },
            { interpreter::MemoryAccessPortion::Right },
            false,
        >
    ),
    /* 0x27 */ Instruction::invalid(),
    /* 0x28 */
    instruction!(
        StoreByte,
        IType,
        interpreter::cpu::load_store::<
            false,
            { interpreter::MemoryAccessType::Store },
            { interpreter::MemoryTransferSize::Byte },
            { interpreter::MemoryAccessPortion::Full },
            false,
        >
    ),
    /* 0x29 */
    instruction!(
        StoreHalfword,
        IType,
        interpreter::cpu::load_store::<
            false,
            { interpreter::MemoryAccessType::Store },
            { interpreter::MemoryTransferSize::HalfWord },
            { interpreter::MemoryAccessPortion::Full },
            false,
        >
    ),
    /* 0x2A */
    instruction!(
        StoreWordLeft,
        IType,
        interpreter::cpu::load_store::<
            false,
            { interpreter::MemoryAccessType::Store },
            { interpreter::MemoryTransferSize::Word },
            { interpreter::MemoryAccessPortion::Left },
            false,
        >
    ),
    /* 0x2B */
    instruction!(
        StoreWord,
        IType,
        interpreter::cpu::load_store::<
            false,
            { interpreter::MemoryAccessType::Store },
            { interpreter::MemoryTransferSize::Word },
            { interpreter::MemoryAccessPortion::Full },
            false,
        >
    ),
    /* 0x2C */ Instruction::invalid(),
    /* 0x2D */ Instruction::invalid(),
    /* 0x2E */
    instruction!(
        StoreWordRight,
        IType,
        interpreter::cpu::load_store::<
            false,
            { interpreter::MemoryAccessType::Store },
            { interpreter::MemoryTransferSize::Word },
            { interpreter::MemoryAccessPortion::Right },
            false,
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
pub static GTE_LUT: [Instruction; 0x40] = [
    /* 0x00 */ Instruction::invalid(),
    /* 0x01 */ instruction!(GteRtps, Cop, interpreter::gte::unimplemented_gte),
    /* 0x02 */ Instruction::invalid(),
    /* 0x03 */ Instruction::invalid(),
    /* 0x04 */ Instruction::invalid(),
    /* 0x05 */ Instruction::invalid(),
    /* 0x06 */ instruction!(GteNclip, Cop, interpreter::gte::unimplemented_gte),
    /* 0x07 */ Instruction::invalid(),
    /* 0x08 */ Instruction::invalid(),
    /* 0x09 */ Instruction::invalid(),
    /* 0x0A */ Instruction::invalid(),
    /* 0x0B */ Instruction::invalid(),
    /* 0x0C */ instruction!(GteOp,  Cop, interpreter::gte::unimplemented_gte),
    /* 0x0D */ Instruction::invalid(),
    /* 0x0E */ Instruction::invalid(),
    /* 0x0F */ Instruction::invalid(),
    /* 0x10 */ instruction!(GteDpcs,  Cop, interpreter::gte::unimplemented_gte),
    /* 0x11 */ instruction!(GteIntpl, Cop, interpreter::gte::unimplemented_gte),
    /* 0x12 */ instruction!(GteMvmva, Cop, interpreter::gte::unimplemented_gte),
    /* 0x13 */ instruction!(GteNcds,  Cop, interpreter::gte::unimplemented_gte),
    /* 0x14 */ instruction!(GteCdp,  Cop, interpreter::gte::unimplemented_gte),
    /* 0x15 */ Instruction::invalid(),
    /* 0x16 */ instruction!(GteNcdt,  Cop, interpreter::gte::unimplemented_gte),
    /* 0x17 */ Instruction::invalid(),
    /* 0x18 */ Instruction::invalid(),
    /* 0x19 */ Instruction::invalid(),
    /* 0x1A */ Instruction::invalid(),
    /* 0x1B */ instruction!(GteNccs,  Cop, interpreter::gte::unimplemented_gte),
    /* 0x1C */ instruction!(GteCc,  Cop, interpreter::gte::unimplemented_gte),
    /* 0x1D */ Instruction::invalid(),
    /* 0x1E */ instruction!(GteNcs,  Cop, interpreter::gte::unimplemented_gte),
    /* 0x1F */ Instruction::invalid(),
    /* 0x20 */ instruction!(GteNct,  Cop, interpreter::gte::unimplemented_gte),
    /* 0x21 */ Instruction::invalid(),
    /* 0x22 */ Instruction::invalid(),
    /* 0x23 */ Instruction::invalid(),
    /* 0x24 */ Instruction::invalid(),
    /* 0x25 */ Instruction::invalid(),
    /* 0x26 */ Instruction::invalid(),
    /* 0x27 */ Instruction::invalid(),
    /* 0x28 */ instruction!(GteSqr,  Cop, interpreter::gte::unimplemented_gte),
    /* 0x29 */ instruction!(GteDcpl,  Cop, interpreter::gte::unimplemented_gte),
    /* 0x2A */ instruction!(GteDpct,  Cop, interpreter::gte::unimplemented_gte),
    /* 0x2B */ Instruction::invalid(),
    /* 0x2C */ Instruction::invalid(),
    /* 0x2D */ instruction!(GteAvsz3, Cop, interpreter::gte::unimplemented_gte),
    /* 0x2E */ instruction!(GteAvsz4, Cop, interpreter::gte::unimplemented_gte),
    /* 0x2F */ Instruction::invalid(),
    /* 0x30 */ instruction!(GteRtpt,  Cop, interpreter::gte::unimplemented_gte),
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
    /* 0x3D */ instruction!(GteGpf,  Cop, interpreter::gte::unimplemented_gte),
    /* 0x3E */ instruction!(GteGpl,  Cop, interpreter::gte::unimplemented_gte),
    /* 0x3F */ instruction!(GteNcct,  Cop, interpreter::gte::unimplemented_gte),
];

pub static REGISTER_NAME_LUT: [&str; 32] = [
    "$zero", "$at", "$v0", "$v1", "$a0", "$a1", "$a2", "$a3", "$t0", "$t1", "$t2", "$t3", "$t4", "$t5", "$t6", "$t7",
    "$s0", "$s1", "$s2", "$s3", "$s4", "$s5", "$s6", "$s7", "$t8", "$t9", "$k0", "$k1", "$gp", "$sp", "$fp", "$ra",
];

pub static COP_REGISTER_NAME_LUT: [&str; 16] = [
    "???", "???", "???", "BPC", "???", "BDA", "TAR", "DCIC", "BadA", "BDAM", "???", "BPCM", "SR", "CAUSE", "EPC",
    "PRID",
];

// GTE Data Registers (cop2r0-31)
pub static GTE_DATA_REGISTER_NAME_LUT: [&str; 32] = [
    "vxy0", "vz0", "vxy1", "vz1", "vxy2", "vz2", "rgbc", "otz", "ir0", "ir1", "ir2", "ir3", "sxy0", "sxy1", "sxy2",
    "sxyp", "sz0", "sz1", "sz2", "sz3", "rgb0", "rgb1", "rgb2", "res1", "mac0", "mac1", "mac2", "mac3", "irgb", "orgb",
    "lzcs", "lzcr",
];

// GTE Control Registers (cop2r32-63)
pub static GTE_CONTROL_REGISTER_NAME_LUT: [&str; 32] = [
    "r11r12", "r13r21", "r22r23", "r31r32", "r33", "trx", "try", "trz", "l11l12", "l13l21", "l22l23", "l31l32", "l33",
    "rbk", "gbk", "bbk", "lr1lr2", "lr3lg1", "lg2lg3", "lb1lb2", "lb3", "rfc", "gfc", "bfc", "ofx", "ofy", "h", "dqa",
    "dqb", "zsf3", "zsf4", "flag",
];
