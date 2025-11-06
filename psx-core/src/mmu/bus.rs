pub trait Bus8 {
    fn read_u8(&mut self, address: u32) -> u8;
    fn write_u8(&mut self, address: u32, value: u8);
}

pub trait Bus16 {
    fn read_u16(&mut self, address: u32) -> u16;
    fn write_u16(&mut self, address: u32, value: u16);
}

pub trait Bus32 {
    fn read_u32(&mut self, address: u32) -> u32;
    fn write_u32(&mut self, address: u32, value: u32);
}
