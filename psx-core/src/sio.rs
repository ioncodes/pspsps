use crate::mmu::bus::{Bus8, Bus16, Bus32};
use proc_bitfield::bitfield;

pub const SIO_ADDR_START: u32 = 0x1F80_1040;
pub const SIO_ADDR_END: u32 = 0x1F80_105F;

crate::define_addr!(SIO0_TX_DATA_ADDR, 0x1F80_1040, 0, 4, 0x10);
crate::define_addr!(SIO0_RX_DATA_ADDR, 0x1F80_1040, 0, 4, 0x10);
crate::define_addr!(SIO0_STATUS_ADDR, 0x1F80_1044, 0, 4, 0x10);
crate::define_addr!(SIO0_MODE_ADDR, 0x1F80_1048, 0, 2, 0x10);
crate::define_addr!(SIO0_CTRL_ADDR, 0x1F80_104A, 0, 2, 0x10);
crate::define_addr!(SIO0_BAUD_ADDR, 0x1F80_104E, 0, 2, 0x10);

crate::define_addr!(SIO1_TX_DATA_ADDR, 0x1F80_1040, 1, 4, 0x10);
crate::define_addr!(SIO1_RX_DATA_ADDR, 0x1F80_1040, 1, 4, 0x10);
crate::define_addr!(SIO1_STATUS_ADDR, 0x1F80_1044, 1, 4, 0x10);
crate::define_addr!(SIO1_MODE_ADDR, 0x1F80_1048, 1, 2, 0x10);
crate::define_addr!(SIO1_CTRL_ADDR, 0x1F80_104A, 1, 2, 0x10);
crate::define_addr!(SIO1_BAUD_ADDR, 0x1F80_104E, 1, 2, 0x10);

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Default)]
    pub struct SerialControl(pub u16): Debug, FromStorage, IntoStorage, DerefStorage {
        pub tx_enable: bool @ 0,
        pub dtr_output_level: bool @ 1,
        pub rx_enable: bool @ 2,
        pub sio1_tx_output_level: bool @ 3,
        pub acknowledge: bool @ 4,
        pub sio1_rts_output_level: bool @ 5,
        pub reset: bool @ 6,
        pub rx_interrupt_mode: u8 @ 8..=9,
        pub tx_interrupt_enable: bool @ 10,
        pub rx_interrupt_enable: bool @ 11,
        pub dsr_interrupt_enable: bool @ 12,
        pub sio0_port_select: bool @ 13
    }
}

#[derive(Default)]
pub struct SerialInterface {
    pub control: SerialControl,
}

pub struct Sio {
    sio0: SerialInterface,
    sio1: SerialInterface,
}

impl Sio {
    pub fn new() -> Self {
        Sio {
            sio0: SerialInterface::default(),
            sio1: SerialInterface::default()
        }
    }
}

impl Bus8 for Sio {
    fn read_u8(&mut self, address: u32) -> u8 {
        match address {
            _ => {
                tracing::error!(
                    target: "psx_core::sio",
                    address = format!("{:08X}", address),
                    "Unimplemented SIO read"
                );
                0xFF
            }
        }
    }

    fn write_u8(&mut self, address: u32, value: u8) {
        tracing::warn!(target: "psx_core::sio", "SIO 8-bit write detected");
        self.write_u16(address, value as u16);
    }
}

impl Bus16 for Sio {
    fn read_u16(&mut self, address: u32) -> u16 {
        match address {
            _ => {
                tracing::error!(
                    target: "psx_core::sio",
                    address = format!("{:08X}", address),
                    "Unimplemented SIO read"
                );
                0xFF
            }
        }
    }

    fn write_u16(&mut self, address: u32, value: u16) {
        match address {
            SIO0_CTRL_ADDR_START => self.sio0.control.0 = value,
            SIO1_CTRL_ADDR_START => self.sio1.control.0 = value,
            _ => {
                tracing::error!(
                    target: "psx_core::sio",
                    address = format!("{:08X}", address),
                    value,
                    "Unimplemented SIO write"
                )
            }
        }
    }
}

impl Bus32 for Sio {
    fn read_u32(&mut self, address: u32) -> u32 {
        match address {
            _ => {
                tracing::error!(
                    target: "psx_core::sio",
                    address = format!("{:08X}", address),
                    "Unimplemented SIO read"
                );
                0xFF
            }
        }
    }

    fn write_u32(&mut self, address: u32, value: u32) {
        tracing::warn!(target: "psx_core::sio", "SIO 32-bit write detected");
        self.write_u16(address, value as u16);
    }
}
