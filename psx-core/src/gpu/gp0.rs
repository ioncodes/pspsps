use crate::gpu::GP0_ADDRESS_START;
use crate::gpu::cmd::Gp0Command;
use crate::mmu::Addressable;
use std::collections::VecDeque;

pub struct Gp0 {
    fifo: VecDeque<u32>,
    current_command: [u8; 4],
    extra_data: usize, // Number of extra data words expected for the current command
}

impl Gp0 {
    pub fn new() -> Self {
        Self {
            fifo: VecDeque::with_capacity(16),
            current_command: [0; 4],
            extra_data: 0,
        }
    }
}

impl Addressable for Gp0 {
    fn read_u8(&self, address: u32) -> u8 {
        tracing::error!(target: "psx_core::gpu", address = %format!("{:08X}", address), "Reading from GP0 is not implemented");
        0xFF
    }

    fn write_u8(&mut self, address: u32, value: u8) {
        match address % GP0_ADDRESS_START {
            0 => self.current_command[0] = value,
            1 => self.current_command[1] = value,
            2 => self.current_command[2] = value,
            3 => {
                self.current_command[3] = value;

                // Extract command/data and push to FIFO
                let command = u32::from_le_bytes(self.current_command);
                self.fifo.push_back(command);

                // Reset current command buffer
                self.current_command = [0; 4];

                // If we are still expecting extra data for the previous command, do not parse a new command
                if self.extra_data > 0 {
                    self.extra_data -= 1;
                    tracing::debug!(target: "psx_core::gpu", raw = %format!("{:08X}", command), "GP0 command data received, waiting for {} more words", self.extra_data);
                    return;
                }

                // We are not expecting extra data, so parse the new command
                let parsed_command = Gp0Command::from(command);
                self.extra_data = parsed_command.extra_data();

                tracing::debug!(target: "psx_core::gpu", raw = %format!("{:08X}", command), command = %format!("{}", Gp0Command::from(command)), "GP0 command received");
            }
            _ => unreachable!(),
        }
    }
}
