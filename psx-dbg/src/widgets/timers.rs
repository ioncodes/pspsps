use super::{SharedContext, Widget};
use egui::{Grid, Ui};

pub struct TimersWidget {}

impl TimersWidget {
    pub fn new() -> Self {
        Self {}
    }

    fn get_clock_source(timer_id: u8, mode: u32) -> &'static str {
        let clock_source = ((mode >> 8) & 0x3) as u8;

        match timer_id {
            0 => {
                // Counter 0: 0 or 2 = System Clock, 1 or 3 = Dotclock
                match clock_source {
                    0 | 2 => "System Clock",
                    1 | 3 => "Dot Clock",
                    _ => unreachable!(),
                }
            }
            1 => {
                // Counter 1: 0 or 2 = System Clock, 1 or 3 = Hblank
                match clock_source {
                    0 | 2 => "System Clock",
                    1 | 3 => "HBLANK",
                    _ => unreachable!(),
                }
            }
            2 => {
                // Counter 2: 0 or 1 = System Clock, 2 or 3 = System Clock/8
                match clock_source {
                    0 | 1 => "System Clock",
                    2 | 3 => "System Clock/8",
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}

impl Widget for TimersWidget {
    fn title(&self) -> &str {
        "Timers"
    }

    fn ui(&mut self, ui: &mut Ui, context: &mut SharedContext) {
        ui.heading("Timers");

        Grid::new("timers_grid")
            .striped(true)
            .min_col_width(70.0)
            .show(ui, |ui| {
                // Header
                ui.label(egui::RichText::new("Timer").strong());
                ui.label(egui::RichText::new("Counter").strong());
                ui.label(egui::RichText::new("Target").strong());
                ui.label(egui::RichText::new("Clock Source").strong());
                ui.end_row();

                // Timer 0
                ui.label("Timer 0");
                ui.label(egui::RichText::new(format!("0x{:04X}", context.state.timers.timer0_counter)).monospace());
                ui.label(egui::RichText::new(format!("0x{:04X}", context.state.timers.timer0_target)).monospace());
                ui.label(Self::get_clock_source(0, context.state.timers.timer0_mode));
                ui.end_row();

                // Timer 1
                ui.label("Timer 1");
                ui.label(egui::RichText::new(format!("0x{:04X}", context.state.timers.timer1_counter)).monospace());
                ui.label(egui::RichText::new(format!("0x{:04X}", context.state.timers.timer1_target)).monospace());
                ui.label(Self::get_clock_source(1, context.state.timers.timer1_mode));
                ui.end_row();

                // Timer 2
                ui.label("Timer 2");
                ui.label(egui::RichText::new(format!("0x{:04X}", context.state.timers.timer2_counter)).monospace());
                ui.label(egui::RichText::new(format!("0x{:04X}", context.state.timers.timer2_target)).monospace());
                ui.label(Self::get_clock_source(2, context.state.timers.timer2_mode));
                ui.end_row();
            });
    }
}
