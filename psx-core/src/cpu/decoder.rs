use crate::cpu::lut::{self, MIPS_OTHER_LUT, MIPS_REGIMM_LUT, MIPS_RTYPE_LUT};
use crate::cpu::{Cpu, handlers};
type InstructionHandler = fn(&Instruction, &mut Cpu);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    // ALU
    Add,
    AddUnsigned,
    AddImmediate,
    AddImmediateUnsigned,
    Sub,
    SubUnsigned,
    Multiply,
    MultiplyUnsigned,
    Divide,
    DivideUnsigned,
    And,
    AndImmediate,
    Or,
    OrImmediate,
    Xor,
    XorImmediate,
    Nor,
    SetLessThan,
    SetLessThanImmediate,
    SetLessThanUnsigned,
    SetLessThanImmediateUnsigned,

    // Shifter
    ShiftLeftLogical,
    ShiftRightLogical,
    ShiftRightArithmetic,
    ShiftLeftLogicalVariable,
    ShiftRightLogicalVariable,
    ShiftRightArithmeticVariable,

    // Memory Access
    LoadByte,
    LoadByteUnsigned,
    LoadHalfword,
    LoadHalfwordUnsigned,
    LoadWord,
    LoadWordLeft,
    LoadWordRight,
    LoadUpperImmediate,
    StoreByte,
    StoreHalfword,
    StoreWord,
    StoreWordLeft,
    StoreWordRight,

    // Branch
    BranchEqual,
    BranchNotEqual,
    BranchGreaterThanZero,
    BranchLessEqualZero,
    BranchGreaterEqualZero,
    BranchLessThanZero,
    BranchLessThanZeroAndLink,
    BranchGreaterEqualZeroAndLink,
    Jump,
    JumpAndLink,
    JumpRegister,
    JumpAndLinkRegister,
    SystemCall,
    Break,
    MoveFromHi,
    MoveToHi,
    MoveFromLo,
    MoveToLo,

    // Coprocessor
    MoveControlFromCoprocessor(u8),
    MoveControlToCoprocessor(u8),
    MoveFromCoprocessor(u8),
    MoveToCoprocessor(u8),
    LoadWordToCoprocessor(u8),
    StoreWordFromCoprocessor(u8),
    ReturnFromException,

    // GTE
    GteRtps,
    GteNclip,
    GteOp,
    GteDpcs,
    GteIntpl,
    GteMvmva,
    GteNcds,
    GteCdp,
    GteNcdt,
    GteNccs,
    GteCc,
    GteNcs,
    GteNct,
    GteSqr,
    GteDcpl,
    GteDpct,
    GteAvsz3,
    GteAvsz4,
    GteRtpt,
    GteGpf,
    GteGpl,
    GteNcct,

    // Other
    Invalid,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InstructionType {
    RType,
    IType,
    JType,
    Cop,
    Invalid,
}

#[derive(Clone, Copy)]
pub struct Instruction {
    pub opcode: Opcode,
    pub raw: u32,
    pub opcode_type: InstructionType,
    pub handler: InstructionHandler,
    pub(crate) is_delay_slot: bool,
}

impl Instruction {
    pub const fn invalid() -> Self {
        static NOOP: InstructionHandler =
            |instr: &Instruction, _: &mut Cpu| todo!("Invalid instruction handler: {}", instr);

        Instruction {
            opcode: Opcode::Invalid,
            raw: 0,
            opcode_type: InstructionType::Invalid,
            handler: NOOP,
            is_delay_slot: false,
        }
    }

    pub const fn nop() -> Self {
        static NOP_HANDLER: InstructionHandler = |_: &Instruction, _: &mut Cpu| {
            // No operation
        };

        Instruction {
            opcode: Opcode::ShiftLeftLogical, // Using ShiftLeftLogical opcode to represent NOP
            raw: 0,
            opcode_type: InstructionType::RType,
            handler: NOP_HANDLER,
            is_delay_slot: false,
        }
    }

    #[inline(always)]
    pub fn is_branch(&self) -> bool {
        matches!(
            self.opcode,
            Opcode::BranchEqual
                | Opcode::BranchNotEqual
                | Opcode::BranchGreaterThanZero
                | Opcode::BranchLessEqualZero
                | Opcode::BranchGreaterEqualZero
                | Opcode::BranchLessThanZero
                | Opcode::BranchLessThanZeroAndLink
                | Opcode::BranchGreaterEqualZeroAndLink
        )
    }

