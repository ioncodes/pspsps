use proc_bitfield::bitfield;

use crate::mmu::bus::{Bus8, Bus16, Bus32};

crate::define_addr!(DMA0_ADDRESS, 0x1F80_1080, 0, 0xB, 0x10);
crate::define_addr!(DMA1_ADDRESS, 0x1F80_1080, 1, 0xB, 0x10);
crate::define_addr!(DMA2_ADDRESS, 0x1F80_1080, 2, 0xB, 0x10);
crate::define_addr!(DMA3_ADDRESS, 0x1F80_1080, 3, 0xB, 0x10);
crate::define_addr!(DMA4_ADDRESS, 0x1F80_1080, 4, 0xB, 0x10);
crate::define_addr!(DMA5_ADDRESS, 0x1F80_1080, 5, 0xB, 0x10);
crate::define_addr!(DMA6_ADDRESS, 0x1F80_1080, 6, 0xB, 0x10);
crate::define_addr!(DMA_CONTROL_REGISTER_ADDRESS, 0x1F80_10F0, 0, 0x4, 0x4);
crate::define_addr!(DMA_INTERRUPT_REGISTER_ADDRESS, 0x1F80_10F4, 0, 0x4, 0x4);

pub const MDEC_IN_CHANNEL_ID: u8 = 0;
pub const MDEC_OUT_CHANNEL_ID: u8 = 1;
pub const GPU_CHANNEL_ID: u8 = 2;
pub const CDROM_CHANNEL_ID: u8 = 3;
pub const SPU_CHANNEL_ID: u8 = 4;
pub const PIO_CHANNEL_ID: u8 = 5;
pub const OTC_CHANNEL_ID: u8 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferMode {
    Burst,
    Slice,
    LinkedList,
}

impl From<u8> for TransferMode {
    fn from(value: u8) -> Self {
        match value {
            0 => TransferMode::Burst,
            1 => TransferMode::Slice,
            2 => TransferMode::LinkedList,
            _ => unreachable!(),
        }
    }
}

impl From<TransferMode> for u8 {
    fn from(mode: TransferMode) -> u8 {
        mode as u8
    }
}

impl std::fmt::Display for TransferMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferMode::Burst => write!(f, "Burst"),
            TransferMode::Slice => write!(f, "Slice"),
            TransferMode::LinkedList => write!(f, "LinkedList"),
        }
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct ChannelControl(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub transfer_direction: bool @ 0,
        pub madr_increment_per_step: bool @ 1,
        pub idk_wtf_this_is: bool @ 8,
        pub transfer_mode: u8 [get TransferMode, set TransferMode] @ 9..=10,
        pub chopping_dma_window_size: u8 @ 16..=18,
        pub chopping_cpu_window_size: u8 @ 20..=22,
        pub start_transfer: bool @ 24,
        pub force_transfer_start: bool @ 28,
        pub god_knows_what_this_is: bool @ 29,
        pub bus_snooping: bool @ 30,
    }
}

