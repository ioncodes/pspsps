use eframe::egui;
use egui_dock::{DockArea, DockState};
use psx_core::cpu::decoder::Instruction;
use psx_core::psx::Psx;
use std::time::Duration;
use tracing_subscriber::Layer as _;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

static BIOS: &[u8] = include_bytes!("../../bios/SCPH1000.BIN");

#[derive(Clone, Copy, Debug, PartialEq)]
enum TabKind {
    Cpu,
    Mmu,
}

impl std::fmt::Display for TabKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TabKind::Cpu => write!(f, "CPU"),
            TabKind::Mmu => write!(f, "MMU"),
        }
    }
}

pub struct PsxDebugger {
    psx: Psx,
    is_running: bool,
    dock_state: DockState<TabKind>,
    memory_address: u32,
}

impl Default for PsxDebugger {
    fn default() -> Self {
        let mut dock_state = DockState::new(vec![TabKind::Cpu]);
        let [_old, _new] = dock_state.main_surface_mut().split_right(
            egui_dock::NodeIndex::root(),
            0.5,
            vec![TabKind::Mmu],
        );

        Self {
            psx: Psx::new(BIOS),
            is_running: false,
            dock_state,
            memory_address: 0x80000000,
        }
    }
}

impl eframe::App for PsxDebugger {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.is_running {
            for _ in 0..100 {
                self.psx.step();
            }

            ctx.request_repaint_after(Duration::from_millis(16));
        }

        let mut tab_viewer = TabViewer {
            psx: &mut self.psx,
            is_running: &mut self.is_running,
            memory_address: &mut self.memory_address,
        };

        DockArea::new(&mut self.dock_state).show(ctx, &mut tab_viewer);
    }
}

