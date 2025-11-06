use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Task, Theme};
use psx_core::cpu::decoder::Instruction;
use psx_core::psx::Psx;

static BIOS: &[u8] = include_bytes!("../../bios/SCPH1000.BIN");

macro_rules! monospace_text {
    ($text:expr) => {
        text($text)
            .font(iced::Font {
                family: iced::font::Family::Monospace,
                ..iced::Font::default()
            })
            .size(14)
    };
    () => {};
}

#[derive(Debug, Clone)]
pub enum Message {
    Step,
    Reset,
}

pub struct PsxDebugger {
    psx: Psx,
}

impl Default for PsxDebugger {
    fn default() -> Self {
        Self {
            psx: Psx::new(BIOS),
        }
    }
}

impl PsxDebugger {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Step => self.psx.step(),
            Message::Reset => {
                self.psx = Psx::new(BIOS);
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            button("Step").on_press(Message::Step),
            button("Reset").on_press(Message::Reset),
        ]
        .spacing(10)
        .padding(10);

        let cpu_state = column![
            text("Registers")
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..iced::Font::default()
                })
                .size(24),
            monospace_text!(format!(
                "$00: {:08X}  $at: {:08X}  $v0: {:08X}  $v1: {:08X}",
                self.psx.cpu.registers[0],
                self.psx.cpu.registers[1],
                self.psx.cpu.registers[2],
                self.psx.cpu.registers[3]
            )),
            monospace_text!(format!(
                "$a0: {:08X}  $a1: {:08X}  $a2: {:08X}  $a3: {:08X}",
                self.psx.cpu.registers[4],
                self.psx.cpu.registers[5],
                self.psx.cpu.registers[6],
                self.psx.cpu.registers[7]
            )),
            monospace_text!(format!(
                "$t0: {:08X}  $t1: {:08X}  $t2: {:08X}  $t3: {:08X}",
                self.psx.cpu.registers[8],
                self.psx.cpu.registers[9],
                self.psx.cpu.registers[10],
                self.psx.cpu.registers[11]
            )),
            monospace_text!(format!(
                "$t4: {:08X}  $t5: {:08X}  $t6: {:08X}  $t7: {:08X}",
                self.psx.cpu.registers[12],
                self.psx.cpu.registers[13],
                self.psx.cpu.registers[14],
                self.psx.cpu.registers[15]
            )),
            monospace_text!(format!(
                "$t8: {:08X}  $t9: {:08X}  $k0: {:08X}  $k1: {:08X}",
                self.psx.cpu.registers[24],
                self.psx.cpu.registers[25],
                self.psx.cpu.registers[26],
                self.psx.cpu.registers[27]
            )),
            monospace_text!(format!(
                "$k0: {:08X}  $k1: {:08X}  $gp: {:08X}  $sp: {:08X}",
                self.psx.cpu.registers[28],
                self.psx.cpu.registers[29],
                self.psx.cpu.registers[30],
                self.psx.cpu.registers[31]
            )),
            monospace_text!(format!(
                "$fp: {:08X}  $ra: {:08X}  $hi: {:08X}  $lo: {:08X}",
                self.psx.cpu.registers[30],
                self.psx.cpu.registers[31],
                self.psx.cpu.hi,
                self.psx.cpu.lo
            )),
            monospace_text!(format!("$pc: {:08X}", self.psx.cpu.pc)),
        ]
        .spacing(5);

        let disassembly = {
            let mut items = Vec::new();

            let start = self.psx.cpu.pc as usize;
            let end = start + 40 * 4;

            let instructions = &self.psx.mmu.memory[start..end]
                .chunks(4)
                .enumerate()
                .map(|(i, chunk)| {
                    let addr = start + i * 4;
                    let instr_raw = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    let instr = Instruction::decode(instr_raw);
                    (addr as u32, instr)
                })
                .collect::<Vec<_>>();

            for (addr, instr) in instructions {
                let line = format!("{:08X}: {}", addr, instr);
                let text = monospace_text!(line);
                items.push(text.into());
            }

            scrollable(column(items).spacing(2))
        };

        let main_content = column![
            container(cpu_state).padding(10),
            container(
                column![
                    text("Disassembly")
                        .font(iced::Font {
                            weight: iced::font::Weight::Bold,
                            ..iced::Font::default()
                        })
                        .size(24),
                    disassembly.width(iced::Length::Fill)
                ]
                .spacing(5)
            )
            .padding(10)
        ];

        container(column![controls, main_content].spacing(10))
            .padding(10)
            .into()
    }
}

fn main() -> iced::Result {
    iced::application(
        "pspsps - psx debugger",
        PsxDebugger::update,
        PsxDebugger::view,
    )
    .theme(|_| Theme::Dark)
    .centered()
    .run()
}
