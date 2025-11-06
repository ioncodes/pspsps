pub mod bus;
pub mod dma;

use crate::cdrom::{CDROM_ADDR_END, CDROM_ADDR_START, Cdrom};
use crate::cdrom::reg::REG_RDDATA_ADDR;
use crate::gpu::status::DmaDirection;
use crate::gpu::{GP0_ADDRESS_END, GP0_ADDRESS_START, GP1_ADDRESS_END, GP1_ADDRESS_START, Gpu};
use crate::irq::{I_MASK_ADDR_END, I_MASK_ADDR_START, I_STAT_ADDR_END, I_STAT_ADDR_START, Irq};
use crate::mmu::bus::{Bus8 as _, Bus32};
use crate::mmu::dma::{Channel, DMA_INTERRUPT_REGISTER_ADDRESS_END, DMA0_ADDRESS_START, Dma, TransferMode};
use crate::sio::{SIO_ADDR_END, SIO_ADDR_START, Sio};
use crate::spu::Spu;

pub struct Mmu {
    pub memory: Box<[u8; 0xFFFF_FFFF]>, // 512 KB BIOS
    pub cdrom: Cdrom,
    pub spu: Spu,
    pub gpu: Gpu,
    pub dma: Dma,
    pub irq: Irq,
    pub sio: Sio,
}

impl Mmu {
    pub fn new() -> Self {
        Self {
            memory: vec![0xFF; 0xFFFF_FFFF].try_into().unwrap(),
            cdrom: Cdrom::new(),
            spu: Spu::new(),
            gpu: Gpu::new(),
            dma: Dma::new(),
            irq: Irq::new(),
            sio: Sio::new()
        }
    }

    pub fn perform_dma_transfers(&mut self) {
        if self.dma.channels.0.channel_control.start_transfer() {
            self.transfer_dma_channel(self.dma.channels.0);
        }

        if self.dma.channels.1.channel_control.start_transfer() {
            self.transfer_dma_channel(self.dma.channels.1);
        }

        if self.dma.channels.2.channel_control.start_transfer() {
            self.transfer_dma_channel(self.dma.channels.2);
            if self.gpu.gp.gp1_status.dma_direction() != DmaDirection::CpuToGpu {
                tracing::error!(
                    target: "psx_core::mmu",
                    channel_id = 2,
                    dma_direction = %self.gpu.gp.gp1_status.dma_direction(),
                    "DMA is ready, but GPUSTAT is in wrong state"
                );
            }
        }

        if self.dma.channels.3.channel_control.start_transfer() {
            self.transfer_dma_channel(self.dma.channels.3);
        }

        if self.dma.channels.4.channel_control.start_transfer() {
            self.transfer_dma_channel(self.dma.channels.4);
        }

        if self.dma.channels.5.channel_control.start_transfer() {
            self.transfer_dma_channel(self.dma.channels.5);
        }

        if self.dma.channels.6.channel_control.start_transfer() {
            self.transfer_dma_channel(self.dma.channels.6);
        }
    }

    #[inline(always)]
    pub fn transfer_dma_channel<const CHANNEL_ID: u8>(&mut self, channel: Channel<CHANNEL_ID>) {
        tracing::debug!(
            target: "psx_core::dma",
            channel_id = CHANNEL_ID,
            base_address = %format!("{:08X}", channel.base_address),
            direction = if channel.channel_control.transfer_direction() {
                "RAM to device"
            } else {
                "Device to RAM"
            },
            transfer_mode = %channel.channel_control.transfer_mode(),
            block_control = %format!("{:08X}", channel.block_control()),
            madr_step = if channel.channel_control.madr_increment_per_step() {
                "-4"
            } else {
                "+4"
            },
            "DMA transfer started"
        );

        match CHANNEL_ID {
            2 => self.perform_gpu_dma(),
            3 => self.perform_cdrom_dma(),
            6 => self.perform_otc_dma(),
            _ => {
                tracing::error!(target: "psx_core::dma", channel_id = CHANNEL_ID, "DMA transfer not implemented for this channel")
            }
        }

        // Clear the start_transfer bit after handling
        match CHANNEL_ID {
            0 => self.dma.channels.0.set_completed(),
            1 => self.dma.channels.1.set_completed(),
            2 => self.dma.channels.2.set_completed(),
            3 => self.dma.channels.3.set_completed(),
            4 => self.dma.channels.4.set_completed(),
            5 => self.dma.channels.5.set_completed(),
            6 => self.dma.channels.6.set_completed(),
            _ => unreachable!(),
        }
    }

