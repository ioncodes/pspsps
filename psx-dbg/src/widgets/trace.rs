use super::{SharedContext, Widget};
use super::instruction_renderer::render_instruction;
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
            .stick_to_bottom(true)
            .show(ui, |ui| {
                // Reduce vertical spacing between trace lines
                ui.spacing_mut().item_spacing.y = 0.0;

                for (address, instruction) in context.state.trace.instructions.iter() {
                    ui.horizontal(|ui| {
                        // Render with colorization (no PC marker, no breakpoint marker)
                        render_instruction(ui, *address, instruction, false, false);
                    });
                }
            });
    }
}
