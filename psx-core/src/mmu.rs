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

    pub fn load(&mut self, data: &[u8], address: u32) {
        self.memory[address as usize..address as usize + data.len()].copy_from_slice(data);
    }

    pub fn read_u8(&self, address: u32) -> u8 {
        tracing::trace!(target: "psx_core::mmu", "Reading byte from address: {:08X}", address);

        self.memory[address as usize]
    }

    pub fn read_u16(&self, address: u32) -> u16 {
        tracing::trace!(target: "psx_core::mmu", "Reading half-word from address: {:08X}", address);

        let address = address as usize;
        u16::from_le_bytes([self.memory[address], self.memory[address + 1]])
    }

    pub fn read_u32(&self, address: u32) -> u32 {
        tracing::trace!(target: "psx_core::mmu", "Reading word from address: {:08X}", address);

        let address = address as usize;
        u32::from_le_bytes([
            self.memory[address],
            self.memory[address + 1],
            self.memory[address + 2],
            self.memory[address + 3],
        ])
    }

    pub fn write_u8(&mut self, address: u32, value: u8) {
        tracing::trace!(target: "psx_core::mmu", "Writing {:02X} to address: {:08X}", value, address);

        self.memory[address as usize] = value;
    }

    pub fn write_u16(&mut self, address: u32, value: u16) {
        tracing::trace!(target: "psx_core::mmu", "Writing {:04X} to address: {:08X}", value, address);

        let address = address as usize;
        self.memory[address] = (value & 0xFF) as u8;
        self.memory[address + 1] = (value >> 8) as u8;
    }

    pub fn write_u32(&mut self, address: u32, value: u32) {
        tracing::trace!(target: "psx_core::mmu", "Writing {:08X} to address: {:08X}", value, address);

        let address = address as usize;
        self.memory[address] = (value & 0xFF) as u8;
        self.memory[address + 1] = ((value >> 8) & 0xFF) as u8;
        self.memory[address + 2] = ((value >> 16) & 0xFF) as u8;
        self.memory[address + 3] = ((value >> 24) & 0xFF) as u8;
    }
}
