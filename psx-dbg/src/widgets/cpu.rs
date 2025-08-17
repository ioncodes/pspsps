use super::{SharedContext, Widget};
use egui::{CollapsingHeader, Label, RichText, Ui};
use psx_core::cpu::cop::Cop;
use psx_core::cpu::cop::cop0::COP0_SR;
use psx_core::cpu::decoder::Instruction;

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
            if *context.is_running {
                if ui.button("Pause").clicked() {
                    *context.is_running = false;
                }
            } else {
                if ui.button("Run").clicked() {
                    *context.is_running = true;
                    *context.breakpoint_hit = false;
                }
            }

            if ui.button("Step").clicked() {
                context.psx.step();
                *context.breakpoint_hit = false;
            }

            if ui.button("Reset").clicked() {
                static BIOS: &[u8] = include_bytes!("../../../bios/SCPH1000.BIN");
                *context.psx = psx_core::psx::Psx::new(BIOS);
                *context.is_running = false;
                *context.breakpoint_hit = false;
            }
        });

        ui.separator();

        ui.heading("Registers");
        ui.monospace(format!(
            "$00: {:08X}  $at: {:08X}  $v0: {:08X}  $v1: {:08X}",
            context.psx.cpu.registers[0],
            context.psx.cpu.registers[1],
            context.psx.cpu.registers[2],
            context.psx.cpu.registers[3]
        ));
        ui.monospace(format!(
            "$a0: {:08X}  $a1: {:08X}  $a2: {:08X}  $a3: {:08X}",
            context.psx.cpu.registers[4],
            context.psx.cpu.registers[5],
            context.psx.cpu.registers[6],
            context.psx.cpu.registers[7]
        ));
        ui.monospace(format!(
            "$t0: {:08X}  $t1: {:08X}  $t2: {:08X}  $t3: {:08X}",
            context.psx.cpu.registers[8],
            context.psx.cpu.registers[9],
            context.psx.cpu.registers[10],
            context.psx.cpu.registers[11]
        ));
        ui.monospace(format!(
            "$t4: {:08X}  $t5: {:08X}  $t6: {:08X}  $t7: {:08X}",
            context.psx.cpu.registers[12],
            context.psx.cpu.registers[13],
            context.psx.cpu.registers[14],
            context.psx.cpu.registers[15]
        ));
        ui.monospace(format!(
            "$s0: {:08X}  $s1: {:08X}  $s2: {:08X}  $s3: {:08X}",
            context.psx.cpu.registers[16],
            context.psx.cpu.registers[17],
            context.psx.cpu.registers[18],
            context.psx.cpu.registers[19]
        ));
        ui.monospace(format!(
            "$s4: {:08X}  $s5: {:08X}  $s6: {:08X}  $s7: {:08X}",
            context.psx.cpu.registers[20],
            context.psx.cpu.registers[21],
            context.psx.cpu.registers[22],
            context.psx.cpu.registers[23]
        ));
        ui.monospace(format!(
            "$t8: {:08X}  $t9: {:08X}  $k0: {:08X}  $k1: {:08X}",
            context.psx.cpu.registers[24],
            context.psx.cpu.registers[25],
            context.psx.cpu.registers[26],
            context.psx.cpu.registers[27]
        ));
        ui.monospace(format!(
            "$gp: {:08X}  $sp: {:08X}  $fp: {:08X}  $ra: {:08X}",
            context.psx.cpu.registers[28],
            context.psx.cpu.registers[29],
            context.psx.cpu.registers[30],
            context.psx.cpu.registers[31]
        ));
        ui.monospace(format!(
            "$hi: {:08X}  $lo: {:08X}",
            context.psx.cpu.hi, context.psx.cpu.lo
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
                            context.psx.cpu.cop0.read_register(COP0_SR)
                        ));
                        ui.monospace(format!(
                            "Current Mode: {}",
                            if !context.psx.cpu.cop0.sr.current_mode() {
                                "Kernel"
                            } else {
                                "User"
                            }
                        ));
                        ui.monospace(format!(
                            "Enabled: {}",
                            if context.psx.cpu.cop0.sr.cop0_enable() {
                                "Yes"
                            } else {
                                "No"
                            }
                        ));
                    });
                CollapsingHeader::new("Cause Register")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.monospace(format!(
                            "Cause: {:08X}",
                            context.psx.cpu.cop0.read_register(13)
                        ));
                        ui.monospace(format!(
                            "Exception Code: {}",
                            context.psx.cpu.cop0.cause.exception_code()
                        ));
                        ui.monospace(format!(
                            "Software Interrupts: {}",
                            context.psx.cpu.cop0.cause.software_interrupts()
                        ));
                        ui.monospace(format!(
                            "Interrupt Pending: {}",
                            context.psx.cpu.cop0.cause.interrupt_pending()
                        ));
                    });
            });

        ui.separator();

        ui.heading("Disassembly");

        ui.horizontal(|ui| {
            if self.follow_pc {
                self.current_address = format!("{:08X}", context.psx.cpu.pc);
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
                ui.monospace(format!("{:08X}", context.psx.cpu.pc));
            });
        });

        ui.separator();

        let start =
            u32::from_str_radix(&self.current_address, 16).unwrap_or(context.psx.cpu.pc) as usize;

        let instructions: Vec<(u32, Instruction)> = (0..40)
            .map(|i| {
                let addr = (start + i * 4) as u32;
                let instr_raw = context.psx.mmu.read_u32(addr);
                let instr = Instruction::decode(instr_raw);
                (addr, instr)
            })
            .collect();

        for (addr, instr) in instructions {
            ui.horizontal(|ui| {
                let has_breakpoint = context.breakpoints.contains(&addr);

                let line = format!("{:08X}: {}", addr, instr);
                let line = if has_breakpoint {
                    Label::new(RichText::new(line).monospace().color(egui::Color32::RED))
                } else {
                    Label::new(RichText::new(line).monospace())
                };
                if ui.add(line).clicked() {
                    if has_breakpoint {
                        context.breakpoints.remove(&addr);
                    } else {
                        context.breakpoints.insert(addr);
                    }
                }
            });
        }
    }
}
