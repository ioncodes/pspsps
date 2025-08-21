use std::collections::HashSet;

pub struct BreakpointsState {
    pub breakpoints: HashSet<u32>,
}

impl Default for BreakpointsState {
    fn default() -> Self {
        Self {
            breakpoints: HashSet::new(),
        }
    }
}
