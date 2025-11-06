pub mod cop;
pub mod decoder;
pub mod handlers;
pub mod internal;
pub mod lut;

use crate::cpu::cop::cop0::{Cop0, Exception};
use crate::cpu::decoder::Instruction;
use crate::mmu::Mmu;
use crate::mmu::bus::{Bus8 as _, Bus16 as _, Bus32 as _};

pub struct Cpu {
    pub pc: u32,
    pub registers: [u32; 32],
    pub hi: u32,
    pub lo: u32,
    pub load_delay: Option<(usize, u32)>, // Loads are delayed by one instruction
    pub load_delay_pending: bool,         // True if load_delay was just scheduled this instruction
    pub delay_slot: Option<(Instruction, u32)>, // Delay slot (instruction, branch destination)
    pub cop0: Cop0,                       // COP0 registers
    pub mmu: Mmu,
    pub cycles: usize,      // Number of cycles executed
    exception_raised: bool, // Indicates if an exception has been raised
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            pc: 0,
            registers: [0; 32],
            hi: 0,
            lo: 0,
            load_delay: None,
            load_delay_pending: false,
            delay_slot: None,
            cop0: Cop0::new(),
            mmu: Mmu::new(),
            exception_raised: false,
            cycles: 0,
        }
    }

    pub fn tick(&mut self) -> Result<Instruction, ()> {
        if let Some(handler) = internal::cpu_hooks().get(&self.pc) {
            handler(self);
        }

        if let Some((mut delay_slot, branch_target)) = self.delay_slot.take() {
            if delay_slot.is_invalid() {
                tracing::error!(target: "psx_core::cpu", pc = %format!("{:08X}", self.pc), "Invalid instruction in delay slot");
                return Err(());
            }

            tracing::trace!(
                target: "psx_core::cpu",
                instruction = %format!("{}", delay_slot),
                brach_target = %format!("{:08X}", branch_target),
                "Executing delay slot instruction"
            );

            delay_slot.is_delay_slot = true; // Mark as a delay slot instruction
            (delay_slot.handler)(&delay_slot, self);

            // Process pending load, or mark it for the next instruction
            self.process_pending_load();

            self.pc = branch_target; // Set PC to the scheduled branch address

            return Ok(delay_slot);
        }

        let instr = Instruction::decode(self.mmu.read_u32(self.pc));
        if instr.is_invalid() {
            tracing::error!(target: "psx_core::cpu", pc = %format!("{:08X}", self.pc), "Invalid instruction");
            return Err(());
        }

        tracing::trace!(target: "psx_core::cpu", "{:08X}: [{:08X}] {: <30}", self.pc, instr.raw, format!("{}", instr));

        (instr.handler)(&instr, self);

        // Process pending load, or mark it for the next instruction
        self.process_pending_load();

        // Do not progress PC if we just raised an exception
        if self.exception_raised {
            self.exception_raised = false;
        } else {
            self.pc = self.pc.wrapping_add(4);
        }

        Ok(instr)
    }

    pub fn cause_exception(&mut self, exception: Exception, is_delay_slot: bool) {
        tracing::debug!(target: "psx_core::cpu", "Exception occurred: {}", &exception);

        self.exception_raised = true;

        // Set the EPC to the current PC or PC - 4 if in a branch delay slot
        self.cop0.cause.set_branch_delay(is_delay_slot);
        self.cop0.cause.set_branch_taken(false); // TODO: do we have to emulate this? it's easy we can just set a flag in handlers

        self.cop0.epc = if !is_delay_slot { self.pc } else { self.pc - 4 };
        self.cop0.cause.set_exception_code(exception as u32); // Set the exception code

        // Back up current state/bits
        self.cop0
            .sr
            .set_old_interrupt_enable(self.cop0.sr.previous_interrupt_enable());
        self.cop0.sr.set_old_mode(self.cop0.sr.previous_mode());
        self.cop0
            .sr
            .set_previous_interrupt_enable(self.cop0.sr.current_interrupt_enable());
        self.cop0.sr.set_previous_mode(self.cop0.sr.current_mode());

        // Disable interrupts and enter kernel mode
        self.cop0.sr.set_current_interrupt_enable(false);
        self.cop0.sr.set_current_mode(false); // Enter kernel mode

        //   Exception     BEV=0         BEV=1
        //   Reset         BFC00000h     BFC00000h   (Reset)
        //   UTLB Miss     80000000h     BFC00100h   (Virtual memory, none such in PSX)
        //   COP0 Break    80000040h     BFC00140h   (Debug Break)
        //   General       80000080h     BFC00180h   (General Interrupts & Exceptions)
        if !self.cop0.sr.boot_exception_vector_location() {
            self.pc = 0x8000_0080;
        } else {
            self.pc = 0xBFC0_0180;
        }
    }

    pub fn restore_from_exception(&mut self) {
        tracing::debug!(target: "psx_core::cpu", "Returning from exception");

        // Restore pre-exception state
        self.cop0
            .sr
            .set_current_interrupt_enable(self.cop0.sr.previous_interrupt_enable());
        self.cop0.sr.set_current_mode(self.cop0.sr.previous_mode());
        self.cop0
            .sr
            .set_previous_interrupt_enable(self.cop0.sr.old_interrupt_enable());
        self.cop0.sr.set_previous_mode(self.cop0.sr.old_mode());
    }

    pub fn write_u8(&mut self, address: u32, value: u8) {
        if self.cop0.sr.isolate_cache() {
            return;
        }

        self.mmu.write_u8(address, value);
    }

    pub fn write_u16(&mut self, address: u32, value: u16) {
        if self.cop0.sr.isolate_cache() {
            return;
        }

        self.mmu.write_u16(address, value);
    }

    pub fn write_u32(&mut self, address: u32, value: u32) {
        if self.cop0.sr.isolate_cache() {
            return;
        }

        self.mmu.write_u32(address, value);
    }

    pub fn read_u8(&mut self, address: u32) -> u8 {
        self.mmu.read_u8(address)
    }

    pub fn read_u16(&mut self, address: u32) -> u16 {
        self.mmu.read_u16(address)
    }

    pub fn read_u32(&mut self, address: u32) -> u32 {
        self.mmu.read_u32(address)
    }

    #[inline(always)]
    pub fn write_register(&mut self, index: u8, value: u32) {
        if index != 0 {
            if let Some((load_reg, _)) = self.load_delay
                && load_reg == index as usize
            {
                self.load_delay = None;
                self.load_delay_pending = false;
            }

            self.registers[index as usize] = value;
        }
    }

    #[inline(always)]
    pub fn read_register(&self, index: u8) -> u32 {
        self.registers[index as usize]
    }

    #[inline(always)]
    pub fn schedule_load(&mut self, index: u8, value: u32) {
        if index != 0 {
            // If there's a non-pending load to a DIFFERENT register, apply it now
            // If it's to the SAME register, it gets cancelled (overwritten)
            if !self.load_delay_pending
                && let Some((reg_idx, val)) = self.load_delay.take()
                && reg_idx != index as usize
            {
                self.registers[reg_idx] = val;
            }

            self.load_delay = Some((index as usize, value));
            self.load_delay_pending = true; // Mark as freshly scheduled
        }
    }

    #[inline(always)]
    pub fn read_register_with_pending_load(&self, index: u8) -> u32 {
        // Read a register, including any pending load delay value
        // This is used by LWL/LWR instructions which can read pending loads

        if let Some((load_reg, load_value)) = self.load_delay
            && load_reg == index as usize
        {
            return load_value;
        }

        self.registers[index as usize]
    }

    #[inline(always)]
    pub(crate) fn add_cycles(&mut self, cycles: usize) {
        self.cycles += cycles;
    }

    #[inline(always)]
    pub(crate) fn drain_cycles(&mut self) -> usize {
        let cycles = self.cycles;
        self.cycles = 0;
        cycles
    }

    #[inline(always)]
    pub(crate) fn set_delay_slot(&mut self, branch_target: u32) {
        // Load the next instruction into the delay slot
        // Also cache the branch target
        self.delay_slot = Some((
            Instruction::decode(self.mmu.read_u32(self.pc + 4)),
            branch_target,
        ));
    }

    #[inline(always)]
    fn process_pending_load(&mut self) {
        if !self.load_delay_pending
            && let Some((reg_idx, value)) = self.load_delay.take()
        {
            self.registers[reg_idx] = value;
        } else {
            self.load_delay_pending = false;
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
