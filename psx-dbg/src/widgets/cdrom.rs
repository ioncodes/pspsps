use super::{SharedContext, Widget};
use egui::{Grid, ProgressBar, RichText, Ui};

pub struct CdromWidget {}

impl CdromWidget {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for CdromWidget {
    fn title(&self) -> &str {
        "CDROM"
    }

    fn ui(&mut self, ui: &mut Ui, context: &mut SharedContext) {
        ui.heading("CDROM Drive Status");

        Grid::new("cdrom_grid")
            .striped(true)
            .min_col_width(120.0)
            .max_col_width(180.0)
            .show(ui, |ui| {
                ui.label(RichText::new("Drive State:").strong());
                ui.label(&context.state.cdrom.drive_state);
                ui.end_row();

                ui.label(RichText::new("Target LBA:").strong());
                ui.label(
                    RichText::new(format!(
                        "0x{:X} ({})",
                        context.state.cdrom.sector_lba, context.state.cdrom.sector_lba
                    ))
                    .monospace(),
                );
                ui.end_row();

                ui.label(RichText::new("Current LBA:").strong());
                ui.label(
                    RichText::new(format!(
                        "0x{:X} ({})",
                        context.state.cdrom.sector_lba_current, context.state.cdrom.sector_lba_current
                    ))
                    .monospace(),
                );
                ui.end_row();

                ui.label(RichText::new("Last Command:").strong());
                ui.label(RichText::new(format!("0x{:02X}", context.state.cdrom.last_command)).monospace());
                ui.end_row();

                ui.label(RichText::new("Read In Progress:").strong());
                ui.label(if context.state.cdrom.read_in_progress {
                    "Yes"
                } else {
                    "No"
                });
                ui.end_row();
            });

        // Show progress bar if reading
        if context.state.cdrom.read_in_progress && context.state.cdrom.sector_lba > 0 {
            ui.add_space(10.0);
            ui.label("Read Progress:");
            let progress = context.state.cdrom.sector_lba_current as f32 / context.state.cdrom.sector_lba.max(1) as f32;
            ui.add(ProgressBar::new(progress).show_percentage());
            ui.end_row();
        }
    }
}