impl ChannelControl {
    pub fn madr_step(&self) -> i32 {
        if self.madr_increment_per_step() {
            -4
        } else {
            4
        }
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct ControlRegister(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub dma0_priority: u8 @ 0..=2,
        pub dma0_master: bool @ 3,
        pub dma1_priority: u8 @ 4..=6,
        pub dma1_master: bool @ 7,
        pub dma2_priority: u8 @ 8..=10,
        pub dma2_master: bool @ 11,
        pub dma3_priority: u8 @ 12..=14,
        pub dma3_master: bool @ 15,
        pub dma4_priority: u8 @ 16..=18,
        pub dma4_master: bool @ 19,
        pub dma5_priority: u8 @ 20..=22,
        pub dma5_master: bool @ 23,
        pub dma6_priority: u8 @ 24..=26,
        pub dma6_master: bool @ 27
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct InterruptRegister(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub completion: u8 @ 0..=6,
        pub bus_error: bool @ 15,
        pub interrupt_mask: u8 @ 16..=22,
        pub master_interrupt_enable: bool @ 23,
        pub interrupt_flags: u8 @ 24..=30,
        pub master_interrupt: bool @ 31
    }
}

#[derive(Clone, Copy)]
pub struct Channel<const CHANNEL_ID: u8> {
    pub base_address: u32,               // D#_MADR
    pub block_control: u32,              // D#_BCR
    pub channel_control: ChannelControl, // D#_CHCR
}

impl<const CHANNEL_ID: u8> Channel<CHANNEL_ID> {
    pub fn new() -> Self {
        Self {
            base_address: 0,
            block_control: 0,
            channel_control: ChannelControl(0),
        }
    }

    /// D#_MADR
    #[inline(always)]
    pub fn base_address(&self) -> u32 {
        // 0-23  Memory Address where the DMA will start reading from/writing to
        // 24-31 Not used (always zero)
        self.base_address & 0x00FF_FFFF
    }

    /// D#_BCR
    #[inline(always)]
    pub fn block_control(&self) -> u32 {
        // For SyncMode=0 (ie. for OTC and CDROM):
        //   0-15  BC    Number of words (0001h..FFFFh) (or 0=10000h words)
        //   16-31 0     Not used (usually 0 for OTC, or 1 ("one block") for CDROM)
        // For SyncMode=1 (ie. for MDEC, SPU, and GPU-vram-data):
        //   0-15  BS    Blocksize (words) ;for GPU/SPU max 10h, for MDEC max 20h
        //   16-31 BA    Amount of blocks  ;ie. total length = BS*BA words
        // For SyncMode=2 (ie. for GPU-command-lists):
        //   0-31  0     Not used (should be zero) (transfer ends at END-CODE in list)

        match self.channel_control.transfer_mode() {
            TransferMode::Burst => self.block_control,
            TransferMode::Slice => self.block_control,
            TransferMode::LinkedList => 0,
        }
    }

    /// BC - SyncMode=0 only
    #[inline(always)]
    pub fn bcr_word_count(&self) -> u32 {
        if CHANNEL_ID != OTC_CHANNEL_ID && CHANNEL_ID != CDROM_CHANNEL_ID {
            tracing::warn!(target: "psx_core::dma", channel_id = CHANNEL_ID, "Accessing BC register in non-OTC or CDROM channel");
        }

        if self.channel_control.transfer_mode() != TransferMode::Burst {
            tracing::warn!(target: "psx_core::dma", channel_id = CHANNEL_ID, "Accessing BC register in non-Burst mode");
        }

        let words = self.block_control & 0xFFFF;
        if words == 0 { 0x10000 } else { words }
    }

    /// BS - SyncMode=1 only
    #[inline(always)]
    pub fn bcr_block_size(&self) -> u32 {
        if self.channel_control.transfer_mode() != TransferMode::Slice {
            tracing::warn!(target: "psx_core::dma", channel_id = CHANNEL_ID, "Accessing BS register in non-Slice mode");
        }

        self.block_control & 0xFFFF
    }

    /// BA - SyncMode=1 only
    #[inline(always)]
    pub fn bcr_block_count(&self) -> u32 {
        if self.channel_control.transfer_mode() != TransferMode::Slice {
            tracing::warn!(target: "psx_core::dma", channel_id = CHANNEL_ID, "Accessing BA register in non-Slice mode");
        }

        (self.block_control & 0xFFFF_0000) >> 16
    }

    #[inline(always)]
    pub fn bcr_block_total(&self) -> u32 {
        self.bcr_block_size() * self.bcr_block_count()
    }

    #[inline(always)]
    pub fn set_completed(&mut self) {
        self.channel_control.set_start_transfer(false);
    }
}

impl<const CHANNEL_ID: u8> Bus8 for Channel<CHANNEL_ID> {
    #[inline(always)]
    fn read_u8(&mut self, address: u32) -> u8 {
        let offset = address & 0b11;
        match address & 0xF {
            0x0..=0x03 => (self.base_address >> (offset * 8)) as u8,
            0x4..=0x07 => (self.block_control >> (offset * 8)) as u8,
            0x8..=0x0B => (self.channel_control.0 >> (offset * 8)) as u8,
            _ => unreachable!(),
        }
    }

    fn write_u8(&mut self, _address: u32, _value: u8) {
        unreachable!()
    }
}

impl<const CHANNEL_ID: u8> Bus16 for Channel<CHANNEL_ID> {
    #[inline(always)]
    fn read_u16(&mut self, address: u32) -> u16 {
        let lo = self.read_u8(address) as u16;
        let hi = self.read_u8(address + 1) as u16;
        lo | (hi << 8)
    }

    fn write_u16(&mut self, _address: u32, _value: u16) {
        unreachable!()
    }
}

impl<const CHANNEL_ID: u8> Bus32 for Channel<CHANNEL_ID> {
    #[inline(always)]
    fn read_u32(&mut self, address: u32) -> u32 {
        match address & 0xF {
            0x0 => self.base_address,
            0x4 => self.block_control,
            0x8 => self.channel_control.0,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    fn write_u32(&mut self, address: u32, value: u32) {
        match address & 0xF {
            0x0 => self.base_address = value,
            0x4 => self.block_control = value,
            0x8 => self.channel_control.0 = value,
            _ => unreachable!(),
        }
    }
}

pub struct Dma {
    pub channels: (
        Channel<0>,
        Channel<1>,
        Channel<2>,
        Channel<3>,
        Channel<4>,
        Channel<5>,
        Channel<6>,
    ),
    pub control: ControlRegister,
    pub interrupt: InterruptRegister,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            channels: (
                Channel::<0>::new(), // MDECin (RAM to MDEC)
                Channel::<1>::new(), // MDECout (MDEC to RAM)
                Channel::<2>::new(), // GPU (lists + image data)
                Channel::<3>::new(), // CDROM (CDROM to RAM)
                Channel::<4>::new(), // SPU
                Channel::<5>::new(), // PIO
                Channel::<6>::new(), // OTC
            ),
            control: ControlRegister(0),
            interrupt: InterruptRegister(0),
        }
    }
}

impl Bus8 for Dma {
    fn read_u8(&mut self, address: u32) -> u8 {
        match address {
            DMA0_ADDRESS_START..=DMA0_ADDRESS_END => self.channels.0.read_u8(address),
            DMA1_ADDRESS_START..=DMA1_ADDRESS_END => self.channels.1.read_u8(address),
            DMA2_ADDRESS_START..=DMA2_ADDRESS_END => self.channels.2.read_u8(address),
            DMA3_ADDRESS_START..=DMA3_ADDRESS_END => self.channels.3.read_u8(address),
            DMA4_ADDRESS_START..=DMA4_ADDRESS_END => self.channels.4.read_u8(address),
            DMA5_ADDRESS_START..=DMA5_ADDRESS_END => self.channels.5.read_u8(address),
            DMA6_ADDRESS_START..=DMA6_ADDRESS_END => self.channels.6.read_u8(address),
            DMA_CONTROL_REGISTER_ADDRESS_START..=DMA_CONTROL_REGISTER_ADDRESS_END => {
                let offset = address & 0b11;
                (self.control.0 >> (offset * 8)) as u8
            }
            DMA_INTERRUPT_REGISTER_ADDRESS_START..=DMA_INTERRUPT_REGISTER_ADDRESS_END => {
                let offset = address & 0b11;
                (self.interrupt.0 >> (offset * 8)) as u8
            }
            _ => unreachable!(),
        }
    }

    #[tracing::instrument(
        target = "psx_core::dma",
        level = "warn",
        skip(self),
        fields(address = %format!("{:08X}", address), value = %format!("{:04X}", value))
    )]
    #[inline(always)]
    fn write_u8(&mut self, address: u32, value: u8) {
        // https://psx-spx.consoledev.net/unpredictablethings/
        self.write_u32(address, value as u32);
    }
}

