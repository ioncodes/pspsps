use super::{SharedContext, Widget};
use egui::Ui;
use psx_core::mmu::bus::Bus8 as _;

const ROWS_TO_DISPLAY: u32 = 128;

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
                self.memory_address = context.state.cpu.pc;
            }

            if ui.button("Refresh").clicked() {
                context
                    .channel_send
                    .send(crate::io::DebuggerEvent::UpdateMmu)
                    .expect("Failed to send refresh event");
            }
        });

        ui.separator();

        let start_addr = self.memory_address & !0xF;

        for row in 0..ROWS_TO_DISPLAY {
            let addr = start_addr + (row * 16);

            let mut line = format!("{:08X}: ", addr);

            for col in 0..16 {
                let byte_addr = addr + col;
                let byte = context.state.mmu.read_u8(byte_addr);
                line.push_str(&format!("{:02X} ", byte));

                if col == 7 {
                    line.push(' ');
                }
            }

            line.push_str(" |");
            for col in 0..16 {
                let byte_addr = addr + col;
                let byte = context.state.mmu.read_u8(byte_addr);
                if byte >= 32 && byte <= 126 {
                    line.push(byte as char);
                } else {
                    line.push('.');
                }
            }
            line.push('|');

            ui.monospace(line);
        }
    }
}
