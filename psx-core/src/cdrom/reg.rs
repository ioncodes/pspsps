use proc_bitfield::bitfield;

use crate::cdrom::irq::DiskIrq;

pub const REG_ADDRESS_ADDR: u32 = 0x1F80_1800;
pub const REG_COMMAND_ADDR: u32 = 0x1F80_1801;
pub const REG_WRDATA_ADDR: u32 = 0x1F80_1801;
pub const REG_CI_ADDR: u32 = 0x1F80_1801;
pub const REG_ATV2_ADDR: u32 = 0x1F80_1801;
pub const REG_PARAMETER_ADDR: u32 = 0x1F80_1802;
pub const REG_HINTMSK_ADDR_W: u32 = 0x1F80_1802;
pub const REG_ATV0_ADDR: u32 = 0x1F80_1802;
pub const REG_ATV3_ADDR: u32 = 0x1F80_1802;
pub const REG_HCHPCTL_ADDR: u32 = 0x1F80_1803;
pub const REG_HCLRCTL_ADDR: u32 = 0x1F80_1803;
pub const REG_ATV1_ADDR: u32 = 0x1F80_1803;
pub const REG_ADPCTL_ADDR: u32 = 0x1F80_1803;

pub const REG_HSTS_ADDR: u32 = 0x1F80_1800;
pub const REG_RESULT_ADDR: u32 = 0x1F80_1801;
pub const REG_RDDATA_ADDR: u32 = 0x1F80_1802;
pub const REG_HINTMSK_ADDR_R: u32 = 0x1F80_1803;
pub const REG_HINTSTS_ADDR: u32 = 0x1F80_1803;

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct AddressRegister(pub u8): Debug, FromStorage, IntoStorage, DerefStorage {
        pub current_bank: u8 @ 0..=1,
        pub adpcm_busy: bool @ 2,
        pub parameter_empty: bool @ 3,
        pub parameter_write_ready: bool @ 4,
        pub result_read_ready: bool @ 5,
        pub data_request: bool @ 6,
        pub busy_status: bool @ 7,
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct AdpCtlRegister(pub u8): Debug, FromStorage, IntoStorage, DerefStorage {
        pub mute_xa_adcp: bool @ 0,
        pub apply_atv_changes: bool @ 5,
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct HIntMaskRegister(pub u8): Debug, FromStorage, IntoStorage, DerefStorage {
        pub enable_irq_on_intsts: u8 @ 0..=2,
        pub enable_irq_on_bfempt: bool @ 3,
        pub enable_irq_on_bfwrdy: bool @ 4,
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct HClrCtl(pub u8): Debug, FromStorage, IntoStorage, DerefStorage {
        pub ack_irq_flags: u8 @ 0..=2,
        pub ack_bfempt_interrupt: bool @ 3,
        pub ack_bfwrdy_interrupt: bool @ 4,
        pub clear_sound_mapper: bool @ 5,
        pub clear_parameter_fifo: bool @ 6,
        pub reset_decoder_chip: bool @ 7,
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct HIntSts(pub u8): Debug, FromStorage, IntoStorage, DerefStorage {
        pub irq_flags: u8 [get DiskIrq, set DiskIrq] @ 0..=2,
        pub sound_map_buffer_empty: bool @ 3,
        pub sound_map_buffer_write_ready: bool @ 4,
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct HChpCtl(pub u8): Debug, FromStorage, IntoStorage, DerefStorage {
        pub sound_map_enable: bool @ 5,
        pub request_sector_buffer_write: bool @ 6,
        pub request_sector_buffer_read: bool @ 7,
    }
}


bitfield! {
     #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct StatusCode(pub u8): Debug, FromStorage, IntoStorage, DerefStorage {
        pub error: bool @ 0,
        pub spindle_motor: bool @ 1,
        pub seek_error: bool @ 2,
        pub id_error: bool @ 3,
        pub shell_open: bool @ 4,
        pub read: bool @ 5,
        pub seek: bool @ 6,
        pub play: bool @ 7,
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct SetModeRegister(pub u8): Debug, FromStorage, IntoStorage, DerefStorage {
        pub cdda: bool @ 0,
        pub autopause: bool @ 1,
        pub report: bool @ 2,
        pub xa_filter: bool @ 3,
        pub ignore_bit: bool @ 4,
        pub sector_size: bool @ 5,
        pub xa_adpcm: bool @ 6,
        pub double_speed: bool @ 7,
    }
}