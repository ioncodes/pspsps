pub mod irq;
pub mod reg;

use proc_bitfield::with_bits;
use crate::cdrom::irq::DiskIrq;
use crate::cdrom::reg::{
    AddressRegister, AdpCtlRegister, HClrCtl, HIntMaskRegister, HIntSts, StatusCode, REG_ADDRESS_ADDR, REG_ADPCTL_ADDR, REG_COMMAND_ADDR, REG_HCLRCTL_ADDR, REG_HINTMSK_ADDR_R, REG_HINTMSK_ADDR_W, REG_HINTSTS_ADDR, REG_HSTS_ADDR, REG_PARAMETER_ADDR, REG_RESULT_ADDR
};
use crate::mmu::bus::Bus8;
use std::collections::VecDeque;

crate::define_addr!(CDROM_ADDR, 0x1F80_1800, 0, 0x04, 0x04);

pub const CYCLE_DELAY: usize = 1000; // TODO: just a random stub

pub struct Cdrom {
    cdrom_cue: Vec<u8>,
    cdrom_bin: Vec<u8>,
    address: AddressRegister,
    adpctl: AdpCtlRegister,
    hintmsk: HIntMaskRegister,
    hclrctl: HClrCtl,
    hintsts: HIntSts,
    parameter_fifo: VecDeque<u8>,
    result_fifo: VecDeque<u8>,
    pending_command: Option<u8>,
    cycles: usize,
}

impl Cdrom {
    pub fn new() -> Self {
        Self {
            cdrom_cue: Vec::new(),
            cdrom_bin: Vec::new(),
            address: AddressRegister(0),
            adpctl: AdpCtlRegister(0),
            hintmsk: HIntMaskRegister(0),
            hclrctl: HClrCtl(0),
            hintsts: HIntSts(0),
            parameter_fifo: VecDeque::new(),
            result_fifo: VecDeque::new(),
            pending_command: None,
            cycles: 0,
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.address.set_parameter_empty(self.parameter_fifo.is_empty());
        self.address.set_parameter_write_ready(self.parameter_fifo.len() < 16);
        self.cycles += cycles;

        if self.cycles >= CYCLE_DELAY {
            self.cycles -= CYCLE_DELAY;

            if let Some(command) = self.pending_command.take() {
                tracing::debug!(
                    target: "psx_core::cdrom",
                    command = format!("{:02X}", command),
                    "CDROM command completed",
                );

                self.parameter_fifo.clear();
                self.address.set_busy_status(false);
                self.address.set_data_request(true);
                self.address.set_result_read_ready(true);

                self.trigger_irq(DiskIrq::CommandCompleted);
            }
        }
    }

    // TODO: interrupts can be queued??
    // "The response interrupts are queued, for example, if the 1st response is INT3, and the second INT5,
    // then INT3 is delivered first, and INT5 is not delivered until INT3 is acknowledged
    // (ie. the response interrupts are NOT ORed together to produce INT7 or so)."
    pub fn check_and_clear_irq(&mut self) -> bool {
        let irq = self.hintsts.irq_flags();
        if irq != DiskIrq::NoIrq && self.hintmsk.enable_irq_on_intsts() & irq as u8 != 0 {
            return true;
        }

        false
    }

    pub fn insert_disk(&mut self, cue: Vec<u8>, bin: Vec<u8>) {
        self.cdrom_cue = cue;
        self.cdrom_bin = bin;

        tracing::info!(
            target: "psx_core::cdrom",
            size = self.cdrom_bin.len(),
            "CD-ROM disk inserted",
        );
    }

    fn execute_command(&mut self, command: u8) {
        match command {
            // 0x01 	Nop 		INT3: status
            0x01 => {
                self.pending_command = Some(command);
                self.send_status(&[]);
                self.trigger_irq(DiskIrq::CommandAcknowledged);
            }
            // 0x19 	Test * 	sub, ... 	INT3: ...
            0x19 => {
                let subcommand = self.parameter_fifo.pop_front().unwrap();
                self.execute_subcommand(subcommand);
            }
            _ => {
                tracing::error!(
                    target: "psx_core::cdrom",
                    command = format!("{:02X}", command),
                    "Unimplemented CDROM command",
                );
            }
        }

        self.address.set_busy_status(true);
    }

    fn execute_subcommand(&mut self, subcommand: u8) {
        match subcommand {
            // 20h      -   INT3(yy,mm,dd,ver) ;Get cdrom BIOS date/version (yy,mm,dd,ver)
            0x20 => {
                self.result_fifo.push_back(0x69); // year
                self.result_fifo.push_back(0x69); // month
                self.result_fifo.push_back(0x69); // day
                self.result_fifo.push_back(0x69); // version
                self.trigger_irq(DiskIrq::CommandAcknowledged);
            }
            _ => {
                tracing::error!(
                    target: "psx_core::cdrom",
                    subcommand = format!("{:02X}", subcommand),
                    "Unimplemented CDROM subcommand",
                );
            }
        }
    }

    fn trigger_irq(&mut self, irq: DiskIrq) {
        tracing::trace!(target: "psx_core::cdrom", %irq, "Raised CDROM IRQ");
        self.hintsts.set_irq_flags(irq);
    }

    fn status(&mut self) -> StatusCode {
        let mut status = StatusCode(0);
        status.set_shell_open(self.cdrom_cue.is_empty());
        status
    }

    fn send_status(&mut self, additional_codes: &[u8]) {
        let status = self.status();
        self.result_fifo.push_back(status.0);
        for &param in additional_codes {
            self.result_fifo.push_back(param);
        }
    }
}

impl Bus8 for Cdrom {
    fn read_u8(&mut self, address: u32) -> u8 {
        match address {
            REG_HSTS_ADDR => {
                tracing::trace!(
                    target: "psx_core::cdrom",
                    bank = self.address.current_bank(),
                    param_empty = self.address.parameter_empty(),
                    param_write_ready = self.address.parameter_write_ready(),
                    result_ready = self.address.result_read_ready(),
                    data_request = self.address.data_request(),
                    busy_status = self.address.busy_status(),
                    adcpm_busy = self.address.adpcm_busy(),
                    "CDROM HSTS/ADDRESS read",
                );
                self.address.0
            }
            REG_RESULT_ADDR => self.result_fifo.pop_front().unwrap_or_else(|| {
                tracing::warn!(
                    target: "psx_core::cdrom",
                    "Result FIFO underflow on read",
                );
                0xFF
            }),
            // REG_RDDATA_ADDR => self.rdata,
            REG_HINTMSK_ADDR_R if self.address.current_bank() == 0 || self.address.current_bank() == 2 => {
                tracing::trace!(
                    target: "psx_core::cdrom",
                    bank = self.address.current_bank(),
                    hintmsk = format!("{:08b}", self.hintmsk.0),
                    "CDROM HINTMSK read",
                );
                self.hintmsk.0
            }
            REG_HINTSTS_ADDR if self.address.current_bank() == 1 || self.address.current_bank() == 3 => {
                tracing::trace!(
                    target: "psx_core::cdrom",
                    bank = self.address.current_bank(),
                    hintsts = format!("{:08b}", self.hintsts.0),
                    "CDROM HINTSTS read",
                );
                with_bits!(self.hintsts.0, 5..=7 = 1)
            }
            _ => {
                tracing::error!(
                    target: "psx_core::cdrom",
                    bank = self.address.current_bank(),
                    address = format!("{:08X}", address),
                    "CDROM read not implemented",
                );
                0xFF
            }
        }
    }

