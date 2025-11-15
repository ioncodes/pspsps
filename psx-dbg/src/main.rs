mod colors;
mod debugger;
mod io;
mod states;
mod widgets;

use clap::Parser;
use eframe::egui;
use egui_dock::{DockArea, DockState};
use egui_toast::{Toast, ToastKind, Toasts};
use psx_core::sio::joy::ControllerState;
use std::collections::HashMap;
use std::time::Duration;
use tracing_subscriber::Layer as _;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use widgets::{
    BreakpointsWidget, CdromWidget, CopWidget, CpuWidget, DisplayWidget, DmaWidget, GpuWidget, MmuWidget,
    SharedContext, TimersWidget, TraceWidget, TtyWidget, Widget,
};

use crate::debugger::Debugger;
use crate::io::DebuggerEvent;

#[derive(Parser, Debug)]
#[command(about = "pspsps - a cute psx debugger", long_about = None)]
struct Args {
    #[arg(long, help = "Path to BIOS file (required)")]
    bios: String,

    #[arg(long, help = "Path to CDROM image")]
    cdrom: Option<String>,

    #[arg(long, help = "Path to EXE to sideload")]
    sideload: Option<String>,

    #[arg(long, value_delimiter = ',', help = "List of tracing targets")]
    log_targets: Option<Vec<String>>,

    #[arg(long, help = "Enable debug logging")]
    debug: bool,

    #[arg(long, help = "Enable trace logging")]
    trace: bool,

    #[arg(long, help = "Enable json logging")]
    json: bool,

    #[arg(long, help = "Disable ANSI colors in the terminal output")]
    no_colors: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum TabKind {
    Cpu,
    Trace,
    Mmu,
    Breakpoints,
    Cop,
    Tty,
    Gpu,
    Display,
    Timers,
    Cdrom,
    Dma,
}

impl std::fmt::Display for TabKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TabKind::Cpu => write!(f, "CPU"),
            TabKind::Trace => write!(f, "Trace"),
            TabKind::Mmu => write!(f, "MMU"),
            TabKind::Breakpoints => write!(f, "Breakpoints"),
            TabKind::Cop => write!(f, "COP"),
            TabKind::Tty => write!(f, "TTY"),
            TabKind::Gpu => write!(f, "GPU"),
            TabKind::Display => write!(f, "Display"),
            TabKind::Timers => write!(f, "Timers"),
            TabKind::Cdrom => write!(f, "CDROM"),
            TabKind::Dma => write!(f, "DMA"),
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
        panic!("BIOS path is required! Use PsxDebugger::new() instead.")
    }
}

