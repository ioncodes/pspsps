pub mod bus;
pub mod dma;

use crate::cdrom::{CDROM_ADDR_END, CDROM_ADDR_START, Cdrom};
use crate::gpu::status::DmaDirection;
use crate::gpu::{GP0_ADDRESS_END, GP0_ADDRESS_START, GP1_ADDRESS_END, GP1_ADDRESS_START, Gpu};
use crate::irq::{I_MASK_ADDR_END, I_MASK_ADDR_START, I_STAT_ADDR_END, I_STAT_ADDR_START, Irq};
use crate::mmu::bus::{Bus8 as _, Bus32};
use crate::mmu::dma::{Channel, DMA_INTERRUPT_REGISTER_ADDRESS_END, DMA0_ADDRESS_START, Dma, TransferMode};
use crate::spu::Spu;

pub struct Mmu {
    pub memory: Box<[u8; 0xFFFF_FFFF]>, // 512 KB BIOS
    pub cdrom: Cdrom,
    pub spu: Spu,
    pub gpu: Gpu,
    pub dma: Dma,
    pub irq: Irq,
    random_timer: u32,
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
            random_timer: 0,
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
            /*tracing::trace!(
                target: "psx_core::dma",
                addr = %format!("{:08X}", addr),
                value = %format!("{:08X}", value),
                "OTC"
            );*/
            self.write_u32(addr, value);
        };

        let mut current_addr = madr - (bcr as u32 * 4);
        write_u32(current_addr, 0xFFFFFFFF); // End marker

        for _ in 0..bcr {
            let previous_addr = current_addr;
            current_addr += 4;
            write_u32(current_addr, previous_addr);
        }
    }

    pub fn perform_gpu_dma(&mut self) {
        let channel = self.dma.channels.2;

        let mut src_addr = channel.base_address();
        let madr_step = channel.channel_control.madr_step();

        let mut transfer_words = |addr: u32, count: u32| {
            for idx in 0..count {
                let addr = addr.wrapping_add_signed(idx as i32 * madr_step);
                let word = self.read_u32(addr);
                self.write_u32(GP0_ADDRESS_START, word);
            }
        };

        match channel.channel_control.transfer_mode() {
            TransferMode::Burst => {
                let total_words = channel.bcr_word_count();
                transfer_words(src_addr, total_words);
            }
            TransferMode::Slice => {
                let total_words = channel.bcr_block_total();
                transfer_words(src_addr, total_words);
            }
            TransferMode::LinkedList => loop {
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
                for idx in 0..words {
                    let addr = src_addr.wrapping_add_signed((idx + 1) as i32 * madr_step);
                    let word = self.read_u32(addr);
                    self.write_u32(GP0_ADDRESS_START, word);

                    tracing::trace!(
                        target: "psx_core::dma",
                        address = format!("{:08X}", addr),
                        word = format!("{:08X}", word),
                        "Sent word from linked list node to GP0"
                    );
                }

                // AFTER processing this node's data, check if this was the last node
                if next_addr == 0x00FFFFFF {
                    break; // End of linked list
                }

                src_addr = next_addr;
            },
        }
    }

    pub fn perform_cdrom_dma(&mut self) {
        let channel = self.dma.channels.3;

        let dest_addr = channel.base_address();
        let madr_step = channel.channel_control.madr_step();

        tracing::debug!(
            target: "psx_core::dma",
            dest_addr = %format!("{:08X}", dest_addr),
            "CDROM DMA transfer starting"
        );

        match channel.channel_control.transfer_mode() {
            TransferMode::Burst => {
                let total_words = channel.bcr_word_count();

                for idx in 0..total_words {
                    let word = self.read_u32(CDROM_ADDR_START);
                    let addr = dest_addr.wrapping_add_signed(idx as i32 * madr_step);
                    self.write_u32(addr, word);

                    tracing::trace!(
                        target: "psx_core::dma",
                        address = format!("{:08X}", addr),
                        word = format!("{:08X}", word),
                        "CDROM DMA word transferred to RAM"
                    );
                }
            }
            _ => {
                tracing::error!(
                    target: "psx_core::dma",
                    transfer_mode = %channel.channel_control.transfer_mode(),
                    "CDROM DMA only supports Burst mode"
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
            CDROM_ADDR_START..=CDROM_ADDR_END => self.cdrom.read_u8(address),
            DMA0_ADDRESS_START..=DMA_INTERRUPT_REGISTER_ADDRESS_END => self.dma.read_u8(address),
            I_MASK_ADDR_START..=I_MASK_ADDR_END => self.irq.read_u8(address),
            I_STAT_ADDR_START..=I_STAT_ADDR_END => self.irq.read_u8(address),
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
            CDROM_ADDR_START..=CDROM_ADDR_END => self.cdrom.write_u8(address, value),
            DMA0_ADDRESS_START..=DMA_INTERRUPT_REGISTER_ADDRESS_END => self.dma.write_u8(address, value),
            I_MASK_ADDR_START..=I_MASK_ADDR_END => self.irq.write_u8(address, value),
            I_STAT_ADDR_START..=I_STAT_ADDR_END => self.irq.write_u8(address, value),
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
            DMA0_ADDRESS_START..=DMA_INTERRUPT_REGISTER_ADDRESS_END => self.dma.read_u16(address),
            I_MASK_ADDR_START..=I_MASK_ADDR_END => self.irq.read_u16(address),
            I_STAT_ADDR_START..=I_STAT_ADDR_END => self.irq.read_u16(address),
            _ => u16::from_le_bytes([self.read_u8(address), self.read_u8(address + 1)]),
        }
    }

    #[inline(always)]
    fn write_u16(&mut self, address: u32, value: u16) {
        let address = Self::canonicalize_virtual_address(address);
        match address {
            DMA0_ADDRESS_START..=DMA_INTERRUPT_REGISTER_ADDRESS_END => self.dma.write_u16(address, value),
            I_MASK_ADDR_START..=I_MASK_ADDR_END => self.irq.write_u16(address, value),
            I_STAT_ADDR_START..=I_STAT_ADDR_END => self.irq.write_u16(address, value),
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
            0x1F801100..=0x1F80112F => {
                self.random_timer = self.random_timer.wrapping_add(1);
                self.random_timer
            }
            DMA0_ADDRESS_START..=DMA_INTERRUPT_REGISTER_ADDRESS_END => self.dma.read_u32(address),
            GP0_ADDRESS_START..=GP0_ADDRESS_END => self.gpu.read_u32(address),
            GP1_ADDRESS_START..=GP1_ADDRESS_END => self.gpu.read_u32(address),
            I_STAT_ADDR_START..=I_STAT_ADDR_END => self.irq.read_u32(address),
            I_MASK_ADDR_START..=I_MASK_ADDR_END => self.irq.read_u32(address),
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
            DMA0_ADDRESS_START..=DMA_INTERRUPT_REGISTER_ADDRESS_END => self.dma.write_u32(address, value),
            GP0_ADDRESS_START..=GP0_ADDRESS_END => self.gpu.write_u32(address, value),
            GP1_ADDRESS_START..=GP1_ADDRESS_END => self.gpu.write_u32(address, value),
            I_MASK_ADDR_START..=I_MASK_ADDR_END => self.irq.write_u32(address, value),
            I_STAT_ADDR_START..=I_STAT_ADDR_END => self.irq.write_u32(address, value),
            _ => {
                self.write_u8(address, (value & 0xFF) as u8);
                self.write_u8(address + 1, ((value >> 8) & 0xFF) as u8);
                self.write_u8(address + 2, ((value >> 16) & 0xFF) as u8);
                self.write_u8(address + 3, ((value >> 24) & 0xFF) as u8);
            }
        }
    }
}