    /// Perform a DMA transfer for channel 6 (OTC)
    /// Clears the linked list used for polygons
    pub fn perform_otc_dma(&mut self) {
        let channel = &self.dma.channels.6;

        let madr = channel.base_address();
        let bcr = channel.bcr_word_count();

        let mut write_u32 = |addr, value| {
            self.write_u32(addr, value);
        };

        let mut current_addr = madr;

        if bcr == 1 {
            write_u32(current_addr, 0xFFFFFFFF);
        } else {
            for i in 0..bcr {
                if i == bcr - 1 {
                    // Last entry gets end marker
                    write_u32(current_addr, 0xFFFFFFFF);
                } else {
                    // Each entry points to next entry
                    write_u32(current_addr, current_addr.wrapping_sub(4));
                }
                current_addr = current_addr.wrapping_sub(4);
            }
        }
    }

    pub fn perform_gpu_dma(&mut self) {
        let channel = self.dma.channels.2;
        let transfer_direction = channel.channel_control.transfer_direction();
        let madr_step = channel.channel_control.madr_step();

        match channel.channel_control.transfer_mode() {
            TransferMode::Burst => {
                let total_words = channel.bcr_word_count();
                let mut addr = channel.base_address();

                if transfer_direction {
                    // RAM to GPU
                    for _ in 0..total_words {
                        let word = self.read_u32(addr);
                        self.write_u32(GP0_ADDRESS_START, word);
                        addr = addr.wrapping_add_signed(madr_step);
                    }
                } else {
                    // GPU to RAM
                    for _ in 0..total_words {
                        let word = self.read_u32(GP0_ADDRESS_START);
                        self.write_u32(addr, word);
                        addr = addr.wrapping_add_signed(madr_step);
                    }
                }
            }
            TransferMode::Slice => {
                let total_words = channel.bcr_block_total();
                let mut addr = channel.base_address();

                if transfer_direction {
                    // RAM to GPU
                    for _ in 0..total_words {
                        let word = self.read_u32(addr);
                        self.write_u32(GP0_ADDRESS_START, word);
                        addr = addr.wrapping_add_signed(madr_step);
                    }
                } else {
                    // GPU to RAM
                    for _ in 0..total_words {
                        let word = self.read_u32(GP0_ADDRESS_START);
                        self.write_u32(addr, word);
                        addr = addr.wrapping_add_signed(madr_step);
                    }
                }
            }
            TransferMode::LinkedList => {
                // Linked list is only for RAM to GPU
                if !transfer_direction {
                    tracing::error!(
                        target: "psx_core::dma",
                        "GPU DMA LinkedList mode only supports RAM to GPU"
                    );
                    return;
                }

                let mut src_addr = channel.base_address();

                loop {
                    let header = self.read_u32(src_addr);
                    let next_addr = header & 0x00FFFFFF; // next node (24-bit address)
                    let words = header >> 24; // number of words in this node

                    tracing::trace!(
                        target: "psx_core::dma",
                        address = format!("{:08X}", src_addr),
                        words = words,
                        next_address = format!("{:08X}", next_addr),
                        is_end = next_addr == 0x00FFFFFF,
                        "Transfering DMA linked list node"
                    );

                    // Transfer all words from this node
                    // Start from the word after the header
                    let mut word_addr = src_addr.wrapping_add(4);
                    for _ in 0..words {
                        let word = self.read_u32(word_addr);
                        self.write_u32(GP0_ADDRESS_START, word);

                        tracing::trace!(
                            target: "psx_core::dma",
                            address = format!("{:08X}", word_addr),
                            word = format!("{:08X}", word),
                            "Sent word from linked list node to GP0"
                        );

                        word_addr = word_addr.wrapping_add(4);
                    }

                    // AFTER processing this node's data, check if this was the last node
                    if next_addr == 0x00FFFFFF {
                        break;
                    }

                    src_addr = next_addr;
                }
            }
        }
    }

