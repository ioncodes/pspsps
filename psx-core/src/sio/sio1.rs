use crate::{mmu::bus::{Bus8, Bus16, Bus32}, sio::SerialControl};

crate::define_addr!(SIO1_TX_DATA_ADDR, 0x1F80_1040, 1, 4, 0x10);
crate::define_addr!(SIO1_RX_DATA_ADDR, 0x1F80_1040, 1, 4, 0x10);
crate::define_addr!(SIO1_STATUS_ADDR, 0x1F80_1044, 1, 4, 0x10);
crate::define_addr!(SIO1_MODE_ADDR, 0x1F80_1048, 1, 2, 0x10);
crate::define_addr!(SIO1_CTRL_ADDR, 0x1F80_104A, 1, 2, 0x10);
crate::define_addr!(SIO1_BAUD_ADDR, 0x1F80_104E, 1, 2, 0x10);

#[derive(Default)]
pub struct Sio1 {
    pub control: SerialControl,
}

impl Sio1 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Bus8 for Sio1 {
    fn read_u8(&mut self, address: u32) -> u8 {
        tracing::error!(target: "psx_core::sio", address = format!("{:08X}", address), "SIO1 read not implemented");
        0xFF
    }

    fn write_u8(&mut self, _address: u32, _value: u8) {
        unreachable!();
    }
}

impl Bus16 for Sio1 {
    fn read_u16(&mut self, address: u32) -> u16 {
        tracing::error!(target: "psx_core::sio", address = format!("{:08X}", address), "SIO1 read not implemented");
        0xFFFF
    }

    fn write_u16(&mut self, address: u32, value: u16) {
        tracing::error!(target: "psx_core::sio", address = format!("{:08X}", address), value = format!("{:04X}", value), "SIO1 write not implemented");
    }
}

impl Bus32 for Sio1 {
    fn read_u32(&mut self, address: u32) -> u32 {
        tracing::error!(target: "psx_core::sio", address = format!("{:08X}", address), "SIO1 read not implemented");
        0xFFFF_FFFF
    }

    fn write_u32(&mut self, _address: u32, _value: u32) {
        unreachable!();
    }
}