use super::{SharedContext, Widget};
use egui::{Grid, RichText, Ui};

pub struct IrqWidget {}

impl IrqWidget {
    pub fn new() -> Self {
        Self {}
    }

    fn render_bit(ui: &mut Ui, bit_name: &str, stat_val: bool, mask_val: bool) {
        ui.label(bit_name);

        // I_STAT bit
        let stat_text = if stat_val {
            RichText::new("1").color(egui::Color32::from_rgb(100, 255, 100))
        } else {
            RichText::new("0").color(egui::Color32::GRAY)
        };
        ui.label(stat_text);

        // I_MASK bit
        let mask_text = if mask_val {
            RichText::new("1").color(egui::Color32::from_rgb(100, 200, 255))
        } else {
            RichText::new("0").color(egui::Color32::GRAY)
        };
        ui.label(mask_text);

        ui.end_row();
    }
}

impl Widget for IrqWidget {
    fn title(&self) -> &str {
        "IRQ"
    }

    fn ui(&mut self, ui: &mut Ui, context: &mut SharedContext) {
        ui.heading("Interrupt Registers");

        ui.horizontal(|ui| {
            ui.label(RichText::new("I_STAT:").strong());
            ui.label(RichText::new(format!("0x{:04X}", context.state.irq.i_stat)).monospace());
            ui.label("  ");
            ui.label(RichText::new("I_MASK:").strong());
            ui.label(RichText::new(format!("0x{:04X}", context.state.irq.i_mask)).monospace());
        });

        ui.add_space(10.0);

        Grid::new("irq_grid")
            .striped(true)
            .min_col_width(80.0)
            .show(ui, |ui| {
                // Header
                ui.label(RichText::new("Bit").strong());
                ui.label(RichText::new("I_STAT").strong());
                ui.label(RichText::new("I_MASK").strong());
                ui.end_row();

                let i_stat = context.state.irq.i_stat;
                let i_mask = context.state.irq.i_mask;

                Self::render_bit(ui, "0: VBLANK", (i_stat & (1 << 0)) != 0, (i_mask & (1 << 0)) != 0);
                Self::render_bit(ui, "1: GPU", (i_stat & (1 << 1)) != 0, (i_mask & (1 << 1)) != 0);
                Self::render_bit(ui, "2: CDROM", (i_stat & (1 << 2)) != 0, (i_mask & (1 << 2)) != 0);
                Self::render_bit(ui, "3: DMA", (i_stat & (1 << 3)) != 0, (i_mask & (1 << 3)) != 0);
                Self::render_bit(ui, "4: TMR0", (i_stat & (1 << 4)) != 0, (i_mask & (1 << 4)) != 0);
                Self::render_bit(ui, "5: TMR1", (i_stat & (1 << 5)) != 0, (i_mask & (1 << 5)) != 0);
                Self::render_bit(ui, "6: TMR2", (i_stat & (1 << 6)) != 0, (i_mask & (1 << 6)) != 0);
                Self::render_bit(ui, "7: Controller", (i_stat & (1 << 7)) != 0, (i_mask & (1 << 7)) != 0);
                Self::render_bit(ui, "8: SIO", (i_stat & (1 << 8)) != 0, (i_mask & (1 << 8)) != 0);
                Self::render_bit(ui, "9: SPU", (i_stat & (1 << 9)) != 0, (i_mask & (1 << 9)) != 0);
                Self::render_bit(ui, "10: Lightpen", (i_stat & (1 << 10)) != 0, (i_mask & (1 << 10)) != 0);
            });
    }
}
