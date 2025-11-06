use crate::mmu::bus::Bus8;

crate::define_addr!(SOUND_RAM_TRANSFER_FIFO_ADDR, 0x1F80_1DA8, 0, 0x02, 0x02);
crate::define_addr!(STATUS_REGISTER_ADDR, 0x1F80_1DAE, 0, 0x02, 0x02);

pub struct Spu {
    pub fake_ram: Vec<u8>,
}

impl Spu {
    pub fn new() -> Self {
        Spu {
            fake_ram: vec![0; 512 * 1024],
        }
    }
}

impl Bus8 for Spu {
    fn read_u8(&mut self, address: u32) -> u8 {
        self.fake_ram[address as usize - 0x1F80_1C00]
    }

    fn write_u8(&mut self, address: u32, value: u8) {
        self.fake_ram[address as usize - 0x1F80_1C00] = value;
    }
}
