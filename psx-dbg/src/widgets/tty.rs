use super::{SharedContext, Widget};
use egui::{ScrollArea, Ui};

pub struct TtyWidget;

impl TtyWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for TtyWidget {
    fn title(&self) -> &str {
        "TTY"
    }

    fn ui(&mut self, ui: &mut Ui, _context: &mut SharedContext) {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.monospace(psx_core::cpu::internal::TTY_BUFFER.lock().unwrap().clone());
            });
    }
}
