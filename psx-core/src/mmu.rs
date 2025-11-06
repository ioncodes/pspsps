pub struct Mmu {
    pub memory: Box<[u8; 512 * 1024]>, // 512 KB BIOS
}

impl Mmu {
    pub fn new() -> Self {
        Self {
            memory: Box::new([0; 512 * 1024]),
        }
    }

    pub fn load(&mut self, data: &[u8], address: u32) {
        self.memory[address as usize..address as usize + data.len()].copy_from_slice(data);
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
}
