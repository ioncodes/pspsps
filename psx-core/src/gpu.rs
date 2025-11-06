pub mod cmd;
pub mod gp0;
pub mod gp1;

use crate::gpu::gp0::Gp0;
use crate::gpu::gp1::Gp1;
use crate::mmu::Addressable;

pub const GP0_ADDRESS_START: u32 = 0x1F80_1810;
pub const GP0_ADDRESS_END: u32 = 0x1F80_1813;
pub const GP1_ADDRESS_START: u32 = 0x1F80_1814;
pub const GP1_ADDRESS_END: u32 = 0x1F80_1817;

pub struct Gpu {
    pub gp0: Gp0,
    pub gp1: Gp1,
}

impl Gpu {
    pub fn new() -> Self {
        Self {
            gp0: Gp0::new(),
            gp1: Gp1::new(),
        }
    }
}

impl Addressable for Gpu {
    #[inline(always)]
    fn read_u8(&self, address: u32) -> u8 {
        match address {
            GP0_ADDRESS_START..=GP0_ADDRESS_END => self.gp0.read_u8(address),
            GP1_ADDRESS_START..=GP1_ADDRESS_END => self.gp1.read_u8(address),
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    fn write_u8(&mut self, address: u32, value: u8) {
        match address {
            GP0_ADDRESS_START..=GP0_ADDRESS_END => self.gp0.write_u8(address, value),
            GP1_ADDRESS_START..=GP1_ADDRESS_END => self.gp1.write_u8(address, value),
            _ => unreachable!(),
        }
    }
}
