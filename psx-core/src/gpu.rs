pub mod cmd;
pub mod gp;

use crate::gpu::cmd::{DrawRectangleCommand, Gp0Command};
use crate::gpu::gp::{Gp, ParsedCommand};
use crate::mmu::bus::Bus32;

pub const VRAM_WIDTH: usize = 1024;
pub const VRAM_HEIGHT: usize = 512;

pub const GP0_ADDRESS_START: u32 = 0x1F80_1810;
pub const GP0_ADDRESS_END: u32 = 0x1F80_1813;
pub const GP1_ADDRESS_START: u32 = 0x1F80_1814;
pub const GP1_ADDRESS_END: u32 = 0x1F80_1817;

pub struct Gpu {
    pub gp: Gp,
}

impl Gpu {
    pub fn new() -> Self {
        Self { gp: Gp::new() }
    }

    /// Generate a texture buffer from the current VRAM contents
    pub fn internal_frame(&self) -> Vec<(u8, u8, u8)> {
        let mut frame = vec![(0, 0, 0); VRAM_WIDTH * VRAM_HEIGHT];
        self.gp.generate_frame(&mut frame);
        frame
    }

    /// Generate the display area as it would appear on screen
    /// Returns a buffer sized to the current display resolution
    pub fn display_frame(&self) -> Vec<(u8, u8, u8)> {
        let (width, height) = self.gp.resolution();
        let mut buffer = vec![(0, 0, 0); width * height];

        // TODO: respect display area X/Y position from GP1 commands
        // For now, assume display starts at (0, 0) in VRAM
        let display_x = 0;
        let display_y = 0;

        for y in 0..height {
            for x in 0..width {
                let vram_x = display_x + x;
                let vram_y = display_y + y;

                if vram_x < VRAM_WIDTH && vram_y < VRAM_HEIGHT {
                    let vram_idx = (vram_y * VRAM_WIDTH + vram_x) * 2;

                    // Read RGB555 pixel from VRAM
                    let pixel_u16 =
                        u16::from_le_bytes([self.gp.vram[vram_idx], self.gp.vram[vram_idx + 1]]);

                    // Extract RGB555 components
                    let r5 = (pixel_u16 & 0x1F) as u8;
                    let g5 = ((pixel_u16 >> 5) & 0x1F) as u8;
                    let b5 = ((pixel_u16 >> 10) & 0x1F) as u8;

                    // Convert RGB555 to RGB888
                    let r8 = (r5 << 3) | (r5 >> 2);
                    let g8 = (g5 << 3) | (g5 >> 2);
                    let b8 = (b5 << 3) | (b5 >> 2);

                    let buffer_idx = y * width + x;
                    buffer[buffer_idx] = (r8, g8, b8);
                }
            }
        }

        buffer
    }

    pub fn tick(&mut self) {
        if let Some(parsed_cmd) = self.gp.pop_command() {
            match parsed_cmd.cmd {
                Gp0Command::RectanglePrimitive(cmd) => {
                    self.process_rectangle_primitive_cmd(parsed_cmd, cmd)
                }
                _ => {
                    tracing::error!(target: "psx_core::gpu", cmd = %parsed_cmd.cmd, raw = %format!("{:032b} / {:08X}", parsed_cmd.raw, parsed_cmd.raw), "Unimplemented GP0 command");
                }
            }
        }
    }

    fn process_rectangle_primitive_cmd(
        &mut self, outer_cmd: ParsedCommand, cmd: DrawRectangleCommand,
    ) {
        let x = (outer_cmd.data[cmd.vertex_idx()] & 0xFFFF) as i16;
        let y = ((outer_cmd.data[cmd.vertex_idx()] >> 16) & 0xFFFF) as i16;

        let (width, height) = match cmd.size() {
            0b00 => (
                (outer_cmd.data[cmd.size_idx()] & 0xFFFF) as u16,
                ((outer_cmd.data[cmd.size_idx()] >> 16) & 0xFFFF) as u16,
            ),
            0b01 => (1, 1),
            0b10 => (8, 8),
            0b11 => (16, 16),
            _ => unreachable!(),
        };

        tracing::debug!(
            target: "psx_core::gpu",
            x, y, width, height, color = format!("{:08X}", cmd.color()), textured = cmd.textured(), size = %format!("{:02b}", cmd.size() & 0b11), expected_extra_data = outer_cmd.cmd.base_extra_data_count(),
            "Draw rectangle primitive"
        );

        // coordinates can be negative, this is relative for primitives that go off-screen
        // but in case of 1x1 we can ignore them
        if x < 0 || y < 0 {
            return;
        }

        // extract RGB565 color components
        let r = (cmd.color() & 0xFF) as u8;
        let g = ((cmd.color() >> 8) & 0xFF) as u8;
        let b = ((cmd.color() >> 16) & 0xFF) as u8;

        // Convert RGB888 to RGB555
        let r5 = (r >> 3) & 0x1F;
        let g5 = (g >> 3) & 0x1F;
        let b5 = (b >> 3) & 0x1F;
        let pixel_value = (b5 as u16) << 10 | (g5 as u16) << 5 | (r5 as u16);

        // write to VRAM
        for row in 0..height {
            for col in 0..width {
                let vram_x = x as usize + col as usize;
                let vram_y = y as usize + row as usize;

                if vram_x < 1024 && vram_y < 512 {
                    let vram_idx = (vram_y * 1024 + vram_x) * 2;
                    let bytes = pixel_value.to_le_bytes();
                    self.gp.vram[vram_idx] = bytes[0];
                    self.gp.vram[vram_idx + 1] = bytes[1];
                }
            }
        }
    }
}

impl Bus32 for Gpu {
    #[inline(always)]
    fn read_u32(&mut self, address: u32) -> u32 {
        self.gp.read_u32(address)
    }

    #[inline(always)]
    fn write_u32(&mut self, address: u32, value: u32) {
        self.gp.write_u32(address, value);
    }
}
