use super::joy::{ControllerDevice, ControllerState};
use super::memcard::MemoryCardDevice;
use crate::mmu::bus::{Bus8, Bus16, Bus32};
use crate::sio::{SerialControl, SerialMode, SerialStatus};
use std::collections::VecDeque;

pub trait SioDevice {
    fn process_byte(&mut self, tx_byte: u8) -> u8;
    fn reset(&mut self);
    fn is_selected(&self) -> bool;
    fn deselect(&mut self);
    fn device_id(&self) -> u8;
}

crate::define_addr!(SIO0_TX_DATA_ADDR, 0x1F80_1040, 0, 4, 0x10);
crate::define_addr!(SIO0_RX_DATA_ADDR, 0x1F80_1040, 0, 4, 0x10);
crate::define_addr!(SIO0_STATUS_ADDR, 0x1F80_1044, 0, 4, 0x10);
crate::define_addr!(SIO0_MODE_ADDR, 0x1F80_1048, 0, 2, 0x10);
crate::define_addr!(SIO0_CTRL_ADDR, 0x1F80_104A, 0, 2, 0x10);
crate::define_addr!(SIO0_BAUD_ADDR, 0x1F80_104E, 0, 2, 0x10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransferPhase {
    Idle,
    ProcessingTx,    // Waiting to process TX byte (1088 cycles)
    WaitingForFlags, // Waiting to set flags/IRQ after RX (1088 cycles)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveDevice {
    None,
    Controller,
    MemoryCard,
}

pub struct Sio0 {
    pub control: SerialControl,
    pub status: SerialStatus,
    pub mode: SerialMode,
    pub baud: u16,
    rx_fifo: VecDeque<u8>,
    pending_tx: Option<u8>,
    pending_rx: Option<u8>, // Byte waiting to be made available
    cycles: usize,
    target_cycles: usize,
    transfer_phase: TransferPhase,
    // Devices
    controller: ControllerDevice,
    memory_card: MemoryCardDevice,
    active_device: ActiveDevice,
}

impl Default for Sio0 {
    fn default() -> Self {
        Sio0 {
            control: SerialControl::default(),
            status: SerialStatus::default(),
            mode: SerialMode::default(),
            baud: 0,
            rx_fifo: VecDeque::new(),
            pending_tx: None,
            pending_rx: None,
            cycles: 0,
            target_cycles: 1088,
            transfer_phase: TransferPhase::Idle,
            controller: ControllerDevice::new(),
            memory_card: MemoryCardDevice::new(),
            active_device: ActiveDevice::None,
        }
    }
}

impl Sio0 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_controller_state(&mut self, state: ControllerState) {
        self.controller.set_state(state);
    }

    pub fn tick(&mut self, cycles: usize) {
        self.cycles += cycles;

        match self.transfer_phase {
            TransferPhase::ProcessingTx if self.cycles >= self.target_cycles => {
                // First 1088 cycles complete: process TX byte
                self.cycles = 0;
                self.process_tx_byte();
                self.transfer_phase = TransferPhase::WaitingForFlags;
            }
            TransferPhase::WaitingForFlags if self.cycles >= self.target_cycles => {
                // Second 1088 cycles complete: set flags and IRQ
                self.cycles = 0;
                self.set_rx_flags();
                self.transfer_phase = TransferPhase::Idle;
            }
            _ => {}
        }

        // Check if devices were deselected (DTR went low)
        if !self.control.dtr_output_level() {
            self.controller.deselect();
            self.memory_card.deselect();
            self.active_device = ActiveDevice::None;
        }
    }

    pub fn should_trigger_irq(&self) -> bool {
        self.status.interrupt_request()
    }

    fn route_byte_to_device(&mut self, tx_byte: u8) -> u8 {
        // If no device is active and DTR is asserted, check for device selection
        if self.active_device == ActiveDevice::None && self.control.dtr_output_level() {
            match tx_byte {
                0x01 => {
                    self.active_device = ActiveDevice::Controller;
                }
                0x81 => {
                    self.active_device = ActiveDevice::MemoryCard;
                }
                _ => {}
            }
        }

        // Route to active device
        match self.active_device {
            ActiveDevice::Controller => self.controller.process_byte(tx_byte),
            ActiveDevice::MemoryCard => self.memory_card.process_byte(tx_byte),
            ActiveDevice::None => {
                tracing::trace!(target: "psx_core::sio", tx = format!("{:02X}", tx_byte), "No device selected");
                0xFF
            }
        }
    }

    fn process_tx_byte(&mut self) {
        if let Some(tx_byte) = self.pending_tx.take() {
            // Route the byte to the appropriate device
            let rx_byte = self.route_byte_to_device(tx_byte);

            // Store response for later (after second delay)
            self.pending_rx = Some(rx_byte);

            tracing::trace!(
                target: "psx_core::sio",
                tx = format!("{:02X}", tx_byte),
                rx = format!("{:02X}", rx_byte),
                "Byte processed"
            );
        }
    }

    fn set_rx_flags(&mut self) {
        if let Some(rx_byte) = self.pending_rx.take() {
            // Put response in RX FIFO
            self.rx_fifo.push_back(rx_byte);
            self.status.set_rx_fifo_not_empty(true);

            // TX is now ready again
            self.status.set_tx_ready_1(true);
            self.status.set_tx_ready_2(true);

            // Trigger interrupt if enabled
            if self.control.rx_interrupt_enable() {
                self.status.set_interrupt_request(true);
            }

            // Set ACK for DSR interrupt (controller acknowledges byte)
            self.status.set_ack_input_level(true);
            if self.control.dsr_interrupt_enable() {
                self.status.set_interrupt_request(true);
            }

            tracing::trace!(
                target: "psx_core::sio",
                rx = format!("{:02X}", rx_byte),
                "RX flags set"
            );
        }
    }
}