    pub fn perform_cdrom_dma(&mut self) {
        let channel = self.dma.channels.3;

        let mut dest_addr = channel.base_address();
        let madr_step = channel.channel_control.madr_step();

        tracing::debug!(
            target: "psx_core::dma",
            dest_addr = %format!("{:08X}", dest_addr),
            transfer_mode = %channel.channel_control.transfer_mode(),
            "CDROM DMA transfer starting"
        );

        match channel.channel_control.transfer_mode() {
            TransferMode::Burst => {
                let total_words = channel.bcr_word_count();

                for _ in 0..total_words {
                    // CDROM data port is 8-bit; pack 4 bytes into a word (little-endian)
                    let b0 = self.read_u8(REG_RDDATA_ADDR) as u32;
                    let b1 = self.read_u8(REG_RDDATA_ADDR) as u32;
                    let b2 = self.read_u8(REG_RDDATA_ADDR) as u32;
                    let b3 = self.read_u8(REG_RDDATA_ADDR) as u32;
                    let word = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);
                    self.write_u32(dest_addr, word);

                    tracing::trace!(
                        target: "psx_core::dma",
                        address = format!("{:08X}", dest_addr),
                        word = format!("{:08X}", word),
                        "CDROM DMA word transferred to RAM"
                    );

                    dest_addr = dest_addr.wrapping_add_signed(madr_step);
                }
            }
            TransferMode::Slice => {
                let total_words = channel.bcr_block_total();

                for _ in 0..total_words {
                    // CDROM data port is 8-bit; pack 4 bytes into a word (little-endian)
                    let b0 = self.read_u8(REG_RDDATA_ADDR) as u32;
                    let b1 = self.read_u8(REG_RDDATA_ADDR) as u32;
                    let b2 = self.read_u8(REG_RDDATA_ADDR) as u32;
                    let b3 = self.read_u8(REG_RDDATA_ADDR) as u32;
                    let word = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);
                    self.write_u32(dest_addr, word);

                    tracing::trace!(
                        target: "psx_core::dma",
                        address = format!("{:08X}", dest_addr),
                        word = format!("{:08X}", word),
                        "CDROM DMA word transferred to RAM (Slice mode)"
                    );

                    dest_addr = dest_addr.wrapping_add_signed(madr_step);
                }
            }
            TransferMode::LinkedList => {
                tracing::error!(
                    target: "psx_core::dma",
                    "CDROM DMA does not support LinkedList mode"
                );
            }
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

    #[inline(always)]
    pub fn canonicalize_virtual_address(address: u32) -> u32 {
        // A0000000h -> 80000000h -> 00000000h
        // BF000000h -> 9F000000h -> 1F000000h
        address & 0x5FFF_FFFF
    }
}

impl bus::Bus8 for Mmu {
    #[inline(always)]
    fn read_u8(&mut self, address: u32) -> u8 {
        let address = Self::canonicalize_virtual_address(address);

        match address {
            SIO_ADDR_START..=SIO_ADDR_END => self.sio.read_u8(address),
            I_MASK_ADDR_START..=I_MASK_ADDR_END => self.irq.read_u8(address),
            I_STAT_ADDR_START..=I_STAT_ADDR_END => self.irq.read_u8(address),
            DMA0_ADDRESS_START..=DMA_INTERRUPT_REGISTER_ADDRESS_END => self.dma.read_u8(address),
            CDROM_ADDR_START..=CDROM_ADDR_END => self.cdrom.read_u8(address),
            0x1F80_1D80..=0x1F80_1DBB => self.spu.read(address), // TODO: not complete
            0x1F80_1000..=0x1F80_1FFF => {
                tracing::error!(target: "psx_core::mmu", address = %format!("{:08X}", address), "Reading from unimplemented I/O port");
                0xFF
            }
            _ => self.memory[address as usize],
        }
    }

    #[inline(always)]
    fn write_u8(&mut self, address: u32, value: u8) {
        let address = Self::canonicalize_virtual_address(address);
        match address {
            SIO_ADDR_START..=SIO_ADDR_END => self.sio.write_u8(address, value),
            I_MASK_ADDR_START..=I_MASK_ADDR_END => self.irq.write_u8(address, value),
            I_STAT_ADDR_START..=I_STAT_ADDR_END => self.irq.write_u8(address, value),
            DMA0_ADDRESS_START..=DMA_INTERRUPT_REGISTER_ADDRESS_END => self.dma.write_u8(address, value),
            CDROM_ADDR_START..=CDROM_ADDR_END => self.cdrom.write_u8(address, value),
            0x1F80_1D80..=0x1F80_1DBB => self.spu.write(address, value),
            0x1F80_1000..=0x1F80_1FFF => {
                tracing::error!(target: "psx_core::mmu", address = %format!("{:08X}", address), value = %format!("{:02X}", value), "Writing to unimplemented I/O port");
            }
            _ => self.memory[address as usize] = value,
        }
    }
}

