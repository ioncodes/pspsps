use super::sio0::SioDevice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemoryCardTransferState {
    Idle,
    Selected,
    CommandReceived(u8),
    SendingData(u8),
}

pub struct MemoryCardDevice {
    transfer_state: MemoryCardTransferState,
}

impl MemoryCardDevice {
    pub fn new() -> Self {
        Self {
            transfer_state: MemoryCardTransferState::Idle,
        }
    }
}

impl SioDevice for MemoryCardDevice {
    fn process_byte(&mut self, tx_byte: u8) -> u8 {
        match self.transfer_state {
            MemoryCardTransferState::Idle => {
                debug_assert!(tx_byte == 0x81, "Unexpected byte in Idle state");

                self.transfer_state = MemoryCardTransferState::Selected;
                tracing::debug!(target: "psx_core::mc", "Memory card selected");

                0xFF
            }
            MemoryCardTransferState::Selected => {
                // Receive command byte
                tracing::debug!(target: "psx_core::mc", cmd = format!("{:02X}", tx_byte), "Memory card command received");
                self.transfer_state = MemoryCardTransferState::CommandReceived(tx_byte);
                0xFF
            }
            MemoryCardTransferState::CommandReceived(cmd) => {
                // Process command
                tracing::error!(target: "psx_core::mc", cmd = format!("{:02X}", cmd), "Unknown memory card command");
                self.transfer_state = MemoryCardTransferState::SendingData(0);
                0xFF
            }
            MemoryCardTransferState::SendingData(index) => {
                // Send data bytes
                tracing::error!(target: "psx_core::mc", index, "Memory card data transfer not implemented");
                self.transfer_state = MemoryCardTransferState::Idle;
                0xFF
            }
        }
    }

    fn reset(&mut self) {
        self.transfer_state = MemoryCardTransferState::Idle;
    }

    fn is_selected(&self) -> bool {
        self.transfer_state != MemoryCardTransferState::Idle
    }

    fn deselect(&mut self) {
        if self.transfer_state != MemoryCardTransferState::Idle {
            tracing::debug!(target: "psx_core::mc", "Memory card deselected");
            self.transfer_state = MemoryCardTransferState::Idle;
        }
    }

    fn device_id(&self) -> u8 {
        0x81
    }
}
