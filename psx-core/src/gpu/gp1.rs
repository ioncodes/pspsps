use crate::mmu::Addressable;

pub struct Gp1;

impl Gp1 {
    pub fn new() -> Self {
        Self
    }
}

impl Addressable for Gp1 {
    fn read_u8(&self, address: u32) -> u8 {
        tracing::error!(target: "psx_core::gpu", address = %format!("{:08X}", address), "Reading from GP1 is not implemented");
        0xFF
    }

    fn write_u8(&mut self, address: u32, value: u8) {
        tracing::error!(target: "psx_core::gpu", address = %format!("{:08X}", address), value = %format!("{:02X}", value), "Writing to GP1 is not implemented");
    }
}
