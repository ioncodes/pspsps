pub mod rect;
pub mod poly;

use crate::gpu::cmd::{poly::DrawPolygonCommand, rect::DrawRectangleCommand};

#[derive(PartialEq, Eq)]
pub enum Gp0Command {
    Misc,
    PolygonPrimitive(DrawPolygonCommand),
    LinePrimitive,
    RectanglePrimitive(DrawRectangleCommand),
    VramToVramBlit,
    CpuToVramBlit,
    VramToCpuBlit,
    Environment,
}

impl From<u32> for Gp0Command {
    fn from(value: u32) -> Self {
        match (value >> 29) & 0b111 {
            0b000 => Gp0Command::Misc,
            0b001 => Gp0Command::PolygonPrimitive(DrawPolygonCommand(value)),
            0b010 => Gp0Command::LinePrimitive,
            0b011 => Gp0Command::RectanglePrimitive(DrawRectangleCommand(value)),
            0b100 => Gp0Command::VramToVramBlit,
            0b101 => Gp0Command::CpuToVramBlit,
            0b110 => Gp0Command::VramToCpuBlit,
            0b111 => Gp0Command::Environment,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for Gp0Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Gp0Command::Misc => "Misc. Commands",
            Gp0Command::PolygonPrimitive(_) => "Polygon Primitive",
            Gp0Command::LinePrimitive => "Line Primitive",
            Gp0Command::RectanglePrimitive(_) => "Rectangle Primitive",
            Gp0Command::VramToVramBlit => "VRAM to VRAM Blit",
            Gp0Command::CpuToVramBlit => "CPU to VRAM Blit",
            Gp0Command::VramToCpuBlit => "VRAM to CPU Blit",
            Gp0Command::Environment => "Environment Commands",
        };
        write!(f, "{}", name)
    }
}

impl Gp0Command {
    pub fn base_extra_data_count(&self) -> usize {
        match self {
            Gp0Command::RectanglePrimitive(cmd) => {
                let mut base = 1;

                // requires UV
                if cmd.textured() {
                    base += 1;
                }

                // variable sized rectangles
                if cmd.size() == 0 {
                    base += 1;
                }

                base
            }
            Gp0Command::PolygonPrimitive(cmd) => {
                let mut base = if cmd.vertices_count() {
                    4
                } else {
                    3
                };

                // requires color
                if cmd.gouraud() {
                    base += 1;
                }

                // requires UV
                if cmd.textured() {
                    base += 1;
                }

                base
            }
            Gp0Command::VramToVramBlit => 3,
            Gp0Command::CpuToVramBlit => 2,
            Gp0Command::VramToCpuBlit => 2,
            _ => 0,
        }
    }
}
