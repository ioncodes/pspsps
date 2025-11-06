#[derive(Debug, Clone)]
pub struct Mmu {
    pub memory: Box<[u8; 0xFFFF_FFFF]>, // 512 KB BIOS
}

impl Mmu {
    pub fn new() -> Self {
        Self {
            memory: vec![0xFF; 0xFFFF_FFFF].try_into().unwrap(),
        }
    }

    pub fn load(&mut self, address: u32, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            self.write_u8(address + i as u32, byte);
        }
    }

    #[inline(always)]
    pub fn read_u8(&self, address: u32) -> u8 {
        let address = Self::canonicalize_address(address);
        self.memory[address as usize]
    }

    #[inline(always)]
    pub fn read_u16(&self, address: u32) -> u16 {
        let address = Self::canonicalize_address(address);
        let address = address as usize;
        u16::from_le_bytes([self.memory[address], self.memory[address + 1]])
    }

    #[inline(always)]
    pub fn read_u32(&self, address: u32) -> u32 {
        let address = Self::canonicalize_address(address);
        let address = address as usize;
        u32::from_le_bytes([
            self.memory[address],
            self.memory[address + 1],
            self.memory[address + 2],
            self.memory[address + 3],
        ])
    }

    #[inline(always)]
    pub fn write_u8(&mut self, address: u32, value: u8) {
        let address = Self::canonicalize_address(address);
        self.memory[address as usize] = value;
    }

    #[inline(always)]
    pub fn write_u16(&mut self, address: u32, value: u16) {
        let address = Self::canonicalize_address(address);
        let address = address as usize;
        self.memory[address] = (value & 0xFF) as u8;
        self.memory[address + 1] = (value >> 8) as u8;
    }

    #[inline(always)]
    pub fn write_u32(&mut self, address: u32, value: u32) {
        let address = Self::canonicalize_address(address);
        let address = address as usize;
        self.memory[address] = (value & 0xFF) as u8;
        self.memory[address + 1] = ((value >> 8) & 0xFF) as u8;
        self.memory[address + 2] = ((value >> 16) & 0xFF) as u8;
        self.memory[address + 3] = ((value >> 24) & 0xFF) as u8;
    }

    #[inline(always)]
    pub(crate) const fn canonicalize_address(address: u32) -> u32 {
        // A0000000h -> 80000000h -> 00000000h
        // BF000000h -> 9F000000h -> 1F000000h
        address & 0x5FFF_FFFF
    }
}
