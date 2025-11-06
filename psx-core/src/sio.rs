pub mod joy;
pub mod memcard;
pub mod sio0;
pub mod sio1;

use crate::mmu::bus::{Bus8, Bus16, Bus32};
use crate::sio::joy::ControllerState;
use crate::sio::sio1::Sio1;
use proc_bitfield::bitfield;
use sio0::{SIO0_BAUD_ADDR_END, SIO0_TX_DATA_ADDR_START, Sio0};
use sio1::{SIO1_BAUD_ADDR_END, SIO1_TX_DATA_ADDR_START};

pub const SIO_ADDR_START: u32 = SIO0_TX_DATA_ADDR_START;
pub const SIO_ADDR_END: u32 = SIO1_BAUD_ADDR_END;

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

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct SerialStatus(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub tx_ready_1: bool @ 0,
        pub rx_fifo_not_empty: bool @ 1,
        pub tx_ready_2: bool @ 2,
        pub rx_parity_error: bool @ 3,
        pub ack_input_level: bool @ 7,
        pub interrupt_request: bool @ 9,
        pub baud_timer: u32 @ 11..=31,
    }
}

impl Default for SerialStatus {
    fn default() -> Self {
        let mut status = SerialStatus(0);
        status.set_tx_ready_1(true);
        status.set_tx_ready_2(true);
        status
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Default)]
    pub struct SerialMode(pub u16): Debug, FromStorage, IntoStorage, DerefStorage {
        pub baud_reload_factor: u8 @ 0..=1,
        pub char_length: u8 @ 2..=3,
        pub parity_enable: bool @ 4,
        pub parity_odd: bool @ 5,
        pub clock_polarity: bool @ 8,
    }
}

pub struct Sio {
    sio0: Sio0,
    sio1: Sio1,
}

impl Sio {
    pub fn new() -> Self {
        Sio {
            sio0: Sio0::new(),
            sio1: Sio1::new(),
        }
    }

    pub fn set_controller_state(&mut self, state: ControllerState) {
        self.sio0.set_controller_state(state);
    }

    pub fn tick(&mut self, cycles: usize) {
        self.sio0.tick(cycles);
    }

    pub fn should_trigger_irq(&self) -> bool {
        self.sio0.should_trigger_irq()
    }
}

impl Bus8 for Sio {
    fn read_u8(&mut self, address: u32) -> u8 {
        match address {
            SIO0_TX_DATA_ADDR_START..=SIO0_BAUD_ADDR_END => self.sio0.read_u8(address),
            SIO1_TX_DATA_ADDR_START..=SIO1_BAUD_ADDR_END => self.sio1.read_u8(address),
            _ => unreachable!("Invalid SIO address: {:08X}", address),
        }
    }

    fn write_u8(&mut self, address: u32, value: u8) {
        self.write_u16(address, value as u16);
    }
}

impl Bus16 for Sio {
    fn read_u16(&mut self, address: u32) -> u16 {
        match address {
            SIO0_TX_DATA_ADDR_START..=SIO0_BAUD_ADDR_END => self.sio0.read_u16(address),
            SIO1_TX_DATA_ADDR_START..=SIO1_BAUD_ADDR_END => self.sio1.read_u16(address),
            _ => unreachable!("Invalid SIO address: {:08X}", address),
        }
    }

    fn write_u16(&mut self, address: u32, value: u16) {
        match address {
            SIO0_TX_DATA_ADDR_START..=SIO0_BAUD_ADDR_END => self.sio0.write_u16(address, value),
            SIO1_TX_DATA_ADDR_START..=SIO1_BAUD_ADDR_END => self.sio1.write_u16(address, value),
            _ => unreachable!("Invalid SIO address: {:08X}", address),
        }
    }
}

impl Bus32 for Sio {
    fn read_u32(&mut self, address: u32) -> u32 {
        match address {
            SIO0_TX_DATA_ADDR_START..=SIO0_BAUD_ADDR_END => self.sio0.read_u32(address),
            SIO1_TX_DATA_ADDR_START..=SIO1_BAUD_ADDR_END => self.sio1.read_u32(address),
            _ => unreachable!("Invalid SIO address: {:08X}", address),
        }
    }

    fn write_u32(&mut self, address: u32, value: u32) {
        self.write_u16(address, value as u16);
    }
}