impl Bus16 for Dma {
    fn read_u16(&mut self, address: u32) -> u16 {
        match address {
            DMA0_ADDRESS_START..=DMA0_ADDRESS_END => self.channels.0.read_u16(address),
            DMA1_ADDRESS_START..=DMA1_ADDRESS_END => self.channels.1.read_u16(address),
            DMA2_ADDRESS_START..=DMA2_ADDRESS_END => self.channels.2.read_u16(address),
            DMA3_ADDRESS_START..=DMA3_ADDRESS_END => self.channels.3.read_u16(address),
            DMA4_ADDRESS_START..=DMA4_ADDRESS_END => self.channels.4.read_u16(address),
            DMA5_ADDRESS_START..=DMA5_ADDRESS_END => self.channels.5.read_u16(address),
            DMA6_ADDRESS_START..=DMA6_ADDRESS_END => self.channels.6.read_u16(address),
            DMA_CONTROL_REGISTER_ADDRESS_START..=DMA_CONTROL_REGISTER_ADDRESS_END => {
                let lo = self.read_u8(address) as u16;
                let hi = self.read_u8(address + 1) as u16;
                lo | (hi << 8)
            }
            DMA_INTERRUPT_REGISTER_ADDRESS_START..=DMA_INTERRUPT_REGISTER_ADDRESS_END => {
                let lo = self.read_u8(address) as u16;
                let hi = self.read_u8(address + 1) as u16;
                lo | (hi << 8)
            }
            _ => unreachable!(),
        }
    }