impl bus::Bus16 for Mmu {
    #[inline(always)]
    fn read_u16(&mut self, address: u32) -> u16 {
        let address = Self::canonicalize_virtual_address(address);
        match address {
            SIO_ADDR_START..=SIO_ADDR_END => self.sio.read_u16(address),
            I_MASK_ADDR_START..=I_MASK_ADDR_END => self.irq.read_u16(address),
            I_STAT_ADDR_START..=I_STAT_ADDR_END => self.irq.read_u16(address),
            DMA0_ADDRESS_START..=DMA_INTERRUPT_REGISTER_ADDRESS_END => self.dma.read_u16(address),
            _ => u16::from_le_bytes([self.read_u8(address), self.read_u8(address + 1)]),
        }
    }

    #[inline(always)]
    fn write_u16(&mut self, address: u32, value: u16) {
        let address = Self::canonicalize_virtual_address(address);
        match address {
            SIO_ADDR_START..=SIO_ADDR_END => self.sio.write_u16(address, value),
            I_MASK_ADDR_START..=I_MASK_ADDR_END => self.irq.write_u16(address, value),
            I_STAT_ADDR_START..=I_STAT_ADDR_END => self.irq.write_u16(address, value),
            DMA0_ADDRESS_START..=DMA_INTERRUPT_REGISTER_ADDRESS_END => self.dma.write_u16(address, value),
            _ => {
                self.write_u8(address, (value & 0xFF) as u8);
                self.write_u8(address + 1, ((value >> 8) & 0xFF) as u8);
            }
        }
    }
}

impl bus::Bus32 for Mmu {
    #[inline(always)]
    fn read_u32(&mut self, address: u32) -> u32 {
        let address = Self::canonicalize_virtual_address(address);
        match address {
            0x1F801100..=0x1F80112F => 0xFF, // TODO: always return 0xFF for these (timers?). helps with getting to shell
            SIO_ADDR_START..=SIO_ADDR_END => self.sio.read_u32(address),
            I_STAT_ADDR_START..=I_STAT_ADDR_END => self.irq.read_u32(address),
            I_MASK_ADDR_START..=I_MASK_ADDR_END => self.irq.read_u32(address),
            DMA0_ADDRESS_START..=DMA_INTERRUPT_REGISTER_ADDRESS_END => self.dma.read_u32(address),
            GP0_ADDRESS_START..=GP0_ADDRESS_END => self.gpu.read_u32(address),
            GP1_ADDRESS_START..=GP1_ADDRESS_END => self.gpu.read_u32(address),
            _ => u32::from_le_bytes([
                self.read_u8(address),
                self.read_u8(address + 1),
                self.read_u8(address + 2),
                self.read_u8(address + 3),
            ]),
        }
    }

    #[inline(always)]
    fn write_u32(&mut self, address: u32, value: u32) {
        let address = Self::canonicalize_virtual_address(address);
        match address {
            SIO_ADDR_START..=SIO_ADDR_END => self.sio.write_u32(address, value),
            I_MASK_ADDR_START..=I_MASK_ADDR_END => self.irq.write_u32(address, value),
            I_STAT_ADDR_START..=I_STAT_ADDR_END => self.irq.write_u32(address, value),
            DMA0_ADDRESS_START..=DMA_INTERRUPT_REGISTER_ADDRESS_END => self.dma.write_u32(address, value),
            GP0_ADDRESS_START..=GP0_ADDRESS_END => self.gpu.write_u32(address, value),
            GP1_ADDRESS_START..=GP1_ADDRESS_END => self.gpu.write_u32(address, value),
            _ => {
                self.write_u8(address, (value & 0xFF) as u8);
                self.write_u8(address + 1, ((value >> 8) & 0xFF) as u8);
                self.write_u8(address + 2, ((value >> 16) & 0xFF) as u8);
                self.write_u8(address + 3, ((value >> 24) & 0xFF) as u8);
            }
        }
    }
}