    fn write_u8(&mut self, address: u32, value: u8) {
        match address {
            REG_ADDRESS_ADDR => {
                self.address.0 = value;
                tracing::debug!(
                    target: "psx_core::cdrom",
                    bank = self.address.current_bank(),
                    "Bank switched",
                );
            }
            REG_PARAMETER_ADDR if self.address.current_bank() == 0 => {
                self.parameter_fifo.push_back(value);
                if self.parameter_fifo.len() == 16 {
                    self.address.set_parameter_empty(false);
                }
                tracing::debug!(
                    target: "psx_core::cdrom",
                    fifo = %format!("{:02X?}", self.parameter_fifo),
                    "Parameter FIFO write",
                );
            }
            REG_COMMAND_ADDR if self.address.current_bank() == 0 => {
                tracing::debug!(
                    target: "psx_core::cdrom",
                    command = format!("{:02X}", value),
                    "Command received",
                );
                self.execute_command(value);
            }
            REG_HCLRCTL_ADDR if self.address.current_bank() == 1 => {
                // clear specified IRQ flags
                let old_flags: u8 = self.hintsts.irq_flags().into();
                self.hintsts.set_irq_flags((old_flags & !value).into());

                if self.hclrctl.ack_bfwrdy_interrupt() || self.hclrctl.ack_bfempt_interrupt() {
                    tracing::error!(target: "psx_core::cdrom", "Unimplemented ACK for CDROM IRQ");
                }
            }
            REG_HINTMSK_ADDR_W if self.address.current_bank() == 1 => {
                self.hintmsk.0 = value & 0b0001_1111;
            }
            REG_ADPCTL_ADDR if self.address.current_bank() == 3 => self.adpctl = AdpCtlRegister(value),
            _ => tracing::error!(
                target: "psx_core::cdrom",
                address = format!("{:08X}", address),
                value = format!("{:02X}", value),
                bank = self.address.current_bank(),
                "CDROM write not implemented"
            ),
        }
    }
}
