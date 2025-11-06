use crate::cpu::lut::{MIPS_OTHER_LUT, MIPS_REGIMM_LUT, MIPS_RTYPE_LUT};
use crate::cpu::{Cpu, handlers};
use crate::mmu::Mmu;

type InstructionHandler = fn(&Instruction, &mut Cpu, &mut Mmu);

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
    MoveFromCoprocessor(u8),
    MoveToCoprocessor(u8),
    ReturnFromException,
    TlbRead,
    TlbWriteRandom,
    TlbWriteIndex,
    TlbProbe,

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
    pub opcode_raw: u32,
    pub opcode_type: InstructionType,
    pub handler: InstructionHandler,
}

impl Instruction {
    pub const fn invalid() -> Self {
        static NOOP: InstructionHandler =
            |_: &Instruction, _: &mut Cpu, _: &mut Mmu| todo!("Invalid instruction handler");

        Instruction {
            opcode: Opcode::Invalid,
            opcode_raw: 0,
            opcode_type: InstructionType::Invalid,
            handler: NOOP,
        }
    }
}

impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        self.opcode == other.opcode
            && self.opcode_raw == other.opcode_raw
            && self.opcode_type == other.opcode_type
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Register(u8);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operand {
    Register(Register),
    Immediate(u32),
    Address(u32),
    MemoryAddress { offset: i16, base: Register },
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self.0 {
            0 => "$zero",
            1 => "$at",
            2 => "$v0",
            3 => "$v1",
            4..=7 => return write!(f, "$a{}", self.0 - 4),
            8..=15 => return write!(f, "$t{}", self.0 - 8),
            16..=23 => return write!(f, "$s{}", self.0 - 16),
            24..=25 => return write!(f, "$t{}", self.0 - 16),
            26..=27 => return write!(f, "$k{}", self.0 - 26),
            28 => "$gp",
            29 => "$sp",
            30 => "$fp",
            31 => "$ra",
            _ => return write!(f, "$invalid"),
        };
        write!(f, "{}", name)
    }
}

impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Register(reg) => write!(f, "{}", reg),
            Operand::Immediate(imm) => write!(f, "0x{:X}", imm),
            Operand::Address(addr) => write!(f, "0x{:08X}", addr),
            Operand::MemoryAddress { offset, base } => {
                write!(f, "0x{:X}({})", offset, base)
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

                match fmt {
                    0x00 => Instruction {
                        opcode: Opcode::MoveFromCoprocessor(cop_num),
                        opcode_raw: opcode,
                        opcode_type: InstructionType::Cop,
                        handler: handlers::cop,
                    },
                    0x04 => Instruction {
                        opcode: Opcode::MoveToCoprocessor(cop_num),
                        opcode_raw: opcode,
                        opcode_type: InstructionType::Cop,
                        handler: handlers::cop,
                    },
                    0x10 if cop_num == 0 => Instruction {
                        opcode: Opcode::ReturnFromException,
                        opcode_raw: opcode,
                        opcode_type: InstructionType::Cop,
                        handler: handlers::cop,
                    },
                    _ => Instruction::invalid(),
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
            opcode_raw: opcode,
            ..instruction
        }
    }

    #[inline(always)]
    pub fn op(&self) -> u8 {
        ((self.opcode_raw >> 26) & 0x3F) as u8
    }

    #[inline(always)]
    pub fn rs(&self) -> u8 {
        ((self.opcode_raw >> 21) & 0x1F) as u8
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
        ((self.opcode_raw >> 16) & 0x1F) as u8
    }

    #[inline(always)]
    pub fn rd(&self) -> u8 {
        ((self.opcode_raw >> 11) & 0x1F) as u8
    }

    #[inline(always)]
    pub fn shamt(&self) -> u8 {
        ((self.opcode_raw >> 6) & 0x1F) as u8
    }

    #[inline(always)]
    pub fn funct(&self) -> u8 {
        (self.opcode_raw & 0x3F) as u8
    }

    #[inline(always)]
    pub fn immediate(&self) -> u16 {
        (self.opcode_raw & 0xFFFF) as u16
    }

    #[inline(always)]
    pub fn offset(&self) -> i16 {
        self.immediate() as i16
    }

    #[inline(always)]
    pub fn address(&self) -> u32 {
        self.opcode_raw & 0x03FFFFFF
    }

    #[inline(always)]
    pub fn jump_target(&self, pc: u32) -> u32 {
        let addr_field = self.address();
        let pc_plus_4 = pc + 4;
        (pc_plus_4 & 0xF0000000) | (addr_field << 2)
    }

    pub fn operand1(&self) -> Option<Operand> {
        match self.opcode_type {
            InstructionType::RType => match self.opcode {
                Opcode::ShiftLeftLogical
                | Opcode::ShiftRightLogical
                | Opcode::ShiftRightArithmetic => Some(Operand::Register(Register(self.rt()))),
                Opcode::JumpRegister | Opcode::JumpAndLinkRegister => {
                    Some(Operand::Register(Register(self.rs())))
                }
                Opcode::MoveFromHi | Opcode::MoveFromLo => {
                    Some(Operand::Register(Register(self.rd())))
                }
                Opcode::MoveToHi | Opcode::MoveToLo => Some(Operand::Register(Register(self.rs()))),
                Opcode::Multiply
                | Opcode::MultiplyUnsigned
                | Opcode::Divide
                | Opcode::DivideUnsigned => Some(Operand::Register(Register(self.rs()))),
                Opcode::SystemCall | Opcode::Break => None,
                _ => Some(Operand::Register(Register(self.rd()))),
            },
            InstructionType::IType => match self.opcode {
                Opcode::LoadUpperImmediate => Some(Operand::Register(Register(self.rt()))),
                Opcode::BranchGreaterThanZero
                | Opcode::BranchLessEqualZero
                | Opcode::BranchGreaterEqualZero
                | Opcode::BranchLessThanZero
                | Opcode::BranchLessThanZeroAndLink
                | Opcode::BranchGreaterEqualZeroAndLink => {
                    Some(Operand::Register(Register(self.rs())))
                }
                _ => Some(Operand::Register(Register(self.rt()))),
            },
            InstructionType::JType => Some(Operand::Address(self.address() << 2)),
            _ => None,
        }
    }

    pub fn operand2(&self) -> Option<Operand> {
        match self.opcode_type {
            InstructionType::RType => match self.opcode {
                Opcode::ShiftLeftLogical
                | Opcode::ShiftRightLogical
                | Opcode::ShiftRightArithmetic => Some(Operand::Immediate(self.shamt() as u32)),
                Opcode::ShiftLeftLogicalVariable
                | Opcode::ShiftRightLogicalVariable
                | Opcode::ShiftRightArithmeticVariable => {
                    Some(Operand::Register(Register(self.rs())))
                }
                Opcode::JumpRegister
                | Opcode::MoveFromHi
                | Opcode::MoveFromLo
                | Opcode::SystemCall
                | Opcode::Break => None,
                Opcode::JumpAndLinkRegister => Some(Operand::Register(Register(self.rd()))),
                Opcode::MoveToHi | Opcode::MoveToLo => None,
                Opcode::Multiply
                | Opcode::MultiplyUnsigned
                | Opcode::Divide
                | Opcode::DivideUnsigned => Some(Operand::Register(Register(self.rt()))),
                _ => Some(Operand::Register(Register(self.rs()))),
            },
            InstructionType::IType => match self.opcode {
                Opcode::LoadUpperImmediate => Some(Operand::Immediate(self.immediate() as u32)),
                Opcode::BranchGreaterThanZero
                | Opcode::BranchLessEqualZero
                | Opcode::BranchGreaterEqualZero
                | Opcode::BranchLessThanZero
                | Opcode::BranchLessThanZeroAndLink
                | Opcode::BranchGreaterEqualZeroAndLink => {
                    Some(Operand::Immediate((self.immediate() as i16) as u32))
                }
                Opcode::BranchEqual | Opcode::BranchNotEqual => {
                    Some(Operand::Register(Register(self.rt())))
                }
                Opcode::LoadByte
                | Opcode::LoadByteUnsigned
                | Opcode::LoadHalfword
                | Opcode::LoadHalfwordUnsigned
                | Opcode::LoadWord
                | Opcode::LoadWordLeft
                | Opcode::LoadWordRight => Some(Operand::MemoryAddress {
                    offset: self.immediate() as i16,
                    base: Register(self.rs()),
                }),
                Opcode::StoreByte
                | Opcode::StoreHalfword
                | Opcode::StoreWord
                | Opcode::StoreWordLeft
                | Opcode::StoreWordRight => Some(Operand::MemoryAddress {
                    offset: self.immediate() as i16,
                    base: Register(self.rs()),
                }),
                _ => Some(Operand::Register(Register(self.rs()))),
            },
            InstructionType::JType => None,
            _ => None,
        }
    }

    pub fn operand3(&self) -> Option<Operand> {
        match self.opcode_type {
            InstructionType::RType => match self.opcode {
                Opcode::ShiftLeftLogical
                | Opcode::ShiftRightLogical
                | Opcode::ShiftRightArithmetic
                | Opcode::ShiftLeftLogicalVariable
                | Opcode::ShiftRightLogicalVariable
                | Opcode::ShiftRightArithmeticVariable => {
                    Some(Operand::Register(Register(self.rd())))
                }
                Opcode::JumpRegister => None,
                Opcode::JumpAndLinkRegister => None,
                Opcode::MoveFromHi
                | Opcode::MoveFromLo
                | Opcode::MoveToHi
                | Opcode::MoveToLo
                | Opcode::SystemCall
                | Opcode::Break
                | Opcode::Multiply
                | Opcode::MultiplyUnsigned
                | Opcode::Divide
                | Opcode::DivideUnsigned => None,
                _ => Some(Operand::Register(Register(self.rt()))),
            },
            InstructionType::IType => match self.opcode {
                Opcode::LoadUpperImmediate
                | Opcode::BranchGreaterThanZero
                | Opcode::BranchLessEqualZero
                | Opcode::BranchGreaterEqualZero
                | Opcode::BranchLessThanZero
                | Opcode::BranchLessThanZeroAndLink
                | Opcode::BranchGreaterEqualZeroAndLink => None,
                Opcode::BranchEqual | Opcode::BranchNotEqual => {
                    Some(Operand::Immediate((self.immediate() as i16) as u32))
                }
                Opcode::LoadByte
                | Opcode::LoadByteUnsigned
                | Opcode::LoadHalfword
                | Opcode::LoadHalfwordUnsigned
                | Opcode::LoadWord
                | Opcode::LoadWordLeft
                | Opcode::LoadWordRight
                | Opcode::StoreByte
                | Opcode::StoreHalfword
                | Opcode::StoreWord
                | Opcode::StoreWordLeft
                | Opcode::StoreWordRight => None,
                _ => Some(Operand::Immediate((self.immediate() as i16) as u32)),
            },
            InstructionType::JType => None,
            _ => None,
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
            Opcode::ReturnFromException => "rfe",
            Opcode::TlbRead => "tlbr",
            Opcode::TlbWriteRandom => "tlbwr",
            Opcode::TlbWriteIndex => "tlbwi",
            Opcode::TlbProbe => "tlbp",

            // Other
            Opcode::Invalid => "???",
        };
        write!(f, "{}", opcode_str)
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = vec![format!("{}", self.opcode)];

        if let Some(op1) = self.operand1() {
            parts.push(format!("{}", op1));
        }

        if let Some(op2) = self.operand2() {
            parts.push(format!("{}", op2));
        }

        if let Some(op3) = self.operand3() {
            parts.push(format!("{}", op3));
        }

        if parts.len() == 1 {
            write!(f, "{}", parts[0])
        } else {
            write!(f, "{} {}", parts[0], parts[1..].join(", "))
        }
    }
}
