use crate::mmu::bus::{Bus8, Bus16, Bus32};
use proc_bitfield::bitfield;

pub const I_STAT_ADDR_START: u32 = 0x1F80_1070;
pub const I_STAT_ADDR_END: u32 = I_STAT_ADDR_START + 0x04 - 1;
pub const I_MASK_ADDR_START: u32 = 0x1F80_1074;
pub const I_MASK_ADDR_END: u32 = I_MASK_ADDR_START + 0x04 - 1;

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct IrqRegister(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub vblank: bool @ 0,
        pub gpu: bool @ 1,
        pub cdrom: bool @ 2,
        pub dma: bool @ 3,
        pub tmr0: bool @ 4,
        pub tmr1: bool @ 5,
        pub tmr2: bool @ 6,
        pub controller_and_memory_card: bool @ 7,
        pub sio: bool @ 8,
        pub lightpen: bool @ 9,
    }
}

pub struct Irq {
    pub status: IrqRegister,
    pub mask: IrqRegister,
}

impl Irq {
    pub fn new() -> Self {
        Irq {
            status: IrqRegister(0),
            mask: IrqRegister(0),
        }
    }
}

impl Bus8 for Irq {
    fn read_u8(&mut self, address: u32) -> u8 {
        match address {
            I_STAT_ADDR_START..=I_STAT_ADDR_END => {
                let offset = address & 0b11;
                (self.status.0 >> (offset * 8)) as u8
            }
            I_MASK_ADDR_START..=I_MASK_ADDR_END => {
                let offset = address & 0b11;
                (self.mask.0 >> (offset * 8)) as u8
            }
            _ => unreachable!(),
        }
    }

    #[tracing::instrument(
        target = "psx_core::irq",
        level = "warn",
        skip(self),
        fields(address = %format!("{:08X}", address), value = %format!("{:04X}", value))
    )]
    #[inline(always)]
    fn write_u8(&mut self, address: u32, value: u8) {
        // https://psx-spx.consoledev.net/unpredictablethings/
        self.write_u32(address, value as u32);
    }
}

impl Bus16 for Irq {
    fn read_u16(&mut self, address: u32) -> u16 {
        match address {
            I_STAT_ADDR_START..=I_STAT_ADDR_END => {
                let lo = self.read_u8(address) as u16;
                let hi = self.read_u8(address + 1) as u16;
                lo | (hi << 8)
            }
            I_MASK_ADDR_START..=I_MASK_ADDR_END => {
                let lo = self.read_u8(address) as u16;
                let hi = self.read_u8(address + 1) as u16;
                lo | (hi << 8)
            }
            _ => unreachable!(),
        }
    }

    #[tracing::instrument(
        target = "psx_core::irq",
        level = "warn",
        skip(self),
        fields(address = %format!("{:08X}", address), value = %format!("{:04X}", value))
    )]
    #[inline(always)]
    fn write_u16(&mut self, address: u32, value: u16) {
        // https://psx-spx.consoledev.net/unpredictablethings/
        self.write_u32(address, value as u32);
    }
}

impl Bus32 for Irq {
    fn read_u32(&mut self, address: u32) -> u32 {
        match address {
            I_STAT_ADDR_START => self.status.0,
            I_MASK_ADDR_START => self.mask.0,
            _ => unreachable!(),
        }
    }

    fn write_u32(&mut self, address: u32, value: u32) {
        match address {
            I_STAT_ADDR_START => self.status.0 = value,
            I_MASK_ADDR_START => self.mask.0 = value,
            _ => unreachable!(),
        }
    }
}
