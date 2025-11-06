use super::{SharedContext, Widget};
use super::instruction_renderer::render_instruction;
use crate::io::DebuggerEvent;
use egui::{CollapsingHeader, RichText, Ui};
use egui_toast::{Toast, ToastKind};
use psx_core::cpu::cop::Cop;
use psx_core::cpu::cop::cop0::{COP0_SR, Exception};
use psx_core::cpu::decoder::Instruction;
use psx_core::mmu::bus::{Bus8, Bus32 as _};
use rayon::prelude::*;
use std::io::Write as _;
use std::time::Duration;
use crate::colors::*;

const INSTRUCTIONS_TO_DISPLAY: usize = 128;

#[derive(Copy, Clone)]
struct SendMmuPtr(usize);
unsafe impl Send for SendMmuPtr {}
unsafe impl Sync for SendMmuPtr {}

impl SendMmuPtr {
    unsafe fn new(ptr: *mut crate::states::mmu::MmuState) -> Self {
        Self(ptr as usize)
    }

    unsafe fn as_ptr(&self) -> *mut crate::states::mmu::MmuState {
        self.0 as *mut crate::states::mmu::MmuState
    }
}

fn dump_memory_multithreaded(mmu_state: &mut crate::states::mmu::MmuState, output_path: &str) -> std::io::Result<()> {
    // Split 4GB into chunks for parallel processing
    const CHUNK_SIZE: u32 = 16 * 1024 * 1024; // 16MB chunks
    const TOTAL_SIZE: u64 = 0x1_0000_0000; // 4GB
    let num_chunks = (TOTAL_SIZE / CHUNK_SIZE as u64) as u32;

    let send_ptr = unsafe { SendMmuPtr::new(mmu_state as *mut crate::states::mmu::MmuState) };

    // Collect all chunks in parallel
    let chunks: Vec<(u32, Vec<u8>)> = (0..num_chunks)
        .into_par_iter()
        .map(move |chunk_idx| {
            let start_addr = chunk_idx * CHUNK_SIZE;
            let mut buffer = Vec::with_capacity(CHUNK_SIZE as usize);

            // SAFETY: We accept unsafe here as there *should* be no side-effects as long as we only read
            unsafe {
                let mmu_ref = &mut *send_ptr.as_ptr();

                // Read this chunk through MmuState to handle address canonicalization
                for offset in 0..CHUNK_SIZE {
                    let addr = start_addr.wrapping_add(offset);
                    buffer.push(mmu_ref.read_u8(addr));
                }
            }

            (start_addr, buffer)
        })
        .collect();

    let mut file = std::fs::File::create(output_path)?;
    for (_, buffer) in chunks {
        file.write_all(&buffer)?;
    }

    Ok(())
}

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
                // Set flag to update previous_cpu on next update (to show what changed)
                context.state.should_update_previous_cpu = true;
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
                match dump_memory_multithreaded(&mut context.state.mmu, "memory_dump.bin") {
                    Ok(_) => {
                        context.toasts.add(Toast {
                            text: "Memory dumped to memory_dump.bin (4GB)".into(),
                            kind: ToastKind::Success,
                            options: egui_toast::ToastOptions::default().duration(Some(Duration::from_secs(3))),
                            style: Default::default(),
                        });
                    }
                    Err(e) => {
                        context.toasts.add(Toast {
                            text: format!("Failed to dump memory: {}", e).into(),
                            kind: ToastKind::Error,
                            options: egui_toast::ToastOptions::default().duration(Some(Duration::from_secs(5))),
                            style: Default::default(),
                        });
                    }
                }
            }

            if ui.checkbox(&mut context.state.ignore_errors, "Ignore Errors").changed() {
                context
                    .channel_send
                    .send(DebuggerEvent::SetIgnoreErrors(context.state.ignore_errors))
                    .expect("Failed to send set ignore errors event");
            }
        });

        ui.separator();

        ui.heading("Registers");

        // Helper closure to render a register with change highlighting
        let render_reg = |ui: &mut Ui, name: &str, idx: usize| {
            let changed = context.state.cpu.registers[idx] != context.state.previous_cpu.registers[idx];
            let text = format!("{}: {:08X}", name, context.state.cpu.registers[idx]);
            if changed {
                ui.colored_label(egui::Color32::from_rgb(255, 150, 150), RichText::new(text).monospace());
            } else {
                ui.label(RichText::new(text).monospace());
            }
        };

        // Render registers in rows with reduced spacing
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.horizontal(|ui| {
            render_reg(ui, "$00", 0);
            ui.label("  ");
            render_reg(ui, "$at", 1);
            ui.label("  ");
            render_reg(ui, "$v0", 2);
            ui.label("  ");
            render_reg(ui, "$v1", 3);
        });
        ui.horizontal(|ui| {
            render_reg(ui, "$a0", 4);
            ui.label("  ");
            render_reg(ui, "$a1", 5);
            ui.label("  ");
            render_reg(ui, "$a2", 6);
            ui.label("  ");
            render_reg(ui, "$a3", 7);
        });
        ui.horizontal(|ui| {
            render_reg(ui, "$t0", 8);
            ui.label("  ");
            render_reg(ui, "$t1", 9);
            ui.label("  ");
            render_reg(ui, "$t2", 10);
            ui.label("  ");
            render_reg(ui, "$t3", 11);
        });
        ui.horizontal(|ui| {
            render_reg(ui, "$t4", 12);
            ui.label("  ");
            render_reg(ui, "$t5", 13);
            ui.label("  ");
            render_reg(ui, "$t6", 14);
            ui.label("  ");
            render_reg(ui, "$t7", 15);
        });
        ui.horizontal(|ui| {
            render_reg(ui, "$s0", 16);
            ui.label("  ");
            render_reg(ui, "$s1", 17);
            ui.label("  ");
            render_reg(ui, "$s2", 18);
            ui.label("  ");
            render_reg(ui, "$s3", 19);
        });
        ui.horizontal(|ui| {
            render_reg(ui, "$s4", 20);
            ui.label("  ");
            render_reg(ui, "$s5", 21);
            ui.label("  ");
            render_reg(ui, "$s6", 22);
            ui.label("  ");
            render_reg(ui, "$s7", 23);
        });
        ui.horizontal(|ui| {
            render_reg(ui, "$t8", 24);
            ui.label("  ");
            render_reg(ui, "$t9", 25);
            ui.label("  ");
            render_reg(ui, "$k0", 26);
            ui.label("  ");
            render_reg(ui, "$k1", 27);
        });
        ui.horizontal(|ui| {
            render_reg(ui, "$gp", 28);
            ui.label("  ");
            render_reg(ui, "$sp", 29);
            ui.label("  ");
            render_reg(ui, "$fp", 30);
            ui.label("  ");
            render_reg(ui, "$ra", 31);
        });
        ui.horizontal(|ui| {
            let hi_changed = context.state.cpu.hi != context.state.previous_cpu.hi;
            let lo_changed = context.state.cpu.lo != context.state.previous_cpu.lo;

            if hi_changed {
                ui.colored_label(
                    COLOR_DIRTY,
                    RichText::new(format!("$hi: {:08X}", context.state.cpu.hi)).monospace(),
                );
            } else {
                ui.label(RichText::new(format!("$hi: {:08X}", context.state.cpu.hi)).monospace());
            }

            ui.label("  ");

            if lo_changed {
                ui.colored_label(
                    COLOR_DIRTY,
                    RichText::new(format!("$lo: {:08X}", context.state.cpu.lo)).monospace(),
                );
            } else {
                ui.label(RichText::new(format!("$lo: {:08X}", context.state.cpu.lo)).monospace());
            }
        });

        ui.separator();

        CollapsingHeader::new("COP0 Registers")
            .default_open(true)
            .show(ui, |ui| {
                CollapsingHeader::new("Status Register")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.monospace(format!("SR: {:08X}", context.state.cpu.cop0.read_register(COP0_SR as u8)));
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
                        ui.monospace(format!("Cause: {:08X}", context.state.cpu.cop0.read_register(13)));
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

        let start = u32::from_str_radix(&self.current_address, 16).unwrap_or(context.state.cpu.pc) as usize;

        let instructions: Vec<(u32, Instruction)> = (0..INSTRUCTIONS_TO_DISPLAY)
            .map(|i| {
                let addr = (start + i * 4) as u32;
                let instr_raw = context.state.mmu.read_u32(addr);
                let instr = Instruction::decode(instr_raw);
                (addr, instr)
            })
            .collect();

        for (addr, instr) in instructions {
            let has_breakpoint = context.state.breakpoints.breakpoints.contains(&addr);
            let is_pc = addr == context.state.cpu.pc;

            let response = ui
                .horizontal(|ui| {
                    // Render the colorized instruction
                    render_instruction(ui, addr, &instr, is_pc, has_breakpoint);
                })
                .response
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
                     address: {:08X}\n\
                     Click to toggle breakpoint",
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
                );

            if response.clicked() {
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
        }
    }
}
