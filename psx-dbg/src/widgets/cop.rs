use super::{SharedContext, Widget};
use crate::colors::*;
use egui::{CollapsingHeader, Grid, RichText, Ui};
use psx_core::cpu::cop::Cop;
use psx_core::cpu::cop::cop0::{COP0_SR, Exception};
use psx_core::cpu::lut::{GTE_CONTROL_REGISTER_NAME_LUT, GTE_DATA_REGISTER_NAME_LUT};

pub struct CopWidget;

impl CopWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for CopWidget {
    fn title(&self) -> &str {
        "COP"
    }

    fn ui(&mut self, ui: &mut Ui, context: &mut SharedContext) {
        ui.heading("Coprocessor Registers");

        CollapsingHeader::new("COP0 Registers")
            .default_open(true)
            .show(ui, |ui| {
                CollapsingHeader::new("Status Register")
                    .default_open(true)
                    .show(ui, |ui| {
                        let sr = context.state.cpu.cop0.read_register(COP0_SR as u8);
                        let sr_prev = context.state.previous_cpu.cop0.read_register(COP0_SR as u8);
                        let sr_changed = sr != sr_prev;

                        let sr_text = format!("SR: {:08X}", sr);
                        if sr_changed {
                            ui.colored_label(COLOR_DIRTY, RichText::new(sr_text).monospace());
                        } else {
                            ui.monospace(sr_text);
                        }

                        ui.monospace(format!(
                            "Isolate Cache: {}",
                            if context.state.cpu.cop0.sr.isolate_cache() {
                                "Yes"
                            } else {
                                "No"
                            }
                        ));

                        let epc = context.state.cpu.cop0.epc;
                        let epc_prev = context.state.previous_cpu.cop0.epc;
                        let epc_changed = epc != epc_prev;

                        let epc_text = format!("Exception PC: {:08X}", epc);
                        if epc_changed {
                            ui.colored_label(COLOR_DIRTY, RichText::new(epc_text).monospace());
                        } else {
                            ui.monospace(epc_text);
                        }
                    });

                CollapsingHeader::new("Cause Register")
                    .default_open(true)
                    .show(ui, |ui| {
                        let cause = context.state.cpu.cop0.read_register(13);
                        let cause_prev = context.state.previous_cpu.cop0.read_register(13);
                        let cause_changed = cause != cause_prev;

                        let cause_text = format!("Cause: {:08X}", cause);
                        if cause_changed {
                            ui.colored_label(COLOR_DIRTY, RichText::new(cause_text).monospace());
                        } else {
                            ui.monospace(cause_text);
                        }

                        let exc_code = context.state.cpu.cop0.cause.exception_code();
                        let exc_code_prev = context.state.previous_cpu.cop0.cause.exception_code();
                        let exc_code_changed = exc_code != exc_code_prev;

                        let exc_code_text = format!("Exception Code: {} ({:08X})", Exception::from(exc_code), exc_code);
                        if exc_code_changed {
                            ui.colored_label(COLOR_DIRTY, RichText::new(exc_code_text).monospace());
                        } else {
                            ui.monospace(exc_code_text);
                        }

                        ui.monospace(format!(
                            "Software Interrupts: {}",
                            context.state.cpu.cop0.cause.software_interrupts()
                        ));
                        ui.monospace(format!(
                            "Interrupt Pending: {}",
                            context.state.cpu.cop0.cause.interrupt_pending()
                        ));
                    });
            });

        ui.separator();

        CollapsingHeader::new("COP2 Registers (GTE)")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Left column: Data Registers (0-31)
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Data Registers").strong());
                        Grid::new("cop2_data_regs")
                            .num_columns(2)
                            .spacing([20.0, 2.0])
                            .show(ui, |ui| {
                                for i in 0..32 {
                                    let reg = context.state.cpu.cop2.read_register(i);
                                    let reg_prev = context.state.previous_cpu.cop2.read_register(i);
                                    let changed = reg != reg_prev;

                                    let name_text =
                                        RichText::new(format!("{}:", GTE_DATA_REGISTER_NAME_LUT[i as usize])).strong().monospace();
                                    let value_text = RichText::new(format!("{:08X}", reg)).monospace();

                                    ui.label(name_text);
                                    if changed {
                                        ui.colored_label(COLOR_DIRTY, value_text);
                                    } else {
                                        ui.label(value_text);
                                    }
                                    ui.end_row();
                                }
                            });
                    });

                    ui.separator();

                    // Right column: Control Registers (32-63)
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Control Registers").strong());
                        Grid::new("cop2_control_regs")
                            .num_columns(2)
                            .spacing([20.0, 2.0])
                            .show(ui, |ui| {
                                for i in 32..64 {
                                    let reg = context.state.cpu.cop2.read_register(i);
                                    let reg_prev = context.state.previous_cpu.cop2.read_register(i);
                                    let changed = reg != reg_prev;

                                    let name_text =
                                        RichText::new(format!("{}:", GTE_CONTROL_REGISTER_NAME_LUT[(i - 32) as usize]))
                                            .strong().monospace();
                                    let value_text = RichText::new(format!("{:08X}", reg)).monospace();

                                    ui.label(name_text);
                                    if changed {
                                        ui.colored_label(COLOR_DIRTY, value_text);
                                    } else {
                                        ui.label(value_text);
                                    }
                                    ui.end_row();
                                }
                            });
                    });
                });
            });
    }
}
