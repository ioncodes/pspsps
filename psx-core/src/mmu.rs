use crate::gpu::{GP0_ADDRESS_END, GP0_ADDRESS_START, GP1_ADDRESS_END, GP1_ADDRESS_START, Gpu};
use crate::spu::Spu;

pub struct Mmu {
    pub memory: Box<[u8; 0xFFFF_FFFF]>, // 512 KB BIOS
    pub spu: Spu,
    pub gpu: Gpu,
}

impl Mmu {
    pub fn new() -> Self {
        Self {
            memory: vec![0xFF; 0xFFFF_FFFF].try_into().unwrap(),
            spu: Spu::new(),
            gpu: Gpu::new(),
        }
    }

    pub fn load(&mut self, address: u32, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            self.write_u8(address + i as u32, byte);
        }
    }

    #[inline(always)]
    pub fn is_word_aligned(address: u32) -> bool {
        address & 0b11 == 0
    }

    #[inline(always)]
    pub fn word_align(address: u32) -> u32 {
        address & 0b11
    }
}

impl Addressable for Mmu {
    #[inline(always)]
    fn read_u8(&mut self, address: u32) -> u8 {
        let address = Self::canonicalize_virtual_address(address);

        match address {
            0x1F80_1D80..=0x1F80_1DBB => self.spu.read(address), // TODO: not complete
            GP0_ADDRESS_START..=GP0_ADDRESS_END => self.gpu.read_u8(address),
            GP1_ADDRESS_START..=GP1_ADDRESS_END => self.gpu.read_u8(address),
            0x1F80_1000..=0x1F80_1FFF => {
                tracing::error!(target: "psx_core::mmu", address = %format!("{:08X}", address), "Reading from unimplemented I/O port");
                0xFF
            }
            _ => self.memory[address as usize],
        }
    }

    #[inline(always)]
    #[tracing::instrument(level = "trace", skip(self), fields(address = %format!("{:08X}", address), value = %format!("{:02X}", value)))]
    fn write_u8(&mut self, address: u32, value: u8) {
        let address = Self::canonicalize_virtual_address(address);
        match address {
            0x1F80_1D80..=0x1F80_1DBB => self.spu.write(address, value),
            GP0_ADDRESS_START..=GP0_ADDRESS_END => self.gpu.write_u8(address, value),
            GP1_ADDRESS_START..=GP1_ADDRESS_END => self.gpu.write_u8(address, value),
            0x1F80_1000..=0x1F80_1FFF => {
                tracing::error!(target: "psx_core::mmu", address = %format!("{:08X}", address), value = %format!("{:02X}", value), "Writing to unimplemented I/O port");
            }
            _ => self.memory[address as usize] = value,
        }
    }
}

pub trait Addressable {
    fn read_u8(&mut self, address: u32) -> u8;
    fn write_u8(&mut self, address: u32, value: u8);

    #[inline(always)]
    fn read_u16(&mut self, address: u32) -> u16 {
        u16::from_le_bytes([self.read_u8(address), self.read_u8(address + 1)])
    }

    #[inline(always)]
    fn read_u32(&mut self, address: u32) -> u32 {
        u32::from_le_bytes([
            self.read_u8(address),
            self.read_u8(address + 1),
            self.read_u8(address + 2),
            self.read_u8(address + 3),
        ])
    }

    #[inline(always)]
    fn write_u16(&mut self, address: u32, value: u16) {
        self.write_u8(address, (value & 0xFF) as u8);
        self.write_u8(address + 1, ((value >> 8) & 0xFF) as u8);
    }

    #[inline(always)]
    fn write_u32(&mut self, address: u32, value: u32) {
        self.write_u8(address, (value & 0xFF) as u8);
        self.write_u8(address + 1, ((value >> 8) & 0xFF) as u8);
        self.write_u8(address + 2, ((value >> 16) & 0xFF) as u8);
        self.write_u8(address + 3, ((value >> 24) & 0xFF) as u8);
    }

    #[inline(always)]
    fn canonicalize_virtual_address(address: u32) -> u32 {
        // A0000000h -> 80000000h -> 00000000h
        // BF000000h -> 9F000000h -> 1F000000h
        address & 0x5FFF_FFFF
    }
}
