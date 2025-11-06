use std::io::Write as _;

use crate::io::DebuggerEvent;

use super::{SharedContext, Widget};
use egui::{CollapsingHeader, Label, RichText, Ui};
use egui_toast::{Toast, ToastKind};
use psx_core::cpu::cop::Cop;
use psx_core::cpu::cop::cop0::{COP0_SR, Exception};
use psx_core::cpu::decoder::Instruction;
use psx_core::mmu::Addressable;
use std::time::Duration;

const INSTRUCTIONS_TO_DISPLAY: usize = 128;

pub struct CpuWidget {
    follow_pc: bool,
    current_address: String,
}

impl CpuWidget {
    pub fn new() -> Self {
        Self {
            follow_pc: true,
            current_address: "00000000".to_string(),
        }
    }
}

impl Widget for CpuWidget {
    fn title(&self) -> &str {
        "CPU"
    }

    fn ui(&mut self, ui: &mut Ui, context: &mut SharedContext) {
        ui.heading("Controls");
        ui.horizontal(|ui| {
            if context.state.is_running {
                if ui.button("Pause").clicked() {
                    context
                        .channel_send
                        .send(DebuggerEvent::Pause)
                        .expect("Failed to send pause event");
                }
            } else {
                if ui.button("Run").clicked() {
                    context
                        .channel_send
                        .send(DebuggerEvent::Run)
                        .expect("Failed to send run event");
                }
            }

            if ui.button("Step").clicked() {
                context
                    .channel_send
                    .send(DebuggerEvent::Step)
                    .expect("Failed to send step event");
            }

            if ui.button("Reset").clicked() {
                context
                    .channel_send
                    .send(DebuggerEvent::Reset)
                    .expect("Failed to send reset event");
            }

            if ui.button("Dump Memory").clicked() {
                let mut file = std::fs::File::create("memory_dump.bin").unwrap();
                file.write_all(context.state.mmu.data.as_slice()).unwrap();

                context.toasts.add(Toast {
                    text: "Memory dumped to memory_dump.bin".into(),
                    kind: ToastKind::Success,
                    options: egui_toast::ToastOptions::default()
                        .duration(Some(Duration::from_secs(3))),
                    style: Default::default(),
                });
            }
        });

        ui.separator();

        ui.heading("Registers");
        ui.monospace(format!(
            "$00: {:08X}  $at: {:08X}  $v0: {:08X}  $v1: {:08X}",
            context.state.cpu.registers[0],
            context.state.cpu.registers[1],
            context.state.cpu.registers[2],
            context.state.cpu.registers[3]
        ));
        ui.monospace(format!(
            "$a0: {:08X}  $a1: {:08X}  $a2: {:08X}  $a3: {:08X}",
            context.state.cpu.registers[4],
            context.state.cpu.registers[5],
            context.state.cpu.registers[6],
            context.state.cpu.registers[7]
        ));
        ui.monospace(format!(
            "$t0: {:08X}  $t1: {:08X}  $t2: {:08X}  $t3: {:08X}",
            context.state.cpu.registers[8],
            context.state.cpu.registers[9],
            context.state.cpu.registers[10],
            context.state.cpu.registers[11]
        ));
        ui.monospace(format!(
            "$t4: {:08X}  $t5: {:08X}  $t6: {:08X}  $t7: {:08X}",
            context.state.cpu.registers[12],
            context.state.cpu.registers[13],
            context.state.cpu.registers[14],
            context.state.cpu.registers[15]
        ));
        ui.monospace(format!(
            "$s0: {:08X}  $s1: {:08X}  $s2: {:08X}  $s3: {:08X}",
            context.state.cpu.registers[16],
            context.state.cpu.registers[17],
            context.state.cpu.registers[18],
            context.state.cpu.registers[19]
        ));
        ui.monospace(format!(
            "$s4: {:08X}  $s5: {:08X}  $s6: {:08X}  $s7: {:08X}",
            context.state.cpu.registers[20],
            context.state.cpu.registers[21],
            context.state.cpu.registers[22],
            context.state.cpu.registers[23]
        ));
        ui.monospace(format!(
            "$t8: {:08X}  $t9: {:08X}  $k0: {:08X}  $k1: {:08X}",
            context.state.cpu.registers[24],
            context.state.cpu.registers[25],
            context.state.cpu.registers[26],
            context.state.cpu.registers[27]
        ));
        ui.monospace(format!(
            "$gp: {:08X}  $sp: {:08X}  $fp: {:08X}  $ra: {:08X}",
            context.state.cpu.registers[28],
            context.state.cpu.registers[29],
            context.state.cpu.registers[30],
            context.state.cpu.registers[31]
        ));
        ui.monospace(format!(
            "$hi: {:08X}  $lo: {:08X}",
            context.state.cpu.hi, context.state.cpu.lo
        ));

        ui.separator();

        CollapsingHeader::new("COP0 Registers")
            .default_open(true)
            .show(ui, |ui| {
                CollapsingHeader::new("Status Register")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.monospace(format!(
                            "SR: {:08X}",
                            context.state.cpu.cop0.read_register(COP0_SR)
                        ));
                        ui.monospace(format!(
                            "Isolate Cache: {}",
                            if context.state.cpu.cop0.sr.isolate_cache() {
                                "Yes"
                            } else {
                                "No"
                            }
                        ));
                        ui.monospace(format!("Exception PC: {:08X}", context.state.cpu.cop0.epc));
                    });
                CollapsingHeader::new("Cause Register")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.monospace(format!(
                            "Cause: {:08X}",
                            context.state.cpu.cop0.read_register(13)
                        ));
                        ui.monospace(format!(
                            "Exception Code: {} ({:08X})",
                            Exception::from(context.state.cpu.cop0.cause.exception_code()),
                            context.state.cpu.cop0.cause.exception_code()
                        ));
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

        ui.heading("Disassembly");

        ui.horizontal(|ui| {
            if self.follow_pc {
                self.current_address = format!("{:08X}", context.state.cpu.pc);
            }

            if let Some(addr) = context.show_in_disassembly.take() {
                self.current_address = format!("{:08X}", addr);
                self.follow_pc = false;
            }

            ui.label("Address:");
            if ui.text_edit_singleline(&mut self.current_address).changed() {
                self.follow_pc = false;
            }
            ui.checkbox(&mut self.follow_pc, "Follow PC");

            ui.horizontal(|ui| {
                ui.label("PC:");
                ui.monospace(format!("{:08X}", context.state.cpu.pc));
            });
        });

        ui.separator();

        let start =
            u32::from_str_radix(&self.current_address, 16).unwrap_or(context.state.cpu.pc) as usize;

        let instructions: Vec<(u32, Instruction)> = (0..INSTRUCTIONS_TO_DISPLAY)
            .map(|i| {
                let addr = (start + i * 4) as u32;
                let instr_raw = context.state.mmu.read_u32(addr);
                let instr = Instruction::decode(instr_raw);
                (addr, instr)
            })
            .collect();

        for (addr, instr) in instructions {
            ui.horizontal(|ui| {
                let has_breakpoint = context.state.breakpoints.breakpoints.contains(&addr);

                let line = format!("{:08X}: {}", addr, instr);
                let line = if has_breakpoint {
                    Label::new(RichText::new(line).monospace().color(egui::Color32::RED))
                } else {
                    Label::new(RichText::new(line).monospace())
                };

                if ui
                    .add(line)
                    .on_hover_text(
                        RichText::new(format!(
                            "Raw: {:08X}\n\
                         Opcode: {}\n\
                         op: {:02X}\n\
                         rs: {:02X}\n\
                         rt: {:02X}\n\
                         rd: {:02X}\n\
                         shamt: {:02X}\n\
                         funct: {:02X}\n\
                         imm: {:04X}\n\
                         offset: {}\n\
                         address: {:08X}\n",
                            instr.raw,
                            instr.opcode,
                            instr.op(),
                            instr.rs(),
                            instr.rt(),
                            instr.rd(),
                            instr.shamt(),
                            instr.funct(),
                            instr.immediate(),
                            instr.offset(),
                            instr.address()
                        ))
                        .monospace(),
                    )
                    .clicked()
                {
                    if has_breakpoint {
                        context
                            .channel_send
                            .send(DebuggerEvent::RemoveBreakpoint(addr))
                            .expect("Failed to send remove breakpoint event");
                    } else {
                        context
                            .channel_send
                            .send(DebuggerEvent::AddBreakpoint(addr))
                            .expect("Failed to send add breakpoint event");
                    }
                }
            });
        }
    }
}
