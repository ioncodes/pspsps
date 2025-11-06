use crate::gpu::GP0_ADDRESS_START;
use crate::gpu::cmd::Gp0Command;
use crate::mmu::Addressable;
use std::collections::VecDeque;

struct ParsedCommand {
    pub cmd: Gp0Command,
    pub data: Vec<u32>,
}

#[derive(PartialEq, Eq)]
enum State {
    WaitingForCommand,
    CollectingParams,
    CollectingExtraData,
}

pub struct Gp0 {
    fifo: VecDeque<ParsedCommand>,
    current_buffer: [u8; 4],
    expected_data: usize,
    state: State,
}

impl Gp0 {
    pub fn new() -> Self {
        Self {
            fifo: VecDeque::with_capacity(16),
            current_buffer: [0; 4],
            expected_data: 0,
            state: State::WaitingForCommand,
        }
    }

    fn process_command(&mut self) {}

    fn process_word(&mut self) {}
}

impl Addressable for Gp0 {
    fn read_u8(&self, address: u32) -> u8 {
        tracing::error!(target: "psx_core::gpu", address = %format!("{:08X}", address), "Reading from GP0 is not implemented");
        0xFF
    }

    fn write_u8(&mut self, address: u32, value: u8) {
        match address % GP0_ADDRESS_START {
            0 => self.current_buffer[0] = value,
            1 => self.current_buffer[1] = value,
            2 => self.current_buffer[2] = value,
            3 => {
                self.current_buffer[3] = value;

                let word = u32::from_le_bytes(self.current_buffer);

                // Are we collecting extra data for a command?
                if self.expected_data > 0 {
                    self.expected_data -= 1;

                    let last_cmd = self.fifo.back_mut().unwrap();
                    last_cmd.data.push(word);

                    tracing::debug!(
                        target: "psx_core::gpu",
                        command = %last_cmd.cmd, extra_data = format!("{:08X?}", last_cmd.data),
                        "Collecting extra data for GP0 command"
                    );

                    // Are we done collecting extra data?
                    if self.expected_data == 0 {
                        tracing::debug!(
                            target: "psx_core::gpu",
                            command = %last_cmd.cmd, extra_data = format!("{:08X?}", last_cmd.data),
                            "Finished collecting extra data for GP0 command"
                        );

                        // We only do this if we're collecting params, not extra data
                        // Allowing this during extra data collection would result in self.expected_data being overwritten
                        if self.state == State::CollectingParams {
                            // Is a variable amount of extra data expected?
                            match last_cmd.cmd {
                                Gp0Command::CpuToVramBlit | Gp0Command::VramToCpuBlit => {
                                    let (width, height) = {
                                        let width = (word & 0xFFFF) as usize;
                                        let height = (word >> 16) as usize;
                                        (width, height)
                                    };
                                    self.expected_data = ((width * height) + 1) / 2;
                                    self.state = State::CollectingExtraData;

                                    tracing::debug!(
                                        target: "psx_core::gpu",
                                        command = %last_cmd.cmd, extra_data = format!("{:08X?}", last_cmd.data), self.expected_data, width, height,
                                        "Expecting variable extra data for GP0 command"
                                    );
                                }
                                _ => {}
                            }
                        }

                        // Reset current command/data buffer
                        self.current_buffer = [0; 4];

                        if self.expected_data > 0 {
                            // Still expecting more data
                            return;
                        } else {
                            // Done processing command
                            self.state = State::WaitingForCommand;
                        }
                    }

                    return;
                }

                // Extract command/data and push to FIFO
                let cmd = Gp0Command::from(word);
                self.expected_data = cmd.base_extra_data_count();
                tracing::debug!(target: "psx_core::gpu", command = %cmd, self.expected_data, "Received GP0 command");

                self.fifo.push_back(ParsedCommand {
                    cmd,
                    data: Vec::with_capacity(self.expected_data),
                });

                // Reset current command/data buffer
                self.current_buffer = [0; 4];

                // Does the command need any data?
                if self.expected_data > 0 {
                    self.state = State::CollectingParams;
                }
            }
            _ => unreachable!(),
        }
    }
}
