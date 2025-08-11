pub mod decoder;
pub mod handlers;
mod lut;

#[derive(Debug, Clone)]
pub struct Cpu {
    pub pc: u32,
    pub registers: [u32; 32],
    pub hi: u32,
    pub lo: u32,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            pc: 0,
            registers: [0; 32],
            hi: 0,
            lo: 0,
        }
    }
}

impl std::fmt::Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "R0:{:08X} R1: {:08X} R2:{:08X} R3:{:08X} R4:{:08X} R5:{:08X} R6:{:08X} R7:{:08X} R8:{:08X} R9:{:08X} R10:{:08X} R11:{:08X} R12:{:08X} R13:{:08X} R14:{:08X} R15:{:08X} R16:{:08X} R17:{:08X} R18:{:08X} R19:{:08X} R20:{:08X} R21:{:08X} R22:{:08X} R23:{:08X} R24:{:08X} R25:{:08X} R26:{:08X} R27:{:08X} R28:{:08X} R29:{:08X} R30:{:08X} R31:{:08X} PC:{:08X} HI:{:08X} LO:{:08X}",
            self.pc,
            self.hi,
            self.lo,
            self.registers[0],
            self.registers[1],
            self.registers[2],
            self.registers[3],
            self.registers[4],
            self.registers[5],
            self.registers[6],
            self.registers[7],
            self.registers[8],
            self.registers[9],
            self.registers[10],
            self.registers[11],
            self.registers[12],
            self.registers[13],
            self.registers[14],
            self.registers[15],
            self.registers[16],
            self.registers[17],
            self.registers[18],
            self.registers[19],
            self.registers[20],
            self.registers[21],
            self.registers[22],
            self.registers[23],
            self.registers[24],
            self.registers[25],
            self.registers[26],
            self.registers[27],
            self.registers[28],
            self.registers[29],
            self.registers[30],
            self.registers[31]
        )
    }
}