    #[tracing::instrument(
        target = "psx_core::dma",
        level = "warn",
        skip(self),
        fields(address = %format!("{:08X}", address), value = %format!("{:04X}", value))
    )]
    #[inline(always)]
    fn write_u16(&mut self, address: u32, value: u16) {
        // https://psx-spx.consoledev.net/unpredictablethings/
        self.write_u32(address, value as u32);
    }
}

impl Bus32 for Dma {
    fn read_u32(&mut self, address: u32) -> u32 {
        match address {
            DMA0_ADDRESS_START..=DMA0_ADDRESS_END => self.channels.0.read_u32(address),
            DMA1_ADDRESS_START..=DMA1_ADDRESS_END => self.channels.1.read_u32(address),
            DMA2_ADDRESS_START..=DMA2_ADDRESS_END => self.channels.2.read_u32(address),
            DMA3_ADDRESS_START..=DMA3_ADDRESS_END => self.channels.3.read_u32(address),
            DMA4_ADDRESS_START..=DMA4_ADDRESS_END => self.channels.4.read_u32(address),
            DMA5_ADDRESS_START..=DMA5_ADDRESS_END => self.channels.5.read_u32(address),
            DMA6_ADDRESS_START..=DMA6_ADDRESS_END => self.channels.6.read_u32(address),
            DMA_CONTROL_REGISTER_ADDRESS_START => self.control.0,
            DMA_INTERRUPT_REGISTER_ADDRESS_START => self.interrupt.0,
            _ => unreachable!(),
        }
    }

    fn write_u32(&mut self, address: u32, value: u32) {
        match address {
            DMA0_ADDRESS_START..=DMA0_ADDRESS_END => self.channels.0.write_u32(address, value),
            DMA1_ADDRESS_START..=DMA1_ADDRESS_END => self.channels.1.write_u32(address, value),
            DMA2_ADDRESS_START..=DMA2_ADDRESS_END => self.channels.2.write_u32(address, value),
            DMA3_ADDRESS_START..=DMA3_ADDRESS_END => self.channels.3.write_u32(address, value),
            DMA4_ADDRESS_START..=DMA4_ADDRESS_END => self.channels.4.write_u32(address, value),
            DMA5_ADDRESS_START..=DMA5_ADDRESS_END => self.channels.5.write_u32(address, value),
            DMA6_ADDRESS_START..=DMA6_ADDRESS_END => self.channels.6.write_u32(address, value),
            DMA_CONTROL_REGISTER_ADDRESS_START => self.control.0 = value,
            DMA_INTERRUPT_REGISTER_ADDRESS_START => self.interrupt.0 = value,
            _ => unreachable!(),
        }
    }
}
