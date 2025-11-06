#[derive(Debug, Clone)]
pub struct Mmu {
    pub memory: Box<[u8; 0xFFFF_FFFF]>, // 512 KB BIOS
}

impl Mmu {
    pub fn new() -> Self {
        Self {
            memory: vec![0; 0xFFFF_FFFF].try_into().unwrap(),
        }
    }

    pub fn load(&mut self, data: &[u8], address: u32) {
        self.memory[address as usize..address as usize + data.len()].copy_from_slice(data);
    }

    pub fn read_u8(&self, address: u32) -> u8 {
        self.memory[address as usize]
    }

    pub fn read_u16(&self, address: u32) -> u16 {
        let offset = address as usize;
        u16::from_le_bytes([self.memory[offset], self.memory[offset + 1]])
    }

    pub fn read_u32(&self, address: u32) -> u32 {
        let offset = address as usize;
        u32::from_le_bytes([
            self.memory[offset],
            self.memory[offset + 1],
            self.memory[offset + 2],
            self.memory[offset + 3],
        ])
    }

    pub fn write_u8(&mut self, address: u32, value: u8) {
        self.memory[address as usize] = value;
    }

    pub fn write_u16(&mut self, address: u32, value: u16) {
        let offset = address as usize;
        self.memory[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    pub fn write_u32(&mut self, address: u32, value: u32) {
        let offset = address as usize;
        self.memory[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}
