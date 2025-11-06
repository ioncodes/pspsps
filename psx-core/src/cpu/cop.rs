pub mod cop0;
pub mod registers;
pub mod cop2;

pub trait Cop {
    fn read_register(&self, register: u32) -> u32;
    fn write_register(&mut self, register: u32, value: u32);
}

// pub struct Cop<Registers: CoProcessor> {
//     pub registers: Registers,
// }
