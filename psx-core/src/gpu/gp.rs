use crate::gpu::cmd::Gp0Command;
use crate::gpu::status::StatusRegister;
use crate::gpu::{GP1_ADDRESS_END, GP1_ADDRESS_START, VRAM_HEIGHT, VRAM_WIDTH};
use crate::mmu::bus::Bus32;
use std::collections::VecDeque;

pub struct ParsedCommand {
    pub raw: u32,
    pub cmd: Gp0Command,
    pub data: Vec<u32>,
    pub ready: bool,
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
    read_counter: usize,
    gp1_status: StatusRegister,
}

impl Gp {
    pub fn new() -> Self {
        Self {
            fifo: VecDeque::with_capacity(16),
            expected_data: 0,
            state: State::WaitingForCommand,
            vram: vec![0; VRAM_WIDTH * VRAM_HEIGHT * 2], // 2 bytes per pixel (16bit)
            read_counter: 0,
            gp1_status: StatusRegister(0),
        }
    }

    pub fn process_read(&mut self, address: u32) -> u32 {
        if self.fifo.is_empty() {
            return 0xFF;
        }

        let last_cmd = self.fifo.back_mut().unwrap();

        if self.expected_data > 0 {
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

                // Calculate current pixel position based on words read (2 pixels per word)
                let pixels_read = self.read_counter * 2; // 2 pixels per 32-bit word
                let pixel0_x = src_x + (pixels_read % width);
                let pixel0_y = src_y + (pixels_read / width);
                let pixel1_x = src_x + ((pixels_read + 1) % width);
                let pixel1_y = src_y + ((pixels_read + 1) / width);

                // Read two 16-bit pixels and pack into 32-bit word
                let idx0 = ((pixel0_y * 1024) + pixel0_x) * 2;
                let idx1 = ((pixel1_y * 1024) + pixel1_x) * 2;

                let pixel0 = u16::from_le_bytes([self.vram[idx0], self.vram[idx0 + 1]]);
                let pixel1 = u16::from_le_bytes([self.vram[idx1], self.vram[idx1 + 1]]);

                let word = (pixel1 as u32) << 16 | (pixel0 as u32);

                tracing::debug!(
                    target: "psx_core::gpu",
                    command = %last_cmd.cmd, vram_addr = %format!("{:08X}", address), self.expected_data, data = %format!("{:08X?}", last_cmd.data), src_x, src_y, pixel0_x, pixel0_y, pixel1_x, pixel1_y, word = %format!("{:08X}", word),
                    "Reading from VRAM during GP0 VramToCpuBlit command"
                );

                self.read_counter += 1;

                word
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

    pub fn process_gp0_word(&mut self, value: u32) {
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
            raw: value,
            cmd,
            data: Vec::with_capacity(self.expected_data),
            ready: self.expected_data == 0,
        });

        // Reset current command/data buffer
        self.read_counter = 0;

        // Does the command need any data?
        if self.expected_data > 0 {
            self.state = State::CollectingParams;
        }
    }

    pub fn process_gp1_cmd(&mut self, word: u32) {
        let cmd = (word >> 24) & 0xFF;
        let params = word & 0x00FF_FFFF;

        match cmd {
            // Reset GPU
            0x00 => {
                self.gp1_status.0 = 0x1480_2000;
                self.vram.fill(0);
                self.fifo.clear();
                self.expected_data = 0;
                self.state = State::WaitingForCommand;
                self.read_counter = 0;

                tracing::trace!(target: "psx_core::gpu", "GPU Reset via GP1 command");
            }
            // Reset Command Buffer
            0x01 => {
                self.fifo.clear();
                self.expected_data = 0;
                self.state = State::WaitingForCommand;

                tracing::trace!(target: "psx_core::gpu", "Command Buffer Reset via GP1 command");
            }
            // Display Mode
            0x08 => {
                let hres1 = (params & 0b11) as u8;
                let vres = ((params >> 2) & 0b1) == 1;
                let video_mode = ((params >> 3) & 0b1) == 1;
                // TODO: display area color depth
                let interlace = ((params >> 5) & 0b1) == 1;
                let hres2 = ((params >> 6) & 0b1) == 1;
                // TODO: flip screen horizontally

                self.gp1_status.set_horizontal_resolution1(hres1);
                self.gp1_status.set_horizontal_resolution2(hres2);
                self.gp1_status.set_vertical_resolution(vres);
                self.gp1_status.set_vertical_interlace(interlace);
                self.gp1_status.set_video_mode(video_mode.into());

                tracing::trace!(
                    target: "psx_core::gpu",
                    hres = self.gp1_status.hres(), vres = self.gp1_status.vres(),
                    "Set GPU display mode via GP1 command"
                );
            }
            _ => {
                tracing::error!(
                    target: "psx_core::gpu",
                    cmd = format!("{:02X}", cmd),
                    "Unknown GP1 command"
                );
            }
        }
    }

    #[inline(always)]
    pub fn pop_command(&mut self) -> Option<ParsedCommand> {
        if let Some(cmd) = self.fifo.front()
            && cmd.ready
        {
            return self.fifo.pop_front();
        }

        None
    }

    #[inline(always)]
    pub fn resolution(&self) -> (usize, usize) {
        (
            self.gp1_status.hres() as usize,
            self.gp1_status.vres() as usize,
        )
    }

    #[inline(always)]
    pub fn status(&self) -> StatusRegister {
        // TODO: actually check these
        let mut gpustat = self.gp1_status;
        gpustat.set_ready_to_receive_cmd_word(true);
        gpustat.set_ready_to_receive_dma_block(true);
        gpustat.set_ready_to_send_vram_to_cpu(true);
        gpustat
    }

    #[inline(always)]
    pub fn fifo_len(&self) -> usize {
        self.fifo.len()
    }

    /// Generate a frame buffer from VRAM
    /// Converts entire VRAM to RGB888 format
    pub fn generate_frame(&self, buffer: &mut [(u8, u8, u8)]) {
        // Render entire VRAM (1024x512)
        for y in 0..512 {
            for x in 0..1024 {
                let vram_idx = (y * 1024 + x) * 2;

                // Read RGB555 pixel from VRAM
                let pixel_u16 = u16::from_le_bytes([self.vram[vram_idx], self.vram[vram_idx + 1]]);

                // Extract RGB555 components
                let r5 = (pixel_u16 & 0x1F) as u8;
                let g5 = ((pixel_u16 >> 5) & 0x1F) as u8;
                let b5 = ((pixel_u16 >> 10) & 0x1F) as u8;

                // Convert RGB555 to RGB888
                let r8 = (r5 << 3) | (r5 >> 2);
                let g8 = (g5 << 3) | (g5 >> 2);
                let b8 = (b5 << 3) | (b5 >> 2);

                // Buffer is 1024 pixels wide (VRAM_WIDTH)
                let buffer_idx = y * VRAM_WIDTH + x;
                buffer[buffer_idx] = (r8, g8, b8);
            }
        }
    }
}

impl Bus32 for Gp {
    #[inline(always)]
    fn read_u32(&mut self, address: u32) -> u32 {
        if address >= GP1_ADDRESS_START && address <= GP1_ADDRESS_END {
            return self.status().0;
        }

        self.process_read(address)
    }

    #[inline(always)]
    fn write_u32(&mut self, address: u32, value: u32) {
        if address >= GP1_ADDRESS_START && address <= GP1_ADDRESS_END {
            self.process_gp1_cmd(value);
        } else {
            self.process_gp0_word(value);
        }
    }
}