    #[inline(always)]
    pub fn is_invalid(&self) -> bool {
        self.opcode == Opcode::Invalid
    }
}

impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        self.opcode == other.opcode
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Register(u8, bool);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operand {
    Register(Register),
    Immediate(u32),
    SignedImmediate(i16),
    Address(u32),
    Offset(i32),
    MemoryAddress { offset: i16, base: Register },
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let register_name = if self.1 {
            lut::COP_REGISTER_NAME_LUT.get(self.0 as usize).unwrap_or(&"???")
        } else {
            lut::REGISTER_NAME_LUT.get(self.0 as usize).unwrap_or(&"???")
        };
        write!(f, "{}", register_name)
    }
}

impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Register(reg) => write!(f, "{}", reg),
            Operand::Immediate(imm) => write!(f, "0x{:X}", imm),
            Operand::SignedImmediate(imm) => write!(f, "{}", imm),
            Operand::Address(addr) => write!(f, "0x{:08X}", addr),
            Operand::Offset(offset) if *offset >= 0 => write!(f, "+{}", offset),
            Operand::Offset(offset) => write!(f, "{}", offset),
            Operand::MemoryAddress { offset, base } => {
                write!(f, "{}({})", offset, base)
            }
        }
    }
}

impl Instruction {
    #[inline(always)]
    pub fn decode(opcode: u32) -> Self {
        let op = (opcode >> 26) & 0x3F; // Extract the opcode bits (bits 31-26)

        let instruction = match op {
            0x00 => {
                // R-Type instructions
                let func = opcode & 0x3F; // Extract the function code (bits 5-0)
                MIPS_RTYPE_LUT[func as usize]
            }
            0x01 => {
                // REGIMM instructions
                let rt = (opcode >> 16) & 0x1F; // Extract the rt field (bits 20-16)
                MIPS_REGIMM_LUT[rt as usize]
            }
            0x10..=0x13 => {
                // Coprocessor instructions (COP0, COP1, COP2, COP3)
                let cop_num = (op & 0x3) as u8; // Extract coprocessor number (bits 1-0 of opcode)
                let fmt = (opcode >> 21) & 0x1F; // Extract the format field (bits 25-21)

                if cop_num == 2 && fmt & 0b10000 != 0 {
                    Self::decode_gte(opcode)
                } else {
                    Self::decode_cop(opcode, cop_num, fmt)
                }
            }
            0x30..=0x33 | 0x38..=0x3B => {
                // LWCn / SWCn instructions for COP0, COP1, COP2, COP3
                let cop_num = (op & 0x3) as u8; // Extract coprocessor number (bits 1-0 of opcode)
                Instruction {
                    opcode: if matches!(op, 0x30..=0x33) {
                        Opcode::LoadWordToCoprocessor(cop_num)
                    } else {
                        Opcode::StoreWordFromCoprocessor(cop_num)
                    },
                    raw: opcode,
                    opcode_type: InstructionType::Cop,
                    handler: if matches!(op, 0x30..=0x33) {
                        handlers::cop::<{ handlers::CopOperation::LoadWordTo }>
                    } else {
                        handlers::cop::<{ handlers::CopOperation::StoreWordFrom }>
                    },
                    is_delay_slot: false,
                }
            }
            _ => {
                if op < 64 {
                    MIPS_OTHER_LUT[op as usize]
                } else {
                    Instruction::invalid()
                }
            }
        };

        Instruction {
            raw: opcode,
            ..instruction
        }
    }

    #[inline(always)]
    pub fn op(&self) -> u8 {
        ((self.raw >> 26) & 0x3F) as u8
    }

    #[inline(always)]
    pub fn rs(&self) -> u8 {
        ((self.raw >> 21) & 0x1F) as u8
    }

    #[inline(always)]
    pub fn base(&self) -> u8 {
        self.rs()
    }

    #[inline(always)]
    pub fn sa(&self) -> u8 {
        self.rs()
    }

    #[inline(always)]
    pub fn rt(&self) -> u8 {
        ((self.raw >> 16) & 0x1F) as u8
    }

