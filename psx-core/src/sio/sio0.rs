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
enum ActiveDevice {
    None,
    Controller,
    MemoryCard,
}

pub struct Sio0 {
    pub control: SerialControl,
    pub status: SerialStatus,
    pub mode: SerialMode,
    baud: u16,
    rx_fifo: VecDeque<u8>,
    cycles: usize,
    target_cycles: usize,       // IRQ delay in cycles (baud * 8)
    irq_trigger_counter: usize, // Number of pending IRQs (0 = idle)

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
            cycles: 0,
            target_cycles: 0,       // Will be set to baud*8 when TX is written
            irq_trigger_counter: 0, // 0 = idle
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

        // Check if we're waiting for IRQ
        if self.cycles >= self.target_cycles && self.irq_trigger_counter > 0 {
            // IRQ delay complete: trigger IRQ
            self.cycles = 0;
            self.trigger_irq();

            // TX is ready again
            self.status.set_tx_ready_1(true);
            self.status.set_tx_ready_2(true);

            self.irq_trigger_counter -= 1;
        }

        // Check if devices were deselected (DTR went low)
        if !self.control.dtr_output_level() {
            self.reset_devices();
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

                    if self.control.port_number() == 2 {
                        // If Port 2 is selected, no controller is present
                        tracing::trace!(
                            target: "psx_core::sio",
                            tx = format!("{:02X}", tx_byte),
                            "Port 2 selected - no device"
                        );
                        return 0xFF;
                    }
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
            ActiveDevice::MemoryCard => {
                // Memory card is stubbed - always return 0xFF (no device present)
                tracing::trace!(target: "psx_core::sio", tx = format!("{:02X}", tx_byte), "Memory card stubbed");
                0xFF
            }
            ActiveDevice::None => {
                tracing::trace!(target: "psx_core::sio", tx = format!("{:02X}", tx_byte), "No device selected");
                0xFF
            }
        }
    }

    fn trigger_irq(&mut self) {
        // Only send IRQs for controller, NOT for memory card (stubbed)
        if self.active_device != ActiveDevice::MemoryCard {
            self.status.set_ack_input_level(true);

            // Trigger interrupt if enabled
            if self.control.rx_interrupt_enable()
                || self.control.tx_interrupt_enable()
                || self.control.dsr_interrupt_enable()
            {
                self.status.set_interrupt_request(true);
            }

            tracing::trace!(
                target: "psx_core::sio",
                "IRQ triggered"
            );
        }
    }

    fn reset_devices(&mut self) {
        self.controller.reset();
        self.memory_card.reset();
        self.active_device = ActiveDevice::None;
    }
}

impl Bus8 for Sio0 {
    fn read_u8(&mut self, address: u32) -> u8 {
        match address {
            SIO0_RX_DATA_ADDR_START => {
                let byte = self.rx_fifo.pop_front().unwrap_or_else(|| {
                    tracing::warn!(target: "psx_core::sio", "RX FIFO underflow on SIO0 read");
                    0xFF
                });

                if self.rx_fifo.is_empty() {
                    self.status.set_rx_fifo_not_empty(false);
                }

                tracing::trace!(target: "psx_core::sio", byte = format!("{:02X}", byte), "RX read");

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
            SIO0_RX_DATA_ADDR_START => self.read_u8(address) as u16,
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
                // TODO: check DTR here too and do not send if not asserted?

                let tx_byte = value as u8;

                // Immediately process TX and get RX response
                let rx_byte = self.route_byte_to_device(tx_byte);

                // Put response in RX FIFO immediately
                self.rx_fifo.push_back(rx_byte);
                self.status.set_rx_fifo_not_empty(true);

                // TX is busy until IRQ fires
                self.status.set_tx_ready_1(false);
                self.status.set_tx_ready_2(false);

                // Start IRQ countdown (baud * 8 cycles)
                self.cycles = 0;
                self.target_cycles = self.baud as usize * 8;
                self.irq_trigger_counter += 1;

                tracing::trace!(
                    target: "psx_core::sio",
                    tx = format!("{:02X}", tx_byte),
                    rx = format!("{:02X}", rx_byte),
                    "TX sent, RX available immediately"
                );
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
                    self.controller.reset();
                    self.memory_card.reset();
                    self.active_device = ActiveDevice::None;
                    self.cycles = 0;
                    self.irq_trigger_counter = 0; // Cancel any pending IRQs
                    self.status.set_rx_fifo_not_empty(false);
                    self.status.set_tx_ready_1(true);
                    self.status.set_tx_ready_2(true);
                    tracing::debug!(target: "psx_core::sio", "Reset");
                }

                self.control.0 = value & !0x40; // Reset bit is write-only

                // If port select changed to port 2, deselect any active device, but only if current device
                // is *NOT* memory card
                if matches!(self.active_device, ActiveDevice::None | ActiveDevice::Controller)
                    && self.control.port_number() == 2
                {
                    self.reset_devices();
                }

                tracing::trace!(target: "psx_core::sio", dtr = self.control.dtr_output_level(), port = self.control.port_number(), "CTRL");
            }
            SIO0_BAUD_ADDR_START => {
                self.baud = value;
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
