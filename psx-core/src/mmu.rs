use crate::spu::Spu;

pub struct Mmu {
    pub memory: Box<[u8; 0xFFFF_FFFF]>, // 512 KB BIOS
    pub spu: Spu,
}

impl Mmu {
    pub fn new() -> Self {
        Self {
            memory: vec![0xFF; 0xFFFF_FFFF].try_into().unwrap(),
            spu: Spu::new(),
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

        match address {
            0x1F80_1D80..=0x1F80_1DBB => self.spu.read(address), // TODO: not complete
            0x1F80_1000..=0x1F80_1FFF => {
                tracing::error!(target: "psx_core::mmu", "Reading from unimplemented I/O port: {:08X}", address);
                0xFF
            }
            _ => self.memory[address as usize],
        }
    }

    #[inline(always)]
    pub fn read_u16(&self, address: u32) -> u16 {
        let address = Self::canonicalize_address(address);
        u16::from_le_bytes([self.read_u8(address), self.read_u8(address + 1)])
    }

    #[inline(always)]
    pub fn read_u32(&self, address: u32) -> u32 {
        let address = Self::canonicalize_address(address);
        u32::from_le_bytes([
            self.read_u8(address),
            self.read_u8(address + 1),
            self.read_u8(address + 2),
            self.read_u8(address + 3),
        ])
    }

    #[inline(always)]
    pub fn write_u8(&mut self, address: u32, value: u8) {
        tracing::trace!(target: "psx_core::mmu", "write_u8({:08X}, {:02X})", address, value);

        let address = Self::canonicalize_address(address);
        match address {
            0x1F80_1D80..=0x1F80_1DBB => self.spu.write(address, value),
            0x1F80_1000..=0x1F80_1FFF => {
                tracing::error!(target: "psx_core::mmu", "Writing {:02X} to unimplemented I/O port: {:08X}", value, address);
            }
            _ => self.memory[address as usize] = value,
        }
    }

    #[inline(always)]
    pub fn write_u16(&mut self, address: u32, value: u16) {
        tracing::trace!(target: "psx_core::mmu", "write_u16({:08X}, {:04X})", address, value);

        let address = Self::canonicalize_address(address);
        self.write_u8(address, (value & 0xFF) as u8);
        self.write_u8(address + 1, ((value >> 8) & 0xFF) as u8);
    }

    #[inline(always)]
    pub fn write_u32(&mut self, address: u32, value: u32) {
        tracing::trace!(target: "psx_core::mmu", "write_u32({:08X}, {:08X})", address, value);

        let address = Self::canonicalize_address(address);
        self.write_u8(address, (value & 0xFF) as u8);
        self.write_u8(address + 1, ((value >> 8) & 0xFF) as u8);
        self.write_u8(address + 2, ((value >> 16) & 0xFF) as u8);
        self.write_u8(address + 3, ((value >> 24) & 0xFF) as u8);
    }

    #[inline(always)]
    pub(crate) const fn canonicalize_address(address: u32) -> u32 {
        // A0000000h -> 80000000h -> 00000000h
        // BF000000h -> 9F000000h -> 1F000000h
        address & 0x5FFF_FFFF
    }
}
