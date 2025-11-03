use crate::cpu::cop::Cop;
use crate::cpu::cop::registers::{CauseRegister, StatusRegister};

pub const COP0_BPC: u32 = 3; // Breakpoint Program Counter
pub const COP0_BDA: u32 = 5; // Breakpoint Data Address
pub const COP0_TAR: u32 = 6; // Target Address
pub const COP0_DCIC: u32 = 7; // Debug and Cache Invalidate Control
pub const COP0_BAD_A: u32 = 8; // Bad Address
pub const COP0_BDAM: u32 = 9; // Breakpoint Data Address Mask
pub const COP0_BDCM: u32 = 11; // Breakpoint Program Counter Mask
pub const COP0_SR: u32 = 12; // Status Register
pub const COP0_CAUSE: u32 = 13; // Cause Register
pub const COP0_EPC: u32 = 14; // Exception Program Counter
pub const COP0_PRID: u32 = 15; // Processor Revision ID

pub const COP0_EXCEPTION_CODE_INT: u32 = 0; // External Interrupt
pub const COP0_EXCEPTION_CODE_AD_EL: u32 = 4; // Address Error (Load)
pub const COP0_EXCEPTION_CODE_AD_ES: u32 = 5; // Address Error (Store)
pub const COP0_EXCEPTION_CODE_IBE: u32 = 6; // Instruction Bus Error
pub const COP0_EXCEPTION_CODE_DBE: u32 = 7; // Data Bus Error
pub const COP0_EXCEPTION_CODE_SYSCALL: u32 = 8; // Syscall
pub const COP0_EXCEPTION_CODE_BREAK: u32 = 9; // Breakpoint
pub const COP0_EXCEPTION_CODE_RI: u32 = 10; // Reserved Instruction
pub const COP0_EXCEPTION_CODE_OV: u32 = 12; // Arithmetic Overflow

#[repr(u32)]
pub enum Exception {
    External = COP0_EXCEPTION_CODE_INT,
    AddressErrorLoad = COP0_EXCEPTION_CODE_AD_EL,
    AddressErrorStore = COP0_EXCEPTION_CODE_AD_ES,
    InstructionBusError = COP0_EXCEPTION_CODE_IBE,
    DataBusError = COP0_EXCEPTION_CODE_DBE,
    Syscall = COP0_EXCEPTION_CODE_SYSCALL,
    Breakpoint = COP0_EXCEPTION_CODE_BREAK,
    ReservedInstruction = COP0_EXCEPTION_CODE_RI,
    ArithmeticOverflow = COP0_EXCEPTION_CODE_OV,
    Unknown = 0xFFFFFFFF,
}

impl From<u32> for Exception {
    fn from(value: u32) -> Self {
        match value {
            COP0_EXCEPTION_CODE_INT => Exception::External,
            COP0_EXCEPTION_CODE_AD_EL => Exception::AddressErrorLoad,
            COP0_EXCEPTION_CODE_AD_ES => Exception::AddressErrorStore,
            COP0_EXCEPTION_CODE_IBE => Exception::InstructionBusError,
            COP0_EXCEPTION_CODE_DBE => Exception::DataBusError,
            COP0_EXCEPTION_CODE_SYSCALL => Exception::Syscall,
            COP0_EXCEPTION_CODE_BREAK => Exception::Breakpoint,
            COP0_EXCEPTION_CODE_RI => Exception::ReservedInstruction,
            COP0_EXCEPTION_CODE_OV => Exception::ArithmeticOverflow,
            _ => {
                tracing::error!(target: "psx_core::cpu", "Unknown exception code: {}", value);
                Exception::Unknown
            },
        }
    }
}

impl std::fmt::Display for Exception {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let description = match self {
            Exception::External => "External Interrupt",
            Exception::AddressErrorLoad => "Address Error (Load)",
            Exception::AddressErrorStore => "Address Error (Store)",
            Exception::InstructionBusError => "Instruction Bus Error",
            Exception::DataBusError => "Data Bus Error",
            Exception::Syscall => "Syscall",
            Exception::Breakpoint => "Breakpoint",
            Exception::ReservedInstruction => "Reserved Instruction",
            Exception::ArithmeticOverflow => "Arithmetic Overflow",
            Exception::Unknown => "???",
        };
        write!(f, "{}", description)
    }
}

#[derive(Clone, Copy)]
pub struct Cop0 {
    pub bpc: u32,             // Breakpoint Program Counter
    pub bda: u32,             // Breakpoint Data Address
    pub tar: u32,             // Target Address
    pub dcic: u32,            // Debug and Cache Invalidate Control
    pub bad_a: u32,           // Bad Address
    pub bdam: u32,            // Breakpoint Data Address Mask
    pub bdcm: u32,            // Breakpoint Program Counter Mask
    pub sr: StatusRegister,   // Status Register
    pub cause: CauseRegister, // Cause Register
    pub epc: u32,             // Exception Program Counter
    pub prid: u32,            // Processor Revision ID
}

impl Cop0 {
    pub fn new() -> Self {
        Self {
            bpc: 0,
            bda: 0,
            tar: 0,
            dcic: 0,
            bad_a: 0,
            bdam: 0,
            bdcm: 0,
            sr: StatusRegister(0),
            cause: CauseRegister(0),
            epc: 0,
            prid: 0,
        }
    }
}

impl Cop for Cop0 {
    #[inline(always)]
    fn read_register(&self, register: u32) -> u32 {
        match register {
            3 => self.bpc,
            5 => self.bda,
            6 => self.tar,
            7 => self.dcic,
            8 => self.bad_a,
            9 => self.bdam,
            11 => self.bdcm,
            12 => self.sr.0,
            13 => self.cause.0,
            14 => self.epc,
            15 => self.prid,
            _ => {
                tracing::warn!(target: "psx_core::cpu", "Attempted to read unimplemented COP0 register: {}", register);
                0
            }
        }
    }

    #[inline(always)]
    fn write_register(&mut self, register: u32, value: u32) {
        match register {
            3 => self.bpc = value,
            5 => self.bda = value,
            6 => self.tar = value,
            7 => self.dcic = value,
            8 => self.bad_a = value,
            9 => self.bdam = value,
            11 => self.bdcm = value,
            12 => self.sr.0 = value,
            13 => self.cause.0 = value,
            14 => self.epc = value,
            15 => self.prid = value,
            _ => {
                tracing::warn!(target: "psx_core::cpu", "Attempted to write to unimplemented COP0 register: {}", register);
            }
        }
    }
}
