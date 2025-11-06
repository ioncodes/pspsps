use psx_core::mmu::Addressable;

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

impl Addressable for MmuState {
    fn read_u8(&mut self, address: u32) -> u8 {
        let address = Self::canonicalize_virtual_address(address);
        self.data[address as usize]
    }

    fn write_u8(&mut self, address: u32, value: u8) {
        let address = Self::canonicalize_virtual_address(address);
        self.data[address as usize] = value;
    }
}
