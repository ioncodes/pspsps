use crate::colors::COLOR_DIRTY;
use super::{SharedContext, Widget};
use egui::{RichText, Ui};
use psx_core::mmu::bus::Bus8 as _;
use std::collections::HashMap;

const ROWS_TO_DISPLAY: u32 = 128;

pub struct MmuWidget {
    memory_address: u32,
    previous_memory: HashMap<u32, u8>,
}

impl MmuWidget {
    pub fn new() -> Self {
        Self {
            memory_address: 0x80000000,
            previous_memory: HashMap::new(),
        }
    }
}

impl Widget for MmuWidget {
    fn title(&self) -> &str {
        "MMU"
    }

    fn ui(&mut self, ui: &mut Ui, context: &mut SharedContext) {
        ui.heading("Memory Viewer");

        ui.horizontal(|ui| {
            ui.label("Address:");
            let mut addr_str = format!("{:08X}", self.memory_address);
            if ui.text_edit_singleline(&mut addr_str).changed() {
                if let Ok(addr) = u32::from_str_radix(&addr_str, 16) {
                    self.memory_address = addr;
                }
            }

            if ui.button("Go to PC").clicked() {
                self.memory_address = context.state.cpu.pc;
            }

            if ui.button("Refresh").clicked() {
                // Capture current visible memory before refreshing
                let start_addr = self.memory_address & !0xF;
                self.previous_memory.clear();
                for row in 0..ROWS_TO_DISPLAY {
                    for col in 0..16 {
                        let addr = start_addr + (row * 16) + col;
                        let byte = context.state.mmu.read_u8(addr);
                        self.previous_memory.insert(addr, byte);
                    }
                }

                context
                    .channel_send
                    .send(crate::io::DebuggerEvent::UpdateMmu)
                    .expect("Failed to send refresh event");
            }
        });

        ui.separator();

        let start_addr = self.memory_address & !0xF;

        // Reduce vertical spacing between memory rows
        ui.spacing_mut().item_spacing.y = 0.0;

        for row in 0..ROWS_TO_DISPLAY {
            let addr = start_addr + (row * 16);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                // Address
                ui.label(RichText::new(format!("{:08X}: ", addr)).monospace());

                // Hex bytes
                for col in 0..16 {
                    let byte_addr = addr + col;
                    let byte = context.state.mmu.read_u8(byte_addr);

                    // Check if byte changed
                    let changed = self.previous_memory.get(&byte_addr)
                        .map(|&prev| prev != byte)
                        .unwrap_or(false);

                    if changed {
                        ui.colored_label(COLOR_DIRTY, RichText::new(format!("{:02X}", byte)).monospace());
                    } else {
                        ui.label(RichText::new(format!("{:02X}", byte)).monospace());
                    }
                    ui.label(RichText::new(" ").monospace());

                    if col == 7 {
                        ui.label(RichText::new(" ").monospace());
                    }
                }

                ui.label(RichText::new(" |").monospace());

                // ASCII representation
                for col in 0..16 {
                    let byte_addr = addr + col;
                    let byte = context.state.mmu.read_u8(byte_addr);

                    // Check if byte changed
                    let changed = self.previous_memory.get(&byte_addr)
                        .map(|&prev| prev != byte)
                        .unwrap_or(false);

                    let ch = if byte >= 32 && byte <= 126 {
                        byte as char
                    } else {
                        '.'
                    };

                    if changed {
                        ui.colored_label(COLOR_DIRTY, RichText::new(ch.to_string()).monospace());
                    } else {
                        ui.label(RichText::new(ch.to_string()).monospace());
                    }
                }

                ui.label(RichText::new("|").monospace());
            });
        }
    }
}
