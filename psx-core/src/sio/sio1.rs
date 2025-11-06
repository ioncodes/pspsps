use crate::sio::SerialControl;

crate::define_addr!(SIO1_TX_DATA_ADDR, 0x1F80_1040, 1, 4, 0x10);
crate::define_addr!(SIO1_RX_DATA_ADDR, 0x1F80_1040, 1, 4, 0x10);
crate::define_addr!(SIO1_STATUS_ADDR, 0x1F80_1044, 1, 4, 0x10);
crate::define_addr!(SIO1_MODE_ADDR, 0x1F80_1048, 1, 2, 0x10);
crate::define_addr!(SIO1_CTRL_ADDR, 0x1F80_104A, 1, 2, 0x10);
crate::define_addr!(SIO1_BAUD_ADDR, 0x1F80_104E, 1, 2, 0x10);

#[derive(Default)]
pub struct Sio1 {
    pub control: SerialControl,
}

impl Sio1 {
    pub fn new() -> Self {
        Self::default()
    }
}
