#[derive(Clone, Default)]
pub struct TimersState {
    // Timer 0
    pub timer0_counter: u16,
    pub timer0_target: u16,
    pub timer0_mode: u32,

    // Timer 1
    pub timer1_counter: u16,
    pub timer1_target: u16,
    pub timer1_mode: u32,

    // Timer 2
    pub timer2_counter: u16,
    pub timer2_target: u16,
    pub timer2_mode: u32,
}
