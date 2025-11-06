pub struct Spu {
    pub sound_ram_data_transfer_fifo: u16, // 0x1F80_1DA8
    pub status_register: u16,              // 0x1F80_1DAE
}

impl Spu {
    pub fn new() -> Self {
        Spu {
            sound_ram_data_transfer_fifo: 0,
            status_register: 0,
        }
    }

    pub fn read(&self, address: u32) -> u8 {
        match address {
            0x1F80_1DA8 => self.sound_ram_data_transfer_fifo as u8,
            0x1F80_1DA9 => (self.sound_ram_data_transfer_fifo >> 8) as u8,
            0x1F80_1DAE => self.status_register as u8,
            0x1F80_1DAF => (self.status_register >> 8) as u8,
            _ => {
                tracing::warn!(target: "psx_core::spu", "Attempted to read unimplemented SPU register: {:08X}", address);
                0
            }
        }
    }

    pub fn write(&mut self, address: u32, value: u8) {
        match address {
            0x1F80_1DA8 => {
                self.sound_ram_data_transfer_fifo =
                    (self.sound_ram_data_transfer_fifo & 0xFF00) | (value as u16);
            }
            0x1F80_1DA9 => {
                self.sound_ram_data_transfer_fifo =
                    (self.sound_ram_data_transfer_fifo & 0x00FF) | ((value as u16) << 8);
            }
            _ => {
                tracing::warn!(target: "psx_core::spu", "Attempted to write to unimplemented SPU register: {:08X}", address);
            }
        }
    }
}
