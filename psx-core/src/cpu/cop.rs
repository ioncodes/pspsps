pub mod cop0;
pub mod registers;
pub mod cop2;

pub trait Cop {
    fn read_register(&self, register: u8) -> u32;
    fn write_register(&mut self, register: u8, value: u32);
}
