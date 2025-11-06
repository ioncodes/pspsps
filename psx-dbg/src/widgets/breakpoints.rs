use super::{SharedContext, Widget};
use egui::Ui;

pub struct BreakpointsWidget {
    new_breakpoint_address: String,
}

impl BreakpointsWidget {
    pub fn new() -> Self {
        Self {
            new_breakpoint_address: String::new(),
        }
    }
}

impl Widget for BreakpointsWidget {
    fn title(&self) -> &str {
        "Breakpoints"
    }

    fn ui(&mut self, ui: &mut Ui, context: &mut SharedContext) {
        ui.heading("Add Breakpoint");

        ui.horizontal(|ui| {
            ui.label("Address:");
            ui.text_edit_singleline(&mut self.new_breakpoint_address);

            if ui.button("Add").clicked() {
                if let Ok(addr) =
                    u32::from_str_radix(&self.new_breakpoint_address.trim_start_matches("0x"), 16)
                {
                    context.breakpoints.insert(addr);
                    self.new_breakpoint_address.clear();
                }
            }

            if ui.button("Clear All").clicked() {
                context.breakpoints.clear();
            }
        });

        ui.separator();

        ui.heading("Active Breakpoints");

        let breakpoints: Vec<u32> = context.breakpoints.iter().copied().collect();

        for &addr in &breakpoints {
            ui.horizontal(|ui| {
                ui.monospace(format!("{:08X}", addr));

                if ui.button("Remove").clicked() {
                    context.breakpoints.remove(&addr);
                }

                if ui.button("Show in Disassembly").clicked() {
                    *context.show_in_disassembly = Some(addr);
                }
            });
        }

        ui.separator();
        ui.label(format!("Total breakpoints: {}", context.breakpoints.len()));
    }
}
