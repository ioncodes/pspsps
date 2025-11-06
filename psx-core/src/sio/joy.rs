#[derive(Clone, Copy, Debug, Default)]
pub struct ControllerState {
    // D-Pad
    pub d_up: bool,
    pub d_down: bool,
    pub d_left: bool,
    pub d_right: bool,

    // Action buttons
    pub cross: bool,
    pub circle: bool,
    pub square: bool,
    pub triangle: bool,

    // Shoulder buttons
    pub l1: bool,
    pub l2: bool,
    pub r1: bool,
    pub r2: bool,

    // System buttons
    pub start: bool,
    pub select: bool,
}

impl ControllerState {
    // Convert button states to PlayStation button format (0 = pressed)
    pub fn to_button_bytes(&self) -> (u8, u8) {
        let byte1 =
            (!self.select as u8) << 0 |
            (1 << 1) |
            (1 << 2) |
            (!self.start as u8) << 3 |
            (!self.d_up as u8) << 4 |
            (!self.d_right as u8) << 5 |
            (!self.d_down as u8) << 6 |
            (!self.d_left as u8) << 7;

        let byte2 =
            (!self.l2 as u8) << 0 |
            (!self.r2 as u8) << 1 |
            (!self.l1 as u8) << 2 |
            (!self.r1 as u8) << 3 |
            (!self.triangle as u8) << 4 |
            (!self.circle as u8) << 5 |
            (!self.cross as u8) << 6 |
            (!self.square as u8) << 7;

        (byte1, byte2)
    }
}
