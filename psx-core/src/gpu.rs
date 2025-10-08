pub mod cmd;
pub mod gp;

use crate::gpu::cmd::Gp0Command;
use crate::gpu::gp::Gp;
use crate::mmu::Addressable;

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;

pub const GP0_ADDRESS_START: u32 = 0x1F80_1810;
pub const GP0_ADDRESS_END: u32 = 0x1F80_1813;
pub const GP1_ADDRESS_START: u32 = 0x1F80_1814;
pub const GP1_ADDRESS_END: u32 = 0x1F80_1817;

pub struct Gpu {
    pub gp: Gp,
    internal_frame: Vec<(u8, u8, u8)>,
}

impl Gpu {
    pub fn new() -> Self {
        Self { gp: Gp::new(), internal_frame: vec![(0, 0, 0); SCREEN_WIDTH * SCREEN_HEIGHT] }
    }

    pub fn internal_frame(&self) -> &Vec<(u8, u8, u8)> {
        &self.internal_frame
    }

    pub fn tick(&mut self) {
        if let Some(parsed_cmd) = self.gp.pop_command() {
            match parsed_cmd.cmd {
                Gp0Command::RectanglePrimitive(cmd) => {
                    let x = parsed_cmd.data[cmd.vertex_idx()] & 0xFFFF;
                    let y = (parsed_cmd.data[cmd.vertex_idx()] >> 16) & 0xFFFF;

                    let (width, height) = match cmd.size() {
                        0b00 => (
                            (parsed_cmd.data[cmd.size_idx()] & 0xFFFF) as u16,
                            ((parsed_cmd.data[cmd.size_idx()] >> 16) & 0xFFFF) as u16,
                        ),
                        0b01 => (1, 1),
                        0b10 => (8, 8),
                        0b11 => (16, 16),
                        _ => unreachable!(),
                    };

                    tracing::debug!(
                        target: "psx_core::gpu",
                        x, y, width, height, color = format!("{:08X}", cmd.color()), textured = cmd.textured(), size = %format!("{:02b}", cmd.size() & 0b11), expected_extra_data = parsed_cmd.cmd.base_extra_data_count(),
                        "Draw rectangle primitive"
                    );

                    let idx = (y as usize * 256) + x as usize;
                    for row in 0..height {
                        for col in 0..width {
                            let pixel_idx = idx + (row as usize * 256) + col as usize;
                            if pixel_idx < self.internal_frame.len() {
                                let r = (cmd.color() & 0xFF) as u8;
                                let g = ((cmd.color() >> 8) & 0xFF) as u8;
                                let b = ((cmd.color() >> 16) & 0xFF) as u8;
                                self.internal_frame[pixel_idx] = (r, g, b);
                            }
                        }
                    }
                }
                _ => {
                    tracing::error!(target: "psx_core::gpu", "Unimplemented GP0 command: {}", parsed_cmd.cmd);
                }
            }
        }
    }
}

impl Addressable for Gpu {
    #[inline(always)]
    fn read_u8(&mut self, address: u32) -> u8 {
        self.gp.read_u8(address)
    }

    #[inline(always)]
    fn write_u8(&mut self, address: u32, value: u8) {
        self.gp.write_u8(address, value);
    }
}
