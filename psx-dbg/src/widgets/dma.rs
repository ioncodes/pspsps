use super::{SharedContext, Widget};
use crate::states::dma::DmaChannelState;
use egui::{Grid, RichText, Ui};

pub struct DmaWidget {}

impl DmaWidget {
    pub fn new() -> Self {
        Self {}
    }

    fn render_channel(ui: &mut egui::Ui, name: &str, channel: &DmaChannelState) {
        ui.label(name);

        let status_text = if channel.active {
            RichText::new("Active").color(egui::Color32::GREEN)
        } else {
            RichText::new("Idle").color(egui::Color32::GRAY)
        };
        ui.label(status_text);

        let direction = if channel.transfer_direction {
            "RAM -> Device"
        } else {
            "Device -> RAM"
        };
        ui.label(direction);

        ui.label(channel.transfer_mode.to_string());
        ui.end_row();
    }
}

impl Widget for DmaWidget {
    fn title(&self) -> &str {
        "DMA"
    }

    fn ui(&mut self, ui: &mut Ui, context: &mut SharedContext) {
        ui.heading("DMA Channels");

        Grid::new("dma_grid")
            .striped(true)
            .min_col_width(100.0)
            .max_col_width(140.0)
            .show(ui, |ui| {
                // Header
                ui.label(RichText::new("Channel").strong());
                ui.label(RichText::new("Status").strong());
                ui.label(RichText::new("Direction").strong());
                ui.label(RichText::new("Mode").strong());
                ui.end_row();

                // All 7 channels
                Self::render_channel(ui, "DMA0 (MDECin)", &context.state.dma.channel0);
                Self::render_channel(ui, "DMA1 (MDECout)", &context.state.dma.channel1);
                Self::render_channel(ui, "DMA2 (GPU)", &context.state.dma.channel2);
                Self::render_channel(ui, "DMA3 (CDROM)", &context.state.dma.channel3);
                Self::render_channel(ui, "DMA4 (SPU)", &context.state.dma.channel4);
                Self::render_channel(ui, "DMA5 (PIO)", &context.state.dma.channel5);
                Self::render_channel(ui, "DMA6 (OTC)", &context.state.dma.channel6);
            });
    }
}
