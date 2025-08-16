use eframe::egui;
use egui_dock::{DockArea, DockState};
use psx_core::psx::Psx;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tracing_subscriber::Layer as _;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

mod widgets;
use widgets::{BreakpointsWidget, CpuWidget, MmuWidget, SharedContext, Widget};

static BIOS: &[u8] = include_bytes!("../../bios/SCPH1000.BIN");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum TabKind {
    Cpu,
    Mmu,
    Breakpoints,
}

impl std::fmt::Display for TabKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TabKind::Cpu => write!(f, "CPU"),
            TabKind::Mmu => write!(f, "MMU"),
            TabKind::Breakpoints => write!(f, "Breakpoints"),
        }
    }
}

pub struct PsxDebugger {
    psx: Psx,
    is_running: bool,
    dock_state: DockState<TabKind>,
    breakpoints: HashSet<u32>,
    breakpoint_hit: bool,
    widgets: HashMap<TabKind, Box<dyn Widget>>,
    show_in_disassembly: Option<u32>,
}

impl Default for PsxDebugger {
    fn default() -> Self {
        let mut dock_state = DockState::new(vec![TabKind::Cpu]);
        let [_old, mmu_node] = dock_state.main_surface_mut().split_right(
            egui_dock::NodeIndex::root(),
            0.5,
            vec![TabKind::Mmu],
        );
        dock_state
            .main_surface_mut()
            .split_below(mmu_node, 0.5, vec![TabKind::Breakpoints]);

        let mut widgets: HashMap<TabKind, Box<dyn Widget>> = HashMap::new();
        widgets.insert(TabKind::Cpu, Box::new(CpuWidget::new()));
        widgets.insert(TabKind::Mmu, Box::new(MmuWidget::new()));
        widgets.insert(TabKind::Breakpoints, Box::new(BreakpointsWidget::new()));

        Self {
            psx: Psx::new(BIOS),
            is_running: false,
            dock_state,
            breakpoints: HashSet::new(),
            breakpoint_hit: false,
            widgets,
            show_in_disassembly: None,
        }
    }
}

impl eframe::App for PsxDebugger {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.is_running && !self.breakpoint_hit {
            for _ in 0..100 {
                // Check for breakpoint before stepping
                if self.breakpoints.contains(&self.psx.cpu.pc) {
                    self.breakpoint_hit = true;
                    self.is_running = false;
                    break;
                }
                self.psx.step();
            }

            ctx.request_repaint_after(Duration::from_millis(16));
        }

        let mut tab_viewer = TabViewer {
            psx: &mut self.psx,
            is_running: &mut self.is_running,
            breakpoints: &mut self.breakpoints,
            breakpoint_hit: &mut self.breakpoint_hit,
            widgets: &mut self.widgets,
            show_in_disassembly: &mut self.show_in_disassembly,
        };

        DockArea::new(&mut self.dock_state).show(ctx, &mut tab_viewer);
    }
}

struct TabViewer<'a> {
    psx: &'a mut Psx,
    is_running: &'a mut bool,
    breakpoints: &'a mut HashSet<u32>,
    breakpoint_hit: &'a mut bool,
    widgets: &'a mut HashMap<TabKind, Box<dyn Widget>>,
    show_in_disassembly: &'a mut Option<u32>,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = TabKind;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        if let Some(widget) = self.widgets.get(tab) {
            widget.title().into()
        } else {
            tab.to_string().into()
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        if let Some(widget) = self.widgets.get_mut(tab) {
            let mut context = SharedContext {
                psx: self.psx,
                is_running: self.is_running,
                breakpoints: self.breakpoints,
                breakpoint_hit: self.breakpoint_hit,
                show_in_disassembly: self.show_in_disassembly,
            };
            widget.ui(ui, &mut context);
        }
    }
}

fn main() -> eframe::Result {
    let args = std::env::args().collect::<Vec<_>>();
    let tracing_level = if args.contains(&"--debug".to_string()) {
        tracing::Level::DEBUG
    } else if args.contains(&"--trace".to_string()) {
        tracing::Level::TRACE
    } else {
        tracing::Level::INFO
    };

    let mut targets = tracing_subscriber::filter::Targets::new();
    targets = targets.with_target("psx_core::cpu", tracing_level);
    targets = targets.with_target("psx_core::mmu", tracing_level);

    let fmt_layer = tracing_subscriber::fmt::layer()
        .without_time()
        .with_filter(targets);
    tracing_subscriber::registry().with(fmt_layer).init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("pspsps - psx debugger"),
        ..Default::default()
    };

    eframe::run_native(
        "pspsps - psx debugger",
        options,
        Box::new(|_cc| Ok(Box::new(PsxDebugger::default()))),
    )
}
