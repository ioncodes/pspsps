use egui::Ui;
use egui_toast::Toasts;

pub mod breakpoints;
pub mod cpu;
pub mod mmu;
pub mod trace;
pub mod tty;

pub use breakpoints::BreakpointsWidget;
pub use cpu::CpuWidget;
pub use mmu::MmuWidget;
pub use trace::TraceWidget;
pub use tty::TtyWidget;

use crate::io::DebuggerEvent;
use crate::states;

pub trait Widget {
    fn title(&self) -> &str;
    fn ui(&mut self, ui: &mut Ui, shared_context: &mut SharedContext);
}

pub struct SharedContext<'a> {
    pub channel_send: &'a crossbeam_channel::Sender<DebuggerEvent>,
    pub state: &'a mut states::State,
    pub toasts: &'a mut Toasts,
}
