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
        let low = self.data[address as usize] as u16;
        let high = self.data[(address + 1) as usize] as u16;
        (high << 8) | low
    }

    fn write_u16(&mut self, address: u32, value: u16) {
        let address = Mmu::canonicalize_virtual_address(address);
        self.data[address as usize] = (value & 0xFF) as u8;
        self.data[(address + 1) as usize] = (value >> 8) as u8;
    }
}

impl Bus32 for MmuState {
    fn read_u32(&mut self, address: u32) -> u32 {
        let address = Mmu::canonicalize_virtual_address(address);
        let low = self.data[address as usize] as u32;
        let mid = self.data[(address + 1) as usize] as u32;
        let high = self.data[(address + 2) as usize] as u32;
        let quad = self.data[(address + 3) as usize] as u32;
        (quad << 24) | (high << 16) | (mid << 8) | low
    }

    fn write_u32(&mut self, address: u32, value: u32) {
        let address = Mmu::canonicalize_virtual_address(address);
        self.data[address as usize] = (value & 0xFF) as u8;
        self.data[(address + 1) as usize] = (value >> 8) as u8;
        self.data[(address + 2) as usize] = (value >> 16) as u8;
        self.data[(address + 3) as usize] = (value >> 24) as u8;
    }
}
