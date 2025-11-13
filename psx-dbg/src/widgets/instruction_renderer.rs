use egui::{RichText, Ui};
use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;
use psx_core::cpu::decoder::Instruction;
use crate::colors::*;

#[derive(Parser)]
#[grammar = "grammar/instruction.pest"]
pub struct InstructionParser;

pub fn render_instruction(ui: &mut Ui, addr: u32, instr: &Instruction, is_pc: bool, has_breakpoint: bool) {
    let (response, painter) = ui.allocate_painter(
        egui::Vec2::new(16.0, ui.text_style_height(&egui::TextStyle::Monospace)),
        egui::Sense::hover(),
    );
    let rect = response.rect;
    let center = rect.center();

    if has_breakpoint {
        // Red circle for breakpoint
        painter.circle_filled(center, 4.0, COLOR_BREAKPOINT);
    } else if is_pc {
        // Green triangle pointing right for PC
        let triangle = vec![
            egui::pos2(center.x - 4.0, center.y - 4.0),
            egui::pos2(center.x + 4.0, center.y),
            egui::pos2(center.x - 4.0, center.y + 4.0),
        ];
        painter.add(egui::Shape::convex_polygon(
            triangle,
            COLOR_CURRENT_LOCATION,
            egui::Stroke::NONE,
        ));
    }

    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;

        // Address
        ui.colored_label(COLOR_ADDRESS, RichText::new(format!("{:08X}", addr)).monospace());
        ui.label(RichText::new(" ").monospace());

        // Raw instruction hex (use opcode color)
        ui.colored_label(COLOR_COP, RichText::new(format!("{:08X}", instr.raw)).monospace());
        ui.label(RichText::new(" ").monospace());

        // Parse the instruction string with PEG
        let instr_str = format!("{}", instr);

        let parsed = InstructionParser::parse(Rule::instruction, &instr_str);

        match parsed {
            Ok(mut pairs) => {
                if let Some(instruction_pair) = pairs.next() {
                    let mut first_operand = true;

                    for pair in instruction_pair.into_inner() {
                        match pair.as_rule() {
                            Rule::opcode => {
                                let opcode_color = if has_breakpoint { COLOR_BREAKPOINT } else { COLOR_OPCODE };
                                ui.colored_label(
                                    opcode_color,
                                    RichText::new(format!("{:<8}", pair.as_str())).monospace(),
                                );
                            }
                            Rule::operands => {
                                for operand in pair.into_inner() {
                                    if !first_operand {
                                        ui.label(RichText::new(", ").monospace());
                                    }
                                    first_operand = false;

                                    render_operand(ui, operand);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(_) => {
                ui.label(RichText::new(&instr_str).monospace());
            }
        }
    });
}

fn render_operand(ui: &mut Ui, operand: Pair<Rule>) {
    match operand.as_rule() {
        Rule::operand => {
            if let Some(inner) = operand.into_inner().next() {
                render_operand(ui, inner);
            }
        }
        Rule::cpu_register => {
            ui.colored_label(COLOR_REGISTER, RichText::new(operand.as_str()).monospace());
        }
        Rule::cop_register | Rule::gte_parameter => {
            ui.colored_label(COLOR_COP, RichText::new(operand.as_str()).monospace());
        }
        Rule::hex_immediate | Rule::decimal_immediate | Rule::offset => {
            ui.colored_label(COLOR_IMMEDIATE, RichText::new(operand.as_str()).monospace());
        }
        Rule::memory_address => {
            // Memory address: offset($register)
            for inner in operand.into_inner() {
                match inner.as_rule() {
                    Rule::hex_immediate | Rule::decimal_immediate | Rule::offset => {
                        ui.colored_label(COLOR_IMMEDIATE, RichText::new(inner.as_str()).monospace());
                    }
                    Rule::cpu_register => {
                        ui.label(RichText::new("(").monospace());
                        ui.colored_label(COLOR_REGISTER, RichText::new(inner.as_str()).monospace());
                        ui.label(RichText::new(")").monospace());
                    }
                    _ => {}
                }
            }
        }
        _ => {
            eprintln!(
                "Unknown rule in render_operand: {:?} ({})",
                operand.as_rule(),
                operand.as_str()
            );
            ui.label(RichText::new(operand.as_str()).monospace());
        }
    }
}
