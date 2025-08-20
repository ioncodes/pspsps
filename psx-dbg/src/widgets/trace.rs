use super::{SharedContext, Widget};
use egui::{ScrollArea, Ui};

pub struct TraceWidget;

impl TraceWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for TraceWidget {
    fn title(&self) -> &str {
        "Trace"
    }

    fn ui(&mut self, ui: &mut Ui, context: &mut SharedContext) {
        ui.heading("Instruction Trace");

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (address, instruction) in context.trace_buffer.iter().rev() {
                    ui.monospace(format!("{:08X}: {}", address, instruction));
                }
            });
    }
}