    #[inline(always)]
    pub fn rd(&self) -> u8 {
        ((self.raw >> 11) & 0x1F) as u8
    }

    #[inline(always)]
    pub fn ft(&self) -> u8 {
        ((self.raw >> 16) & 0x1F) as u8
    }

    #[inline(always)]
    pub fn shamt(&self) -> u8 {
        ((self.raw >> 6) & 0x1F) as u8
    }

    #[inline(always)]
    pub fn funct(&self) -> u8 {
        (self.raw & 0x3F) as u8
    }

    #[inline(always)]
    pub fn immediate(&self) -> u16 {
        (self.raw & 0xFFFF) as u16
    }

    #[inline(always)]
    pub fn offset(&self) -> i16 {
        self.immediate() as i16
    }

    #[inline(always)]
    pub fn address(&self) -> u32 {
        self.raw & 0x03FFFFFF
    }

    #[inline(always)]
    pub fn jump_target(&self, pc: u32) -> u32 {
        let addr_field = self.address();
        let pc_plus_4 = pc + 4;
        (pc_plus_4 & 0xF0000000) | (addr_field << 2)
    }

    fn decode_cop(opcode: u32, cop_num: u8, fmt: u32) -> Self {
        match fmt {
            0b00000 => Instruction {
                opcode: Opcode::MoveFromCoprocessor(cop_num),
                raw: opcode,
                opcode_type: InstructionType::Cop,
                handler: handlers::cop::<{ handlers::CopOperation::MoveFrom }>,
                is_delay_slot: false,
            },
            0b00010 => Instruction {
                opcode: Opcode::MoveControlFromCoprocessor(cop_num),
                raw: opcode,
                opcode_type: InstructionType::Cop,
                handler: handlers::cop::<{ handlers::CopOperation::MoveControlFrom }>,
                is_delay_slot: false,
            },
            0b00100 => Instruction {
                opcode: Opcode::MoveToCoprocessor(cop_num),
                raw: opcode,
                opcode_type: InstructionType::Cop,
                handler: handlers::cop::<{ handlers::CopOperation::MoveTo }>,
                is_delay_slot: false,
            },
            0b00110 => Instruction {
                opcode: Opcode::MoveControlToCoprocessor(cop_num),
                raw: opcode,
                opcode_type: InstructionType::Cop,
                handler: handlers::cop::<{ handlers::CopOperation::MoveControlTo }>,
                is_delay_slot: false,
            },
            16 if cop_num == 0 => Instruction {
                opcode: Opcode::ReturnFromException,
                raw: opcode,
                opcode_type: InstructionType::Cop,
                handler: handlers::cop::<{ handlers::CopOperation::ReturnFromException }>,
                is_delay_slot: false,
            },
            _ => Instruction::invalid(),
        }
    }

    fn decode_gte(opcode: u32) -> Self {
        let cmd = (opcode & 0x3F) as u8;

        let gte_opcode = match cmd {
            0x01 => Opcode::GteRtps,
            0x06 => Opcode::GteNclip,
            0x0C => Opcode::GteOp,
            0x10 => Opcode::GteDpcs,
            0x11 => Opcode::GteIntpl,
            0x12 => Opcode::GteMvmva,
            0x13 => Opcode::GteNcds,
            0x14 => Opcode::GteCdp,
            0x16 => Opcode::GteNcdt,
            0x1B => Opcode::GteNccs,
            0x1C => Opcode::GteCc,
            0x1E => Opcode::GteNcs,
            0x20 => Opcode::GteNct,
            0x28 => Opcode::GteSqr,
            0x29 => Opcode::GteDcpl,
            0x2A => Opcode::GteDpct,
            0x2D => Opcode::GteAvsz3,
            0x2E => Opcode::GteAvsz4,
            0x30 => Opcode::GteRtpt,
            0x3D => Opcode::GteGpf,
            0x3E => Opcode::GteGpl,
            0x3F => Opcode::GteNcct,
            _ => return Instruction::invalid(),
        };

        Instruction {
            opcode: gte_opcode,
            raw: opcode,
            opcode_type: InstructionType::Cop,
            handler: handlers::gte_dispatch,
            is_delay_slot: false,
        }
    }

