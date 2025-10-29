use super::joy::ControllerState;
use crate::mmu::bus::{Bus8, Bus16, Bus32};
use crate::sio::{SerialControl, SerialMode, SerialStatus};
use std::collections::VecDeque;

crate::define_addr!(SIO0_TX_DATA_ADDR, 0x1F80_1040, 0, 4, 0x10);
crate::define_addr!(SIO0_RX_DATA_ADDR, 0x1F80_1040, 0, 4, 0x10);
crate::define_addr!(SIO0_STATUS_ADDR, 0x1F80_1044, 0, 4, 0x10);
crate::define_addr!(SIO0_MODE_ADDR, 0x1F80_1048, 0, 2, 0x10);
crate::define_addr!(SIO0_CTRL_ADDR, 0x1F80_104A, 0, 2, 0x10);
crate::define_addr!(SIO0_BAUD_ADDR, 0x1F80_104E, 0, 2, 0x10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControllerTransferState {
    Idle,
    Selected,
    CommandReceived,
    SendingData(u8), // byte index
}

pub struct Sio0 {
    pub control: SerialControl,
    pub status: SerialStatus,
    pub mode: SerialMode,
    pub baud: u16,
    rx_fifo: VecDeque<u8>,
    pending_tx: Option<u8>,
    cycles: usize,
    target_cycles: usize,
    transfer_state: ControllerTransferState,
    controller_state: ControllerState,
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
            cycles: 0,
            target_cycles: 500, // TODO: delay per byte?
            transfer_state: ControllerTransferState::Idle,
            controller_state: ControllerState::default(),
        }
    }
}

impl Sio0 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_controller_state(&mut self, state: ControllerState) {
        self.controller_state = state;
    }

    pub fn tick(&mut self, cycles: usize) {
        self.cycles += cycles;

        if self.cycles >= self.target_cycles && self.pending_tx.is_some() {
            self.cycles = 0;
            self.handle_tx_byte();
        }

        // Check if controller was deselected (DTR went low)
        if !self.control.dtr_output_level() {
            if self.transfer_state != ControllerTransferState::Idle {
                tracing::debug!(target: "psx_core::sio", "Controller deselected");
                self.transfer_state = ControllerTransferState::Idle;
            }
        }
    }

    pub fn should_trigger_irq(&self) -> bool {
        self.status.interrupt_request()
    }

    fn process_controller_byte(&mut self, tx_byte: u8) -> u8 {
        match self.transfer_state {
            ControllerTransferState::Idle => {
                // First byte should be 0x01 to select controller
                // DTR bit = 1 means controller port is selected
                if tx_byte == 0x01 && self.control.dtr_output_level() {
                    self.transfer_state = ControllerTransferState::Selected;
                    tracing::debug!(target: "psx_core::sio", "Controller selected");
                    0xFF
                } else {
                    0xFF
                }
            }
            ControllerTransferState::Selected => {
                // Second byte is the command (0x42 = Read Data)
                if tx_byte == 0x42 {
                    self.transfer_state = ControllerTransferState::CommandReceived;
                    tracing::trace!(target: "psx_core::sio", "Read controller command");
                    0x69
                } else {
                    tracing::warn!(target: "psx_core::sio", command = format!("{:02X}", tx_byte), "Unknown controller command");
                    self.transfer_state = ControllerTransferState::Idle;
                    0xFF
                }
            }
            ControllerTransferState::CommandReceived => {
                // Third byte: ignore (TAP), return 0x5A (always)
                self.transfer_state = ControllerTransferState::SendingData(0);
                0x5A
            }
            ControllerTransferState::SendingData(index) => {
                let (byte1, byte2) = self.controller_state.to_button_bytes();
                let response = match index {
                    0 => {
                        self.transfer_state = ControllerTransferState::SendingData(1);
                        byte1 // First button byte
                    }
                    1 => {
                        // Last byte - transfer complete
                        self.transfer_state = ControllerTransferState::Idle;
                        byte2 // Second button byte
                    }
                    _ => {
                        self.transfer_state = ControllerTransferState::Idle;
                        0xFF
                    }
                };

                response
            }
        }
    }

    fn handle_tx_byte(&mut self) {
        if let Some(tx_byte) = self.pending_tx.take() {
            // Process the byte through the controller
            let rx_byte = self.process_controller_byte(tx_byte);

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
                tx = format!("{:02X}", tx_byte),
                rx = format!("{:02X}", rx_byte),
                "Byte transfer"
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
                    self.transfer_state = ControllerTransferState::Idle;
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
                // Update target cycles based on baud rate
                let factor = match self.mode.baud_reload_factor() {
                    0 => 1,
                    1 => 16,
                    2 => 64,
                    _ => 1,
                };
                if value > 0 {
                    self.target_cycles = (value as usize * factor).max(100);
                }
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
