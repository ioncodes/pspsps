use super::sio0::SioDevice;

#[derive(Clone, Copy, Default)]
pub struct ControllerState {
    // D-Pad
    pub d_up: bool,
    pub d_down: bool,
    pub d_left: bool,
    pub d_right: bool,

    // Action buttons
    pub cross: bool,
    pub circle: bool,
    pub square: bool,
    pub triangle: bool,

    // Shoulder buttons
    pub l1: bool,
    pub l2: bool,
    pub r1: bool,
    pub r2: bool,

    // System buttons
    pub start: bool,
    pub select: bool,
}

impl ControllerState {
    pub fn to_button_bytes(&self) -> (u8, u8) {
        let byte1 =
            (!self.select as u8) << 0 |
            (1 << 1) |
            (1 << 2) |
            (!self.start as u8) << 3 |
            (!self.d_up as u8) << 4 |
            (!self.d_right as u8) << 5 |
            (!self.d_down as u8) << 6 |
            (!self.d_left as u8) << 7;

        let byte2 =
            (!self.l2 as u8) << 0 |
            (!self.r2 as u8) << 1 |
            (!self.l1 as u8) << 2 |
            (!self.r1 as u8) << 3 |
            (!self.triangle as u8) << 4 |
            (!self.circle as u8) << 5 |
            (!self.cross as u8) << 6 |
            (!self.square as u8) << 7;

        (byte1, byte2)
    }
}

impl std::fmt::Debug for ControllerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[DPad: {}{}{}{} | Buttons: {}{}{}{} | Shoulders: {}{}{}{} | System: {}{}]",
            if self.d_up { "U" } else { "" },
            if self.d_down { "D" } else { "" },
            if self.d_left { "L" } else { "" },
            if self.d_right { "R" } else { "" },
            if self.cross { "X" } else { "" },
            if self.circle { "O" } else { "" },
            if self.square { "[]" } else { "" },
            if self.triangle { "Δ" } else { "" },
            if self.l1 { "L1" } else { "" },
            if self.l2 { "L2" } else { "" },
            if self.r1 { "R1" } else { "" },
            if self.r2 { "R2" } else { "" },
            if self.start { "Start" } else { "" },
            if self.select { "Select" } else { "" },
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControllerTransferState {
    Idle,
    Selected,
    CommandReceived,
    SendingData(u8),
}

pub struct ControllerDevice {
    state: ControllerState,
    transfer_state: ControllerTransferState,
}

impl ControllerDevice {
    pub fn new() -> Self {
        Self {
            state: ControllerState::default(),
            transfer_state: ControllerTransferState::Idle,
        }
    }

    pub fn set_state(&mut self, state: ControllerState) {
        self.state = state;
    }
}

impl SioDevice for ControllerDevice {
    fn process_byte(&mut self, tx_byte: u8) -> u8 {
        tracing::trace!(target: "psx_core::joy", tx = format!("{:02X}", tx_byte), "Processing byte");

        match self.transfer_state {
            ControllerTransferState::Idle => {
                debug_assert!(tx_byte == 0x01, "Unexpected byte in Idle state");

                self.transfer_state = ControllerTransferState::Selected;
                tracing::debug!(target: "psx_core::joy", "Controller selected");

                0xFF
            }
            ControllerTransferState::Selected => {
                // 42h  idlo  Receive ID bit0..7 (variable) and Send Read Command (ASCII "B")
                if tx_byte == 0x42 {
                    self.transfer_state = ControllerTransferState::CommandReceived;
                    tracing::trace!(target: "psx_core::joy", "Read controller command");
                    0x41 // Digital pad ID low byte
                } else {
                    tracing::error!(target: "psx_core::joy", tx = format!("{:02X}", tx_byte), "Unknown command while controller selected");
                    0xFF
                }
            }
            ControllerTransferState::CommandReceived => {
                tracing::trace!(target: "psx_core::joy", "Sending controller ID high byte");
                // TAP  idhi  Receive ID bit8..15 (usually/always 5Ah)
                self.transfer_state = ControllerTransferState::SendingData(0);
                0x5A
            }
            ControllerTransferState::SendingData(index) => {                
                let (byte1, byte2) = self.state.to_button_bytes();
                tracing::trace!(target: "psx_core::joy", state = ?self.state, "Sending controller data");

                match index {
                    0 => {
                        self.transfer_state = ControllerTransferState::SendingData(1);
                        tracing::trace!(target: "psx_core::joy", btn = format!("{:02X}", byte1), "Sending controller button data 1");
                        byte1 // First button byte
                    }
                    1 => {
                        // Last byte - transfer complete
                        self.transfer_state = ControllerTransferState::Idle;
                        tracing::trace!(target: "psx_core::joy", btn = format!("{:02X}", byte2), "Sending controller button data 2");
                        byte2 // Second button byte
                    }
                    _ => {
                        self.transfer_state = ControllerTransferState::Idle;
                        tracing::error!(target: "psx_core::joy", index, "Controller sending data index out of range");
                        0xFF
                    }
                }
            }
        }
    }

    fn reset(&mut self) {
        self.transfer_state = ControllerTransferState::Idle;
    }

    fn is_selected(&self) -> bool {
        self.transfer_state != ControllerTransferState::Idle
    }

    fn deselect(&mut self) {
        if self.transfer_state != ControllerTransferState::Idle {
            tracing::debug!(target: "psx_core::joy", "Controller deselected");
            self.transfer_state = ControllerTransferState::Idle;
        }
    }

    fn device_id(&self) -> u8 {
        0x01
    }
}