    // Helper methods to format instruction fields for display
    fn fmt_rd(&self) -> String {
        lut::REGISTER_NAME_LUT
            .get(self.rd() as usize)
            .unwrap_or(&"???")
            .to_string()
    }

    fn fmt_rs(&self) -> String {
        lut::REGISTER_NAME_LUT
            .get(self.rs() as usize)
            .unwrap_or(&"???")
            .to_string()
    }

    fn fmt_rt(&self) -> String {
        lut::REGISTER_NAME_LUT
            .get(self.rt() as usize)
            .unwrap_or(&"???")
            .to_string()
    }

    fn fmt_ft(&self) -> String {
        lut::COP_REGISTER_NAME_LUT
            .get(self.ft() as usize)
            .unwrap_or(&"???")
            .to_string()
    }

    fn fmt_shamt(&self) -> String {
        format!("0x{:X}", self.shamt())
    }

    fn fmt_offset(&self) -> String {
        format!("{}", self.offset())
    }

    fn fmt_base(&self) -> String {
        lut::REGISTER_NAME_LUT
            .get(self.base() as usize)
            .unwrap_or(&"???")
            .to_string()
    }

    fn fmt_imm(&self) -> String {
        format!("0x{:X}", self.immediate())
    }

    fn fmt_simm(&self) -> String {
        format!("{}", self.immediate() as i16)
    }

    fn fmt_target(&self) -> String {
        format!("0x{:08X}", self.address() << 2)
    }

    fn fmt_branch_offset(&self) -> String {
        let offset = ((self.offset() as i32) << 2) + 4;
        if offset >= 0 {
            format!("+{}", offset)
        } else {
            format!("{}", offset)
        }
    }

