#[derive(Clone, Default)]
pub struct CdromState {
    pub drive_state: String,
    pub sector_lba: usize,
    pub sector_lba_current: usize,
    pub last_command: u8,
    pub read_in_progress: bool,
}
