use super::{SharedContext, Widget};
use egui::Ui;

pub struct MmuWidget {
    memory_address: u32,
}

impl MmuWidget {
    pub fn new() -> Self {
        Self {
            memory_address: 0x80000000,
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
                self.memory_address = context.psx.cpu.pc;
            }
        });

        ui.separator();

        let start_addr = self.memory_address & !0xF;
        let memory = &context.psx.mmu.memory;

        for row in 0..32 {
            let addr = start_addr + (row * 16);
            if addr as usize >= memory.len() {
                break;
            }

            let mut line = format!("{:08X}: ", addr);

            for col in 0..16 {
                let byte_addr = addr + col;
                if (byte_addr as usize) < memory.len() {
                    line.push_str(&format!("{:02X} ", memory[byte_addr as usize]));
                } else {
                    line.push_str("?? ");
                }

                if col == 7 {
                    line.push(' ');
                }
            }

            line.push_str(" |");
            for col in 0..16 {
                let byte_addr = addr + col;
                if (byte_addr as usize) < memory.len() {
                    let byte = memory[byte_addr as usize];
                    if byte >= 32 && byte <= 126 {
                        line.push(byte as char);
                    } else {
                        line.push('.');
                    }
                } else {
                    line.push('?');
                }
            }
            line.push('|');

            ui.monospace(line);
        }
    }
}