    // Returns the format string for the instruction
    fn format_string(&self) -> &'static str {
        match self.opcode {
            // ALU - R-type
            Opcode::Add => "@rd, @rs, @rt",
            Opcode::AddUnsigned => "@rd, @rs, @rt",
            Opcode::Sub => "@rd, @rs, @rt",
            Opcode::SubUnsigned => "@rd, @rs, @rt",
            Opcode::And => "@rd, @rs, @rt",
            Opcode::Or => "@rd, @rs, @rt",
            Opcode::Xor => "@rd, @rs, @rt",
            Opcode::Nor => "@rd, @rs, @rt",
            Opcode::SetLessThan => "@rd, @rs, @rt",
            Opcode::SetLessThanUnsigned => "@rd, @rs, @rt",

            // ALU - I-type
            Opcode::AddImmediate => "@rt, @rs, @simm",
            Opcode::AddImmediateUnsigned => "@rt, @rs, @simm",
            Opcode::AndImmediate => "@rt, @rs, @imm",
            Opcode::OrImmediate => "@rt, @rs, @imm",
            Opcode::XorImmediate => "@rt, @rs, @imm",
            Opcode::SetLessThanImmediate => "@rt, @rs, @simm",
            Opcode::SetLessThanImmediateUnsigned => "@rt, @rs, @imm",
            Opcode::LoadUpperImmediate => "@rt, @imm",

            // Multiply/Divide
            Opcode::Multiply => "@rs, @rt",
            Opcode::MultiplyUnsigned => "@rs, @rt",
            Opcode::Divide => "@rs, @rt",
            Opcode::DivideUnsigned => "@rs, @rt",

            // Shifter - immediate
            Opcode::ShiftLeftLogical => "@rd, @rt, @shamt",
            Opcode::ShiftRightLogical => "@rd, @rt, @shamt",
            Opcode::ShiftRightArithmetic => "@rd, @rt, @shamt",

            // Shifter - variable
            Opcode::ShiftLeftLogicalVariable => "@rd, @rs, @rt",
            Opcode::ShiftRightLogicalVariable => "@rd, @rs, @rt",
            Opcode::ShiftRightArithmeticVariable => "@rd, @rs, @rt",

            // Memory Access - Load
            Opcode::LoadByte => "@rt, @offset(@base)",
            Opcode::LoadByteUnsigned => "@rt, @offset(@base)",
            Opcode::LoadHalfword => "@rt, @offset(@base)",
            Opcode::LoadHalfwordUnsigned => "@rt, @offset(@base)",
            Opcode::LoadWord => "@rt, @offset(@base)",
            Opcode::LoadWordLeft => "@rt, @offset(@base)",
            Opcode::LoadWordRight => "@rt, @offset(@base)",

            // Memory Access - Store
            Opcode::StoreByte => "@rt, @offset(@base)",
            Opcode::StoreHalfword => "@rt, @offset(@base)",
            Opcode::StoreWord => "@rt, @offset(@base)",
            Opcode::StoreWordLeft => "@rt, @offset(@base)",
            Opcode::StoreWordRight => "@rt, @offset(@base)",

            // Branches
            Opcode::BranchEqual => "@rs, @rt, @branch_offset",
            Opcode::BranchNotEqual => "@rs, @rt, @branch_offset",
            Opcode::BranchGreaterThanZero => "@rs, @branch_offset",
            Opcode::BranchLessEqualZero => "@rs, @branch_offset",
            Opcode::BranchGreaterEqualZero => "@rs, @branch_offset",
            Opcode::BranchLessThanZero => "@rs, @branch_offset",
            Opcode::BranchLessThanZeroAndLink => "@rs, @branch_offset",
            Opcode::BranchGreaterEqualZeroAndLink => "@rs, @branch_offset",

            // Jumps
            Opcode::Jump => "@target",
            Opcode::JumpAndLink => "@target",
            Opcode::JumpRegister => "@rs",
            Opcode::JumpAndLinkRegister => "@rd, @rs",

            // HI/LO
            Opcode::MoveFromHi => "@rd",
            Opcode::MoveToHi => "@rs",
            Opcode::MoveFromLo => "@rd",
            Opcode::MoveToLo => "@rs",

            // System
            Opcode::SystemCall => "",
            Opcode::Break => "",

            // Coprocessor
            Opcode::MoveFromCoprocessor(_) => "@rt, @ft",
            Opcode::MoveToCoprocessor(_) => "@rt, @ft",
            Opcode::MoveControlFromCoprocessor(_) => "@rt, @ft",
            Opcode::MoveControlToCoprocessor(_) => "@rt, @ft",
            Opcode::LoadWordToCoprocessor(_) => "@ft, @offset(@base)",
            Opcode::StoreWordFromCoprocessor(_) => "@ft, @offset(@base)",
            Opcode::ReturnFromException => "",

            // Other
            _ => "???",
        }
    }
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "opcode: {}, op: {}, rs: {}, rt: {}, rd: {}, shamt: {}, funct: {}, immediate: {:04X}, address: {:08X}",
            self.opcode,
            self.op(),
            self.rs(),
            self.rt(),
            self.rd(),
            self.shamt(),
            self.funct(),
            self.immediate(),
            self.address()
        )
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let opcode_str = match self {
            // ALU
            Opcode::Add => "add",
            Opcode::AddUnsigned => "addu",
            Opcode::AddImmediate => "addi",
            Opcode::AddImmediateUnsigned => "addiu",
            Opcode::Sub => "sub",
            Opcode::SubUnsigned => "subu",
            Opcode::Multiply => "mult",
            Opcode::MultiplyUnsigned => "multu",
            Opcode::Divide => "div",
            Opcode::DivideUnsigned => "divu",
            Opcode::And => "and",
            Opcode::AndImmediate => "andi",
            Opcode::Or => "or",
            Opcode::OrImmediate => "ori",
            Opcode::Xor => "xor",
            Opcode::XorImmediate => "xori",
            Opcode::Nor => "nor",
            Opcode::SetLessThan => "slt",
            Opcode::SetLessThanImmediate => "slti",
            Opcode::SetLessThanUnsigned => "sltu",
            Opcode::SetLessThanImmediateUnsigned => "sltiu",

            // Shifter
            Opcode::ShiftLeftLogical => "sll",
            Opcode::ShiftRightLogical => "srl",
            Opcode::ShiftRightArithmetic => "sra",
            Opcode::ShiftLeftLogicalVariable => "sllv",
            Opcode::ShiftRightLogicalVariable => "srlv",
            Opcode::ShiftRightArithmeticVariable => "srav",

            // Memory Access
            Opcode::LoadByte => "lb",
            Opcode::LoadByteUnsigned => "lbu",
            Opcode::LoadHalfword => "lh",
            Opcode::LoadHalfwordUnsigned => "lhu",
            Opcode::LoadWord => "lw",
            Opcode::LoadWordLeft => "lwl",
            Opcode::LoadWordRight => "lwr",
            Opcode::LoadUpperImmediate => "lui",
            Opcode::StoreByte => "sb",
            Opcode::StoreHalfword => "sh",
            Opcode::StoreWord => "sw",
            Opcode::StoreWordLeft => "swl",
            Opcode::StoreWordRight => "swr",

            // Branch
            Opcode::BranchEqual => "beq",
            Opcode::BranchNotEqual => "bne",
            Opcode::BranchGreaterThanZero => "bgtz",
            Opcode::BranchLessEqualZero => "blez",
            Opcode::BranchGreaterEqualZero => "bgez",
            Opcode::BranchLessThanZero => "bltz",
            Opcode::BranchLessThanZeroAndLink => "bltzal",
            Opcode::BranchGreaterEqualZeroAndLink => "bgezal",
            Opcode::Jump => "j",
            Opcode::JumpAndLink => "jal",
            Opcode::JumpRegister => "jr",
            Opcode::JumpAndLinkRegister => "jalr",
            Opcode::SystemCall => "syscall",
            Opcode::Break => "break",
            Opcode::MoveFromHi => "mfhi",
            Opcode::MoveToHi => "mthi",
            Opcode::MoveFromLo => "mflo",
            Opcode::MoveToLo => "mtlo",

            // Coprocessor
            Opcode::MoveFromCoprocessor(cop) => return write!(f, "mfc{}", cop),
            Opcode::MoveToCoprocessor(cop) => return write!(f, "mtc{}", cop),
            Opcode::MoveControlFromCoprocessor(cop) => return write!(f, "cfc{}", cop),
            Opcode::MoveControlToCoprocessor(cop) => return write!(f, "ctc{}", cop),
            Opcode::LoadWordToCoprocessor(cop) => return write!(f, "lwc{}", cop),
            Opcode::StoreWordFromCoprocessor(cop) => return write!(f, "swc{}", cop),
            Opcode::ReturnFromException => "rfe",

            // GTE
            Opcode::GteRtps => "rtps",
            Opcode::GteNclip => "nclip",
            Opcode::GteOp => "op",
            Opcode::GteDpcs => "dpcs",
            Opcode::GteIntpl => "intpl",
            Opcode::GteMvmva => "mvmva",
            Opcode::GteNcds => "ncds",
            Opcode::GteCdp => "cdp",
            Opcode::GteNcdt => "ncdt",
            Opcode::GteNccs => "nccs",
            Opcode::GteCc => "cc",
            Opcode::GteNcs => "ncs",
            Opcode::GteNct => "nct",
            Opcode::GteSqr => "sqr",
            Opcode::GteDcpl => "dcpl",
            Opcode::GteDpct => "dpct",
            Opcode::GteAvsz3 => "avsz3",
            Opcode::GteAvsz4 => "avsz4",
            Opcode::GteRtpt => "rtpt",
            Opcode::GteGpf => "gpf",
            Opcode::GteGpl => "gpl",
            Opcode::GteNcct => "ncct",

            // Other
            Opcode::Invalid => "???",
        };
        write!(f, "{}", opcode_str)
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let opcode_str = format!("{}", self.opcode);
        let fmt_str = self.format_string();

        if fmt_str.is_empty() {
            // Instructions with no operands
            return write!(f, "{}", opcode_str);
        }

        // Perform replacements for all tagged entities
        let formatted = fmt_str
            .replace("@rd", &self.fmt_rd())
            .replace("@rs", &self.fmt_rs())
            .replace("@rt", &self.fmt_rt())
            .replace("@ft", &self.fmt_ft())
            .replace("@shamt", &self.fmt_shamt())
            .replace("@offset", &self.fmt_offset())
            .replace("@base", &self.fmt_base())
            .replace("@imm", &self.fmt_imm())
            .replace("@simm", &self.fmt_simm())
            .replace("@target", &self.fmt_target())
            .replace("@branch_offset", &self.fmt_branch_offset());

        write!(f, "{} {}", opcode_str, formatted)
    }
}
