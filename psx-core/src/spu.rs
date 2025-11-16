use crate::mmu::bus::Bus8;

crate::define_addr!(SOUND_RAM_TRANSFER_FIFO_ADDR, 0x1F80_1DA8, 0, 0x02, 0x02);
crate::define_addr!(STATUS_REGISTER_ADDR, 0x1F80_1DAE, 0, 0x02, 0x02);

pub struct Spu {
    pub fake_ram: Vec<u8>,
    cycles: usize,
}


impl Spu {
    pub fn new() -> Self {
        Spu {
            fake_ram: vec![0; 512 * 1024],
            cycles: 0,
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.cycles += cycles;
    }

    pub fn irq_pending(&mut self) -> bool {
        if self.cycles >= 75600 {
            self.cycles -= 75600;
            true
        } else {
            false
        }
    }
}

impl Bus8 for Spu {
    fn read_u8(&mut self, address: u32) -> u8 {
        tracing::error!(target: "psx_core::spu", address = format!("{:08X}", address), "SPU read_u8 not implemented");
        self.fake_ram[address as usize - 0x1F80_1C00]
    }

    fn write_u8(&mut self, address: u32, value: u8) {
        tracing::debug!(target: "psx_core::spu", address = format!("{:08X}", address), value = format!("{:02X}", value), "SPU write_u8 not implemented");
        self.fake_ram[address as usize - 0x1F80_1C00] = value;
    }
}
