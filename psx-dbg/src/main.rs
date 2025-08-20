mod widgets;

use clap::Parser;
use eframe::egui;
use egui_dock::{DockArea, DockState};
use egui_toast::{Toast, ToastKind, Toasts};
use psx_core::cpu::decoder::Instruction;
use psx_core::psx::Psx;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Duration;
use tracing_subscriber::Layer as _;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use widgets::{
    BreakpointsWidget, CpuWidget, MmuWidget, SharedContext, TraceWidget, TtyWidget, Widget,
};

static BIOS: &[u8] = include_bytes!("../../bios/SCPH1000.BIN");

#[derive(Parser, Debug)]
#[command(about = "pspsps - a cute psx debugger", long_about = None)]
struct Args {
    #[arg(long, value_delimiter = ',', help = "List of tracing targets")]
    targets: Option<Vec<String>>,

    #[arg(long, help = "Path to EXE to sideload")]
    sideload: Option<String>,

    #[arg(long, help = "Enable debug logging")]
    debug: bool,

    #[arg(long, help = "Enable trace logging")]
    trace: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum TabKind {
    Cpu,
    Trace,
    Mmu,
    Breakpoints,
    Tty,
}

impl std::fmt::Display for TabKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TabKind::Cpu => write!(f, "CPU"),
            TabKind::Trace => write!(f, "Trace"),
            TabKind::Mmu => write!(f, "MMU"),
            TabKind::Breakpoints => write!(f, "Breakpoints"),
            TabKind::Tty => write!(f, "TTY"),
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
    toasts: Toasts,
    trace_buffer: VecDeque<(u32, Instruction)>,
}

impl Default for PsxDebugger {
    fn default() -> Self {
        Self::new(None)
    }
}

impl PsxDebugger {
    fn new(sideload_file: Option<String>) -> Self {
        let mut dock_state = DockState::new(vec![TabKind::Cpu, TabKind::Trace]);
        let [_old, mmu_node] = dock_state.main_surface_mut().split_right(
            egui_dock::NodeIndex::root(),
            0.5,
            vec![TabKind::Mmu],
        );
        dock_state.main_surface_mut().split_below(
            mmu_node,
            0.5,
            vec![TabKind::Breakpoints, TabKind::Tty],
        );

        let mut widgets: HashMap<TabKind, Box<dyn Widget>> = HashMap::new();
        widgets.insert(TabKind::Cpu, Box::new(CpuWidget::new()));
        widgets.insert(TabKind::Trace, Box::new(TraceWidget::new()));
        widgets.insert(TabKind::Mmu, Box::new(MmuWidget::new()));
        widgets.insert(TabKind::Breakpoints, Box::new(BreakpointsWidget::new()));
        widgets.insert(TabKind::Tty, Box::new(TtyWidget::new()));

        let mut psx = Psx::new(BIOS);

        // Sideload EXE if provided
        if let Some(file_path) = sideload_file {
            match std::fs::read(&file_path) {
                Ok(exe_data) => psx.sideload_exe(exe_data),
                Err(e) => panic!("Failed to read EXE file {}: {}", file_path, e),
            }
        }

        Self {
            psx,
            is_running: false,
            dock_state,
            breakpoints: HashSet::new(),
            breakpoint_hit: false,
            widgets,
            show_in_disassembly: None,
            toasts: Toasts::new()
                .anchor(egui::Align2::RIGHT_BOTTOM, (-10.0, -10.0))
                .direction(egui::Direction::BottomUp),
            trace_buffer: VecDeque::new(),
        }
    }
}

impl eframe::App for PsxDebugger {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.is_running && !self.breakpoint_hit {
            for _ in 0..1000 {
                if self.breakpoints.contains(&self.psx.cpu.pc) {
                    self.breakpoint_hit = true;
                    self.is_running = false;

                    self.toasts.add(Toast {
                        text: format!("Breakpoint hit at {:08X}", self.psx.cpu.pc).into(),
                        kind: ToastKind::Info,
                        options: egui_toast::ToastOptions::default()
                            .duration(Some(Duration::from_secs(3))),
                        style: Default::default(),
                    });

                    break;
                }

                let pc_before = self.psx.cpu.pc;
                let instruction = self.psx.step();

                // Add to trace buffer with limit of 1000
                if self.trace_buffer.len() >= 1000 {
                    self.trace_buffer.pop_front();
                }
                self.trace_buffer.push_back((pc_before, instruction));
            }

            ctx.request_repaint();
        }

        let mut tab_viewer = TabViewer {
            psx: &mut self.psx,
            is_running: &mut self.is_running,
            breakpoints: &mut self.breakpoints,
            breakpoint_hit: &mut self.breakpoint_hit,
            widgets: &mut self.widgets,
            show_in_disassembly: &mut self.show_in_disassembly,
            trace_buffer: &mut self.trace_buffer,
            toasts: &mut self.toasts,
        };

        DockArea::new(&mut self.dock_state).show(ctx, &mut tab_viewer);
        self.toasts.show(ctx);
    }
}

struct TabViewer<'a> {
    psx: &'a mut Psx,
    is_running: &'a mut bool,
    breakpoints: &'a mut HashSet<u32>,
    breakpoint_hit: &'a mut bool,
    widgets: &'a mut HashMap<TabKind, Box<dyn Widget>>,
    show_in_disassembly: &'a mut Option<u32>,
    trace_buffer: &'a mut VecDeque<(u32, Instruction)>,
    toasts: &'a mut Toasts,
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
                trace_buffer: self.trace_buffer,
                toasts: self.toasts,
            };
            widget.ui(ui, &mut context);
        }
    }
}

fn main() -> eframe::Result {
    let args = Args::parse();

    let tracing_level = if args.debug {
        tracing::Level::DEBUG
    } else if args.trace {
        tracing::Level::TRACE
    } else {
        tracing::Level::INFO
    };

    let mut targets = tracing_subscriber::filter::Targets::new();

    // Use custom targets if provided, otherwise use defaults
    if let Some(custom_targets) = &args.targets {
        for target in custom_targets {
            let full_target = format!("psx_core::{}", target);
            targets = targets.with_target(full_target, tracing_level);
        }
    } else {
        targets = targets.with_target("psx_core::cpu", tracing_level);
        targets = targets.with_target("psx_core::mmu", tracing_level);
        targets = targets.with_target("psx_core::tty", tracing_level);
        targets = targets.with_target("psx_core::bios", tracing_level);
    }

    let fmt_layer = tracing_subscriber::fmt::layer()
        .without_time()
        .with_filter(targets);
    tracing_subscriber::registry().with(fmt_layer).init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("pspsps - a cute psx debugger"),
        ..Default::default()
    };

    eframe::run_native(
        "pspsps - a cute psx debugger",
        options,
        Box::new(|_cc| Ok(Box::new(PsxDebugger::new(args.sideload)))),
    )
}
