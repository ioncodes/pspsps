mod debugger;
mod io;
mod states;
mod widgets;

use clap::Parser;
use eframe::egui;
use egui_dock::{DockArea, DockState};
use egui_toast::{Toast, ToastKind, Toasts};
use std::collections::HashMap;
use std::time::Duration;
use tracing_subscriber::Layer as _;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use widgets::{
    BreakpointsWidget, CpuWidget, DisplayWidget, GpuWidget, MmuWidget, SharedContext, TraceWidget,
    TtyWidget, Widget,
};

use crate::debugger::Debugger;
use crate::io::DebuggerEvent;

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

    #[arg(long, help = "Enable json logging")]
    json: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum TabKind {
    Cpu,
    Trace,
    Mmu,
    Breakpoints,
    Tty,
    Gpu,
    Display,
}

impl std::fmt::Display for TabKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TabKind::Cpu => write!(f, "CPU"),
            TabKind::Trace => write!(f, "Trace"),
            TabKind::Mmu => write!(f, "MMU"),
            TabKind::Breakpoints => write!(f, "Breakpoints"),
            TabKind::Tty => write!(f, "TTY"),
            TabKind::Gpu => write!(f, "GPU"),
            TabKind::Display => write!(f, "Display"),
        }
    }
}

pub struct PsxDebugger {
    _psx_thread: std::thread::JoinHandle<()>,
    channel_send: crossbeam_channel::Sender<DebuggerEvent>,
    channel_recv: crossbeam_channel::Receiver<DebuggerEvent>,
    state: states::State,
    dock_state: DockState<TabKind>,
    widgets: HashMap<TabKind, Box<dyn Widget>>,
    toasts: Toasts,
    show_in_disassembly: Option<u32>,
}

impl Default for PsxDebugger {
    fn default() -> Self {
        Self::new(None)
    }
}

impl PsxDebugger {
    fn new(sideload_file: Option<String>) -> Self {
        let mut dock_state = DockState::new(vec![TabKind::Cpu, TabKind::Trace]);
        let [_left_node, right_node] = dock_state.main_surface_mut().split_right(
            egui_dock::NodeIndex::root(),
            0.33,
            vec![TabKind::Gpu],
        );
        let [gpu_node, mmu_node] =
            dock_state
                .main_surface_mut()
                .split_right(right_node, 0.5, vec![TabKind::Mmu]);
        dock_state
            .main_surface_mut()
            .split_below(gpu_node, 0.66, vec![TabKind::Display]);
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
        widgets.insert(TabKind::Gpu, Box::new(GpuWidget::new()));
        widgets.insert(TabKind::Display, Box::new(DisplayWidget::new()));

        let (request_channel_send, request_channel_recv) = crossbeam_channel::unbounded();
        let (response_channel_send, response_channel_recv) = crossbeam_channel::unbounded();

        let thread = std::thread::spawn(move || {
            let mut debugger = Debugger::new(response_channel_send, request_channel_recv);

            // Sideload EXE if provided
            if let Some(file_path) = sideload_file {
                match std::fs::read(&file_path) {
                    Ok(exe_data) => debugger.sideload_exe(exe_data),
                    Err(e) => panic!("Failed to read EXE file {}: {}", file_path, e),
                }
            }

            debugger.run();
        });

        request_channel_send
            .send(DebuggerEvent::UpdateMmu)
            .expect("Failed to send initial MMU update request");

        Self {
            _psx_thread: thread,
            channel_send: request_channel_send,
            channel_recv: response_channel_recv,
            dock_state,
            widgets,
            state: states::State::new(),
            toasts: Toasts::new()
                .anchor(egui::Align2::RIGHT_BOTTOM, (-10.0, -10.0))
                .direction(egui::Direction::BottomUp),
            show_in_disassembly: None,
        }
    }
}

impl eframe::App for PsxDebugger {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(event) = self.channel_recv.try_recv() {
            match event {
                DebuggerEvent::BreakpointHit(addr) => {
                    self.state.is_running = false;

                    self.toasts.add(Toast {
                        text: format!("Breakpoint hit at {:08X}", addr).into(),
                        kind: ToastKind::Info,
                        options: egui_toast::ToastOptions::default()
                            .duration(Some(Duration::from_secs(3))),
                        style: Default::default(),
                    });
                }
                DebuggerEvent::TraceUpdated(state) => {
                    self.state.trace = state;
                }
                DebuggerEvent::CpuUpdated(state) => {
                    self.state.cpu = state;
                }
                DebuggerEvent::MmuUpdated(state) => {
                    self.state.mmu = state;
                }
                DebuggerEvent::BreakpointsUpdated(state) => {
                    self.state.breakpoints = state;
                }
                DebuggerEvent::TtyUpdated(state) => {
                    self.state.tty = state;
                }
                DebuggerEvent::GpuUpdated(state) => {
                    self.state.gpu = state;
                }
                DebuggerEvent::Paused => {
                    self.state.is_running = false;
                    self.toasts.add(Toast {
                        text: "Debugger paused".into(),
                        kind: ToastKind::Info,
                        options: egui_toast::ToastOptions::default()
                            .duration(Some(Duration::from_secs(3))),
                        style: Default::default(),
                    });
                }
                DebuggerEvent::Unpaused => {
                    self.state.is_running = true;
                    self.toasts.add(Toast {
                        text: "Debugger running".into(),
                        kind: ToastKind::Info,
                        options: egui_toast::ToastOptions::default()
                            .duration(Some(Duration::from_secs(3))),
                        style: Default::default(),
                    });
                }
                _ => {}
            }
        }

        self.channel_send
            .send(DebuggerEvent::UpdateCpu)
            .expect("Failed to send update CPU event");
        self.channel_send
            .send(DebuggerEvent::UpdateTty)
            .expect("Failed to send update TTY event");
        self.channel_send
            .send(DebuggerEvent::UpdateTrace)
            .expect("Failed to send update Trace event");

        let mut tab_viewer = TabViewer {
            channel_send: &self.channel_send,
            widgets: &mut self.widgets,
            toasts: &mut self.toasts,
            state: &mut self.state,
            show_in_disassembly: &mut self.show_in_disassembly,
        };

        DockArea::new(&mut self.dock_state).show(ctx, &mut tab_viewer);

        self.toasts.show(ctx);

        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

struct TabViewer<'a> {
    channel_send: &'a crossbeam_channel::Sender<DebuggerEvent>,
    widgets: &'a mut HashMap<TabKind, Box<dyn Widget>>,
    toasts: &'a mut Toasts,
    state: &'a mut states::State,
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
                channel_send: self.channel_send,
                state: self.state,
                toasts: self.toasts,
                show_in_disassembly: self.show_in_disassembly,
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
        targets = targets.with_target("psx_core::gpu", tracing_level);
    }

    if args.json {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .without_time()
            .json()
            .with_filter(targets);
        tracing_subscriber::registry().with(fmt_layer).init();
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .without_time()
            .with_filter(targets);
        tracing_subscriber::registry().with(fmt_layer).init();
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1700.0, 900.0])
            //.with_maximized(true)
            .with_title("pspsps - a cute psx debugger"),
        ..Default::default()
    };

    eframe::run_native(
        "pspsps - a cute psx debugger",
        options,
        Box::new(|_cc| Ok(Box::new(PsxDebugger::new(args.sideload)))),
    )
}
