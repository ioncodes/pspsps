pub mod cmd;
pub mod gp;

use crate::gpu::cmd::Gp0Command;
use crate::gpu::gp::Gp;
use crate::mmu::Addressable;

pub const GP0_ADDRESS_START: u32 = 0x1F80_1810;
pub const GP0_ADDRESS_END: u32 = 0x1F80_1813;
pub const GP1_ADDRESS_START: u32 = 0x1F80_1814;
pub const GP1_ADDRESS_END: u32 = 0x1F80_1817;

pub struct Gpu {
    pub gp: Gp,
}

impl Gpu {
    pub fn new() -> Self {
        Self {
            gp: Gp::new(),
        }
    }

    pub fn tick(&mut self) {
        if let Some(parsed_cmd) = self.gp.pop_command() {
            match parsed_cmd.cmd {
                Gp0Command::RectanglePrimitive(_) => {
                    let x = parsed_cmd.data[0] & 0xFFFF;
                    let y = (parsed_cmd.data[0] >> 16) & 0xFFFF;
                    let width = parsed_cmd.data[2] & 0xFFFF;
                    let height = (parsed_cmd.data[2] >> 16) & 0xFFFF;

                    tracing::debug!(target: "psx_core::gpu", "Draw rectangle at ({}, {}) with size {}x{}", x, y, width, height);
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
