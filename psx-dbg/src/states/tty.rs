pub struct TtyState {
    pub buffer: String,
}

impl Default for TtyState {
    fn default() -> Self {
        Self {
            buffer: String::new(),
        }
    }
}
