use psx_core::mmu::dma::TransferMode;

#[derive(Clone)]
pub struct DmaChannelState {
    pub active: bool,
    pub transfer_direction: bool, // false = Device to RAM, true = RAM to Device
    pub transfer_mode: TransferMode,
}

impl Default for DmaChannelState {
    fn default() -> Self {
        Self {
            active: false,
            transfer_direction: false,
            transfer_mode: TransferMode::Burst,
        }
    }
}

#[derive(Clone, Default)]
pub struct DmaState {
    pub channel0: DmaChannelState, // MDECin
    pub channel1: DmaChannelState, // MDECout
    pub channel2: DmaChannelState, // GPU
    pub channel3: DmaChannelState, // CDROM
    pub channel4: DmaChannelState, // SPU
    pub channel5: DmaChannelState, // PIO
    pub channel6: DmaChannelState, // OTC
}
