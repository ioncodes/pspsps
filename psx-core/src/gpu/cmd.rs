pub enum Gp0Command {
    Misc,
    PolygonPrimitive,
    LinePrimitive,
    RectanglePrimitive,
    VramToVramBlit,
    CpuToVramBlit,
    VramToCpuBlit,
    Environment,
}

impl Gp0Command {
    pub fn extra_data(&self) -> usize {
        match self {
            Gp0Command::RectanglePrimitive => 3,
            _ => 0,
        }
    }
}

impl From<u32> for Gp0Command {
    fn from(value: u32) -> Self {
        match (value >> 29) & 0b111 {
            0b000 => Gp0Command::Misc,
            0b001 => Gp0Command::PolygonPrimitive,
            0b010 => Gp0Command::LinePrimitive,
            0b011 => Gp0Command::RectanglePrimitive,
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
            Gp0Command::PolygonPrimitive => "Polygon Primitive",
            Gp0Command::LinePrimitive => "Line Primitive",
            Gp0Command::RectanglePrimitive => "Rectangle Primitive",
            Gp0Command::VramToVramBlit => "VRAM to VRAM Blit",
            Gp0Command::CpuToVramBlit => "CPU to VRAM Blit",
            Gp0Command::VramToCpuBlit => "VRAM to CPU Blit",
            Gp0Command::Environment => "Environment Commands",
        };
        write!(f, "{}", name)
    }
}