impl Bus8 for Sio0 {
    fn read_u8(&mut self, address: u32) -> u8 {
        match address {
            SIO0_RX_DATA_ADDR_START => {
                let byte = self.rx_fifo.pop_front().unwrap_or(0xFF);
                if self.rx_fifo.is_empty() {
                    self.status.set_rx_fifo_not_empty(false);
                }
                byte
            }
            _ => {
                tracing::error!(
                    target: "psx_core::sio",
                    address = format!("{:08X}", address),
                    "Unimplemented SIO0 8-bit read"
                );
                0xFF
            }
        }
    }

    fn write_u8(&mut self, _address: u32, _value: u8) {
        unreachable!();
    }
}

impl Bus16 for Sio0 {
    fn read_u16(&mut self, address: u32) -> u16 {
        match address {
            SIO0_TX_DATA_ADDR_START => self.read_u8(address) as u16,
            SIO0_STATUS_ADDR_START => self.status.0 as u16,
            SIO0_MODE_ADDR_START => self.mode.0,
            SIO0_CTRL_ADDR_START => self.control.0,
            SIO0_BAUD_ADDR_START => self.baud,
            _ => {
                tracing::error!(
                    target: "psx_core::sio",
                    address = format!("{:08X}", address),
                    "Unimplemented SIO0 16-bit read"
                );
                0xFF
            }
        }
    }

    fn write_u16(&mut self, address: u32, value: u16) {
        match address {
            SIO0_TX_DATA_ADDR_START => {
                self.pending_tx = Some(value as u8);
                self.status.set_tx_ready_1(false);
                self.status.set_tx_ready_2(false);
                self.cycles = 0; // Reset cycle counter for first 1088 cycle delay
                self.transfer_phase = TransferPhase::ProcessingTx;
            }
            SIO0_MODE_ADDR_START => {
                self.mode.0 = value;
                tracing::debug!(target: "psx_core::sio", mode = format!("{:04X}", value), "MODE");
            }
            SIO0_CTRL_ADDR_START => {
                // Handle acknowledge bit (write 1 to clear IRQ)
                if value & 0x10 != 0 {
                    self.status.set_interrupt_request(false);
                    self.status.set_ack_input_level(false);
                }

                // Handle reset bit
                if value & 0x40 != 0 {
                    self.rx_fifo.clear();
                    self.pending_tx = None;
                    self.pending_rx = None;
                    self.controller.reset();
                    self.memory_card.reset();
                    self.active_device = ActiveDevice::None;
                    self.transfer_phase = TransferPhase::Idle;
                    self.cycles = 0;
                    self.status.set_rx_fifo_not_empty(false);
                    self.status.set_tx_ready_1(true);
                    self.status.set_tx_ready_2(true);
                    tracing::debug!(target: "psx_core::sio", "Reset");
                }

                self.control.0 = value & !0x40; // Reset bit is write-only

                let dtr = self.control.dtr_output_level();
                tracing::trace!(target: "psx_core::sio", dtr, "CTRL");
            }
            SIO0_BAUD_ADDR_START => {
                self.baud = value;
                self.target_cycles = value as usize * 8; // Acordding to JunstinCase
                tracing::debug!(target: "psx_core::sio", baud = value, "BAUD");
            }
            _ => {
                tracing::error!(
                    target: "psx_core::sio",
                    address = format!("{:08X}", address),
                    value = format!("{:04X}", value),
                    "Unimplemented SIO0 16-bit write"
                )
            }
        }
    }
}

impl Bus32 for Sio0 {
    fn read_u32(&mut self, address: u32) -> u32 {
        match address {
            SIO0_RX_DATA_ADDR_START => self.read_u8(address) as u32,
            SIO0_STATUS_ADDR_START => self.status.0,
            _ => {
                tracing::error!(
                    target: "psx_core::sio",
                    address = format!("{:08X}", address),
                    "Unimplemented SIO0 32-bit read"
                );
                0xFF
            }
        }
    }

    fn write_u32(&mut self, _address: u32, _value: u32) {
        unreachable!();
    }
}
