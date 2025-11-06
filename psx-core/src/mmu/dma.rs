use proc_bitfield::bitfield;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferMode {
    Burst,
    Slice,
    LinkedList,
}

impl TryFrom<u8> for TransferMode {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TransferMode::Burst),
            1 => Ok(TransferMode::Slice),
            2 => Ok(TransferMode::LinkedList),
            _ => Err(value),
        }
    }
}

impl From<TransferMode> for u8 {
    fn from(mode: TransferMode) -> u8 {
        mode as u8
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct ChannelControl(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub transfer_direction: bool @ 0,
        pub madr_increment_per_step: bool @ 1,
        pub idk_wtf_this_is: bool @ 8,
        pub transfer_mode: u8 [try_get TransferMode, set TransferMode] @ 9..=10,
        pub chopping_dma_window_size: u8 @ 16..=18,
        pub chopping_cpu_window_size: u8 @ 20..=22,
        pub start_transfer: bool @ 24,
        pub force_transfer_start: bool @ 28,
        pub god_knows_what_this_is: bool @ 29,
        pub bus_snooping: bool @ 30,
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

pub struct Channel {
    pub id: u8,
    pub base_address: u32,               // D#_MADR
    pub block_control: u32,              // D#_BCR
    pub channel_control: ChannelControl, // D#_CHCR
}

impl Channel {
    pub fn new(id: u8) -> Self {
        Self {
            id,
            base_address: 0,
            block_control: 0,
            channel_control: ChannelControl(0),
        }
    }

    pub fn base_address(&self) -> u32 {
        // 0-23  Memory Address where the DMA will start reading from/writing to
        // 24-31 Not used (always zero)
        self.base_address & 0x00FF_FFFF
    }

    pub fn block_control(&self) -> u32 {
        // For SyncMode=0 (ie. for OTC and CDROM):
        //   0-15  BC    Number of words (0001h..FFFFh) (or 0=10000h words)
        //   16-31 0     Not used (usually 0 for OTC, or 1 ("one block") for CDROM)
        // For SyncMode=1 (ie. for MDEC, SPU, and GPU-vram-data):
        //   0-15  BS    Blocksize (words) ;for GPU/SPU max 10h, for MDEC max 20h
        //   16-31 BA    Amount of blocks  ;ie. total length = BS*BA words
        // For SyncMode=2 (ie. for GPU-command-lists):
        //   0-31  0     Not used (should be zero) (transfer ends at END-CODE in list)

        if let Ok(transfer_mode) = self.channel_control.transfer_mode() {
            return match transfer_mode {
                TransferMode::Burst => 0x69,
                TransferMode::Slice => 0x69,
                TransferMode::LinkedList => 0,
            };
        }

        tracing::error!(target: "psx_core::dma", channel_id = self.id, "Failed to get transfer mode");
        0
    }
}

pub struct Dma {
    pub channels: [Channel; 7],
    pub control: ControlRegister,
    pub interrupt: InterruptRegister,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            channels: [
                Channel::new(0), // MDECin (RAM to MDEC)
                Channel::new(1), // MDECout (MDEC to RAM)
                Channel::new(2), // GPU (lists + image data)
                Channel::new(3), // CDROM (CDROM to RAM)
                Channel::new(4), // SPU
                Channel::new(5), // PIO
                Channel::new(6), // OTC
            ],
            control: ControlRegister(0),
            interrupt: InterruptRegister(0),
        }
    }
}