struct TabViewer<'a> {
    psx: &'a mut Psx,
    is_running: &'a mut bool,
    memory_address: &'a mut u32,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = TabKind;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.to_string().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            TabKind::Cpu => {
                ui.heading("Controls");
                ui.horizontal(|ui| {
                    if *self.is_running {
                        if ui.button("Pause").clicked() {
                            *self.is_running = false;
                        }
                    } else {
                        if ui.button("Run").clicked() {
                            *self.is_running = true;
                        }
                    }

                    if ui.button("Step").clicked() {
                        self.psx.step();
                    }

                    if ui.button("Reset").clicked() {
                        *self.psx = Psx::new(BIOS);
                        *self.is_running = false;
                    }
                });

                ui.separator();

                ui.heading("Registers");
                ui.monospace(format!(
                    "$00: {:08X}  $at: {:08X}  $v0: {:08X}  $v1: {:08X}",
                    self.psx.cpu.registers[0],
                    self.psx.cpu.registers[1],
                    self.psx.cpu.registers[2],
                    self.psx.cpu.registers[3]
                ));
                ui.monospace(format!(
                    "$a0: {:08X}  $a1: {:08X}  $a2: {:08X}  $a3: {:08X}",
                    self.psx.cpu.registers[4],
                    self.psx.cpu.registers[5],
                    self.psx.cpu.registers[6],
                    self.psx.cpu.registers[7]
                ));
                ui.monospace(format!(
                    "$t0: {:08X}  $t1: {:08X}  $t2: {:08X}  $t3: {:08X}",
                    self.psx.cpu.registers[8],
                    self.psx.cpu.registers[9],
                    self.psx.cpu.registers[10],
                    self.psx.cpu.registers[11]
                ));
                ui.monospace(format!(
                    "$t4: {:08X}  $t5: {:08X}  $t6: {:08X}  $t7: {:08X}",
                    self.psx.cpu.registers[12],
                    self.psx.cpu.registers[13],
                    self.psx.cpu.registers[14],
                    self.psx.cpu.registers[15]
                ));
                ui.monospace(format!(
                    "$s0: {:08X}  $s1: {:08X}  $s2: {:08X}  $s3: {:08X}",
                    self.psx.cpu.registers[16],
                    self.psx.cpu.registers[17],
                    self.psx.cpu.registers[18],
                    self.psx.cpu.registers[19]
                ));
                ui.monospace(format!(
                    "$s4: {:08X}  $s5: {:08X}  $s6: {:08X}  $s7: {:08X}",
                    self.psx.cpu.registers[20],
                    self.psx.cpu.registers[21],
                    self.psx.cpu.registers[22],
                    self.psx.cpu.registers[23]
                ));
                ui.monospace(format!(
                    "$t8: {:08X}  $t9: {:08X}  $k0: {:08X}  $k1: {:08X}",
                    self.psx.cpu.registers[24],
                    self.psx.cpu.registers[25],
                    self.psx.cpu.registers[26],
                    self.psx.cpu.registers[27]
                ));
                ui.monospace(format!(
                    "$gp: {:08X}  $sp: {:08X}  $fp: {:08X}  $ra: {:08X}",
                    self.psx.cpu.registers[28],
                    self.psx.cpu.registers[29],
                    self.psx.cpu.registers[30],
                    self.psx.cpu.registers[31]
                ));
                ui.monospace(format!("$pc: {:08X}", self.psx.cpu.pc));
                ui.monospace(format!("$hi: {:08X}", self.psx.cpu.hi));
                ui.monospace(format!("$lo: {:08X}", self.psx.cpu.lo));

                ui.separator();

                ui.heading("Disassembly");
                let start = self.psx.cpu.pc as usize;
                let end = start + 40 * 4;

                let instructions: Vec<(u32, Instruction)> = self.psx.mmu.memory[start..end]
                    .chunks(4)
                    .enumerate()
                    .map(|(i, chunk)| {
                        let addr = start + i * 4;
                        let instr_raw =
                            u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                        let instr = Instruction::decode(instr_raw);
                        (addr as u32, instr)
                    })
                    .collect();

                for (addr, instr) in instructions {
                    let line = format!("{:08X}: {}", addr, instr);
                    ui.monospace(line);
                }
            }
            TabKind::Mmu => {
                ui.heading("Memory Viewer");

                ui.horizontal(|ui| {
                    ui.label("Address:");
                    let mut addr_str = format!("{:08X}", *self.memory_address);
                    if ui.text_edit_singleline(&mut addr_str).changed() {
                        if let Ok(addr) = u32::from_str_radix(&addr_str, 16) {
                            *self.memory_address = addr;
                        }
                    }

                    if ui.button("Go to PC").clicked() {
                        *self.memory_address = self.psx.cpu.pc;
                    }
                });

                ui.separator();

                let start_addr = *self.memory_address & !0xF;
                let memory = &self.psx.mmu.memory;

                for row in 0..32 {
                    let addr = start_addr + (row * 16);
                    if addr as usize >= memory.len() {
                        break;
                    }

                    let mut line = format!("{:08X}: ", addr);

                    for col in 0..16 {
                        let byte_addr = addr + col;
                        if (byte_addr as usize) < memory.len() {
                            line.push_str(&format!("{:02X} ", memory[byte_addr as usize]));
                        } else {
                            line.push_str("?? ");
                        }

                        if col == 7 {
                            line.push(' ');
                        }
                    }

                    line.push_str(" |");
                    for col in 0..16 {
                        let byte_addr = addr + col;
                        if (byte_addr as usize) < memory.len() {
                            let byte = memory[byte_addr as usize];
                            if byte >= 32 && byte <= 126 {
                                line.push(byte as char);
                            } else {
                                line.push('.');
                            }
                        } else {
                            line.push('?');
                        }
                    }
                    line.push('|');

                    ui.monospace(line);
                }
            }
        }
    }
}

fn main() -> eframe::Result {
    let mut targets = tracing_subscriber::filter::Targets::new();
    targets = targets.with_target("psx_core::cpu", tracing::Level::DEBUG);

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
