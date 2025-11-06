pub mod cop;
pub mod decoder;
pub mod handlers;
pub mod lut;

use colored::Colorize;

use crate::cpu::decoder::Instruction;
use crate::mmu::Mmu;

type RegisterValue = u32;
type RegisterIndex = usize;

pub struct Cpu {
    pub pc: u32,
    pub registers: [RegisterValue; 32],
    pub hi: u32,
    pub lo: u32,
    pub load_delay: Option<(RegisterIndex, RegisterValue)>, // Loads are delayed by one instruction
    pub delay_slot: Option<(Instruction, u32)>, // Delay slot (instruction, branch destination)
    pub cop0: [RegisterValue; 64],              // COP0 registers
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            pc: 0,
            registers: [0; 32],
            hi: 0,
            lo: 0,
            load_delay: None,
            delay_slot: None,
            cop0: [0; 64],
        }
    }

    #[inline(always)]
    pub fn tick(&mut self, mmu: &mut Mmu) {
        if let Some((delay_slot, branch_target)) = self.delay_slot.take() {
            tracing::debug!(
                target: "psx_core::cpu", 
                "{}", 
                format!("Executing delay slot instruction: {}, with branch target: {:08X}", delay_slot, branch_target).yellow());
            (delay_slot.handler)(&delay_slot, self, mmu);
            self.pc = branch_target; // Set PC to the scheduled branch address
            return;
        }

        let instr = Instruction::decode(mmu.read_u32(self.pc));

        tracing::debug!(target: "psx_core::cpu", "{:08X}: [{:08X}] {: <30}", self.pc, instr.raw, format!("{}", instr));
        tracing::trace!(target: "psx_core::cpu", "{:?}", instr);

        (instr.handler)(&instr, self, mmu);

        self.pc += 4;
    }

    #[inline(always)]
    pub(crate) fn set_delay_slot(&mut self, mmu: &mut Mmu, branch_target: u32) {
        // Load the next instruction into the delay slot
        // Also cache the branch target
        self.delay_slot = Some((
            Instruction::decode(mmu.read_u32(self.pc + 4)),
            branch_target,
        ));
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
