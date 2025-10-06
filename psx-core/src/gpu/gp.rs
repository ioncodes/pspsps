use crate::gpu::cmd::Gp0Command;
use crate::gpu::{GP0_ADDRESS_START, GP1_ADDRESS_END, GP1_ADDRESS_START};
use crate::mmu::Addressable;
use std::collections::VecDeque;

pub struct ParsedCommand {
    pub cmd: Gp0Command,
    pub data: Vec<u32>,
    pub ready: bool
}

#[derive(PartialEq, Eq)]
enum State {
    WaitingForCommand,
    CollectingParams,
    CollectingExtraData,
}

pub struct Gp {
    pub vram: Vec<u8>,
    fifo: VecDeque<ParsedCommand>,
    expected_data: usize,
    state: State,
    rw_cache: [u8; 4],
    read_counter: usize,
}

impl Gp {
    pub fn new() -> Self {
        Self {
            fifo: VecDeque::with_capacity(16),
            expected_data: 0,
            state: State::WaitingForCommand,
            rw_cache: [0; 4],
            vram: vec![0; 1024 * 1024],
            read_counter: 0,
        }
    }

    pub fn process_read(&mut self, address: u32) -> u8 {
        if self.fifo.is_empty() {
            return 0xFF;
        }
        
        let last_cmd = self.fifo.back_mut().unwrap();

        // Only decrement expected data on word boundaries
        if self.expected_data > 0 && address % 4 == 0 {
            self.expected_data -= 1;
        }

        if self.expected_data == 0 {
            self.state = State::WaitingForCommand;
            self.read_counter = 0;
            last_cmd.ready = true;
        }

        match last_cmd.cmd {
            Gp0Command::VramToCpuBlit => {
                let (src_x, src_y) = {
                    let word = last_cmd.data[0];
                    let x = word & 0xFFFF;
                    let y = word >> 16;
                    (x as usize, y as usize)
                };

                let (width, _height) = {
                    let word = last_cmd.data[1];
                    let w = word & 0xFFFF;
                    let h = word >> 16;
                    (w as usize, h as usize)
                };

                // Calculate current pixel position based on bytes read
                let pixels_read = self.read_counter / 2;  // 2 bytes per pixel
                let current_x = src_x + (pixels_read % width);
                let current_y = src_y + (pixels_read / width);

                let byte_in_pixel = self.read_counter % 2;
                let idx = ((current_y * 1024) + current_x) * 2 + byte_in_pixel;

                tracing::debug!(
                    target: "psx_core::gpu",
                    command = %last_cmd.cmd, vram_addr = %format!("{:08X}", address), self.expected_data, data = %format!("{:08X?}", last_cmd.data), src_x, src_y, current_x, current_y, idx,
                    "Reading from VRAM during GP0 VramToCpuBlit command"
                );

                self.read_counter += 1;

                self.vram[idx]
            }
            _ => {
                tracing::error!(
                    target: "psx_core::gpu",
                    command = %last_cmd.cmd, address, self.expected_data,
                    "Reading from GP0 during illegal state"
                );
                0xFF
            }
        }
    }

    pub fn process_word(&mut self, value: u32) {
        // Are we collecting extra data for a command?
        if self.expected_data > 0 {
            let last_cmd = self.fifo.back_mut().unwrap();
            last_cmd.data.push(value);

            tracing::debug!(
                target: "psx_core::gpu",
                command = %last_cmd.cmd, extra_data = format!("{:08X?}", last_cmd.data),
                "Collecting extra data for GP0 command"
            );

            self.expected_data -= 1;

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
                                let width = (value & 0xFFFF) as usize;
                                let height = (value >> 16) as usize;
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
                self.rw_cache = [0; 4];

                if self.expected_data > 0 {
                    // Still expecting more data
                    return;
                } else {
                    // Done processing command
                    self.state = State::WaitingForCommand;
                    last_cmd.ready = true;
                }
            }

            return;
        }

        // Extract command/data and push to FIFO
        let cmd = Gp0Command::from(value);
        self.expected_data = cmd.base_extra_data_count();
        tracing::debug!(target: "psx_core::gpu", command = %cmd, self.expected_data, "Received GP0 command");

        self.fifo.push_back(ParsedCommand {
            cmd,
            data: Vec::with_capacity(self.expected_data),
            ready: self.expected_data == 0
        });

        // Reset current command/data buffer
        self.rw_cache = [0; 4];
        self.read_counter = 0;

        // Does the command need any data?
        if self.expected_data > 0 {
            self.state = State::CollectingParams;
        }
    }

    pub fn pop_command(&mut self) -> Option<ParsedCommand> {
        if let Some(cmd) = self.fifo.front() && cmd.ready {
            return self.fifo.pop_front();
        }

        None
    }
}

impl Addressable for Gp {
    fn read_u8(&mut self, address: u32) -> u8 {
        if address >= GP1_ADDRESS_START && address <= GP1_ADDRESS_END {
            tracing::error!(target: "psx_core::gpu", address = %format!("{:08X}", address), "GP1 not implemented");
            return 0xFF;
        }

        self.process_read(address)
    }

    fn write_u8(&mut self, address: u32, value: u8) {
        if address >= GP1_ADDRESS_START && address <= GP1_ADDRESS_END {
            tracing::error!(target: "psx_core::gpu", address = %format!("{:08X}", address), value = %format!("{:02X}", value), "GP1 not implemented");
            return;
        }

        match address % GP0_ADDRESS_START {
            0 => self.rw_cache[0] = value,
            1 => self.rw_cache[1] = value,
            2 => self.rw_cache[2] = value,
            3 => {
                self.rw_cache[3] = value;
                let word = u32::from_le_bytes(self.rw_cache);
                self.process_word(word);
            }
            _ => unreachable!(),
        }
    }
}
