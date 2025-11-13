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
                    context
                        .channel_send
                        .send(crate::io::DebuggerEvent::AddBreakpoint(addr))
                        .expect("Failed to send add breakpoint event");
                    self.new_breakpoint_address.clear();
                }
            }

            if ui.button("Clear All").clicked() {
                context
                    .channel_send
                    .send(crate::io::DebuggerEvent::ClearBreakpoints)
                    .expect("Failed to send clear breakpoints event");
            }
        });

        ui.separator();

        let breakpoints: Vec<u32> = context
            .state
            .breakpoints
            .breakpoints
            .iter()
            .copied()
            .collect();

        if breakpoints.is_empty() {
            ui.label("No breakpoints set.");
            return;
        }

        for &addr in &breakpoints {
            ui.horizontal(|ui| {
                ui.monospace(format!("{:08X}", addr));

                if ui.button("Remove").clicked() {
                    context
                        .channel_send
                        .send(crate::io::DebuggerEvent::RemoveBreakpoint(addr))
                        .expect("Failed to send remove breakpoint event");
                }

                if ui.button("Show in Disassembly").clicked() {
                    *context.show_in_disassembly = Some(addr);
                }
            });
        }

        ui.separator();
        ui.label(format!(
            "Total breakpoints: {}",
            context.state.breakpoints.breakpoints.len()
        ));
    }
}
