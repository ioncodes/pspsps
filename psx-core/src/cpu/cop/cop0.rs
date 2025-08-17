use crate::cpu::cop::registers::StatusRegister;
use crate::cpu::cop::Cop;

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

pub struct Cop0 {
    pub bpc: u32,           // Breakpoint Program Counter
    pub bda: u32,           // Breakpoint Data Address
    pub tar: u32,           // Target Address
    pub dcic: u32,          // Debug and Cache Invalidate Control
    pub bad_a: u32,         // Bad Address
    pub bdam: u32,          // Breakpoint Data Address Mask
    pub bdcm: u32,          // Breakpoint Program Counter Mask
    pub sr: StatusRegister, // Status Register
    pub cause: u32,         // Cause Register
    pub epc: u32,           // Exception Program Counter
    pub prid: u32,          // Processor Revision ID
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
            cause: 0,
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
            13 => self.cause,
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
            13 => self.cause = value,
            14 => self.epc = value,
            15 => self.prid = value,
            _ => {
                tracing::warn!(target: "psx_core::cpu", "Attempted to write to unimplemented COP0 register: {}", register);
            }
        }
    }
}