impl PsxDebugger {
    fn new(bios_path: String, sideload_file: Option<String>, cdrom_file: Option<String>) -> Self {
        let mut dock_state = DockState::new(vec![TabKind::Cpu, TabKind::Trace]);
        let [left_node, right_node] =
            dock_state
                .main_surface_mut()
                .split_right(egui_dock::NodeIndex::root(), 0.33, vec![TabKind::Gpu]);

        dock_state
            .main_surface_mut()
            .split_below(left_node, 0.8, vec![TabKind::Breakpoints]);

        let [gpu_node, mmu_node] = dock_state.main_surface_mut().split_right(
            right_node,
            0.5,
            vec![TabKind::Mmu, TabKind::Timers, TabKind::Cdrom, TabKind::Dma],
        );
        dock_state
            .main_surface_mut()
            .split_below(gpu_node, 0.66, vec![TabKind::Display]);
        dock_state
            .main_surface_mut()
            .split_below(mmu_node, 0.5, vec![TabKind::Cop, TabKind::Tty]);

        let mut widgets: HashMap<TabKind, Box<dyn Widget>> = HashMap::new();
        widgets.insert(TabKind::Cpu, Box::new(CpuWidget::new()));
        widgets.insert(TabKind::Trace, Box::new(TraceWidget::new()));
        widgets.insert(TabKind::Mmu, Box::new(MmuWidget::new()));
        widgets.insert(TabKind::Breakpoints, Box::new(BreakpointsWidget::new()));
        widgets.insert(TabKind::Cop, Box::new(CopWidget::new()));
        widgets.insert(TabKind::Tty, Box::new(TtyWidget::new()));
        widgets.insert(TabKind::Gpu, Box::new(GpuWidget::new()));
        widgets.insert(TabKind::Display, Box::new(DisplayWidget::new()));
        widgets.insert(TabKind::Timers, Box::new(TimersWidget::new()));
        widgets.insert(TabKind::Cdrom, Box::new(CdromWidget::new()));
        widgets.insert(TabKind::Dma, Box::new(DmaWidget::new()));

        let (request_channel_send, request_channel_recv) = crossbeam_channel::unbounded();
        let (response_channel_send, response_channel_recv) = crossbeam_channel::unbounded();

        let thread = std::thread::spawn(move || {
            let mut debugger = Debugger::new(bios_path, response_channel_send, request_channel_recv)
                .with_sideloaded_exe(sideload_file)
                .with_cdrom_image(cdrom_file);
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
        let mut controller = ControllerState::default();

        ctx.input(|i| {
            // D-Pad (Arrow keys)
            controller.d_up = i.key_down(egui::Key::ArrowUp);
            controller.d_down = i.key_down(egui::Key::ArrowDown);
            controller.d_left = i.key_down(egui::Key::ArrowLeft);
            controller.d_right = i.key_down(egui::Key::ArrowRight);

            // Action buttons
            controller.cross = i.key_down(egui::Key::Y); // Y = Cross
            controller.circle = i.key_down(egui::Key::X); // X = Circle
            controller.square = i.key_down(egui::Key::A); // A = Square
            controller.triangle = i.key_down(egui::Key::S); // S = Triangle

            // Shoulder buttons
            controller.l1 = i.key_down(egui::Key::Q); // Q = L1
            controller.l2 = i.key_down(egui::Key::W); // W = L2
            controller.r1 = i.key_down(egui::Key::E); // E = R1
            controller.r2 = i.key_down(egui::Key::R); // R = R2

            // System buttons
            controller.start = i.key_down(egui::Key::Enter); // Enter = Start
            controller.select = i.key_down(egui::Key::Space); // Space = Select
        });

        let _ = self.channel_send.send(DebuggerEvent::UpdateController(controller));

        while let Ok(event) = self.channel_recv.try_recv() {
            match event {
                DebuggerEvent::BreakpointHit(addr) => {
                    self.state.is_running = false;

                    self.toasts.add(Toast {
                        text: format!("Breakpoint hit at {:08X}", addr).into(),
                        kind: ToastKind::Info,
                        options: egui_toast::ToastOptions::default().duration(Some(Duration::from_secs(3))),
                        style: Default::default(),
                    });
                }
                DebuggerEvent::TraceUpdated(state) => {
                    self.state.trace = state;
                }
                DebuggerEvent::CpuUpdated(state) => {
                    // Only update previous state when flagged (running or just stepped)
                    // This prevents change highlighting from disappearing when paused
                    if self.state.should_update_previous_cpu {
                        self.state.previous_cpu = self.state.cpu.clone();
                        self.state.should_update_previous_cpu = false;
                    }
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
                DebuggerEvent::TimersUpdated(state) => {
                    self.state.timers = state;
                }
                DebuggerEvent::CdromUpdated(state) => {
                    self.state.cdrom = state;
                }
                DebuggerEvent::DmaUpdated(state) => {
                    self.state.dma = state;
                }
                DebuggerEvent::Paused => {
                    self.state.is_running = false;
                    self.toasts.add(Toast {
                        text: "Debugger paused".into(),
                        kind: ToastKind::Info,
                        options: egui_toast::ToastOptions::default().duration(Some(Duration::from_secs(3))),
                        style: Default::default(),
                    });
                }
                DebuggerEvent::Unpaused => {
                    self.state.is_running = true;
                    self.toasts.add(Toast {
                        text: "Debugger running".into(),
                        kind: ToastKind::Info,
                        options: egui_toast::ToastOptions::default().duration(Some(Duration::from_secs(3))),
                        style: Default::default(),
                    });
                }
                _ => {}
            }
        }

        // Set flag to update previous_cpu when running (for change tracking)
        if self.state.is_running {
            self.state.should_update_previous_cpu = true;
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
        self.channel_send
            .send(DebuggerEvent::UpdateTimers)
            .expect("Failed to send update Timers event");
        self.channel_send
            .send(DebuggerEvent::UpdateCdrom)
            .expect("Failed to send update CDROM event");
        self.channel_send
            .send(DebuggerEvent::UpdateDma)
            .expect("Failed to send update DMA event");

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
    if let Some(custom_targets) = &args.log_targets {
        for target in custom_targets {
            let full_target = format!("psx_core::{}", target);
            targets = targets.with_target(full_target, tracing_level);
        }
    } else {
        targets = targets.with_target("psx_core::cpu", tracing_level);
        targets = targets.with_target("psx_core::mmu", tracing_level);
        targets = targets.with_target("psx_core::dma", tracing_level);
        targets = targets.with_target("psx_core::tty", tracing_level);
        targets = targets.with_target("psx_core::bios", tracing_level);
        targets = targets.with_target("psx_core::gpu", tracing_level);
        targets = targets.with_target("psx_core::irq", tracing_level);
        targets = targets.with_target("psx_core::sio", tracing_level);
        targets = targets.with_target("psx_core::spu", tracing_level);
        targets = targets.with_target("psx_core::joy", tracing_level);
        targets = targets.with_target("psx_core::mc", tracing_level);
        targets = targets.with_target("psx_core::cdrom", tracing_level);
    }

    if args.json {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .without_time()
            .with_ansi(!args.no_colors)
            .json()
            .with_filter(targets);
        tracing_subscriber::registry().with(fmt_layer).init();
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .without_time()
            .with_ansi(!args.no_colors)
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
        Box::new(|_cc| Ok(Box::new(PsxDebugger::new(args.bios, args.sideload, args.cdrom)))),
    )
}
