use psx_core::mmu::Mmu;
use psx_core::mmu::bus::{Bus8, Bus16, Bus32};

pub struct MmuState {
    pub data: Box<[u8; 0xFFFF_FFFF]>,
}

impl Default for MmuState {
    fn default() -> Self {
        Self {
            data: vec![0xFF; 0xFFFF_FFFF].try_into().unwrap(),
        }
    }
}

impl Bus8 for MmuState {
    fn read_u8(&mut self, address: u32) -> u8 {
        let address = Mmu::canonicalize_virtual_address(address);
        self.data[address as usize]
    }

    fn write_u8(&mut self, address: u32, value: u8) {
        let address = Mmu::canonicalize_virtual_address(address);
        self.data[address as usize] = value;
    }
}

impl Bus16 for MmuState {
    fn read_u16(&mut self, address: u32) -> u16 {
        let address = Mmu::canonicalize_virtual_address(address);
        let low = self.read_u8(address) as u16;
        let high = self.read_u8(address + 1) as u16;
        (high << 8) | low
    }

    fn write_u16(&mut self, address: u32, value: u16) {
        let address = Mmu::canonicalize_virtual_address(address);
        self.write_u8(address, (value & 0xFF) as u8);
        self.write_u8(address + 1, (value >> 8) as u8);
    }
}

impl Bus32 for MmuState {
    fn read_u32(&mut self, address: u32) -> u32 {
        let address = Mmu::canonicalize_virtual_address(address);
        let a = self.read_u8(address) as u32;
        let b = self.read_u8(address + 1) as u32;
        let c = self.read_u8(address + 2) as u32;
        let d = self.read_u8(address + 3) as u32;
        (d << 24) | (c << 16) | (b << 8) | a
    }

    fn write_u32(&mut self, address: u32, value: u32) {
        let address = Mmu::canonicalize_virtual_address(address);
        self.write_u8(address, (value & 0xFF) as u8);
        self.write_u8(address + 1, (value >> 8) as u8);
        self.write_u8(address + 2, (value >> 16) as u8);
        self.write_u8(address + 3, (value >> 24) as u8);
    }
}
