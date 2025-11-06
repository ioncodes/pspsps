#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskIrq {
    NoIrq,
    DataReady,
    CommandCompleted,
    CommandAcknowledged,
    ReachedEndOfData,
    DiskError,
}

impl From<u8> for DiskIrq {
    fn from(value: u8) -> Self {
        match value {
            0 => DiskIrq::NoIrq,
            1 => DiskIrq::DataReady,
            2 => DiskIrq::CommandCompleted,
            3 => DiskIrq::CommandAcknowledged,
            4 => DiskIrq::ReachedEndOfData,
            5 => DiskIrq::DiskError,
            _ => DiskIrq::NoIrq,
        }
    }
}

impl Into<u8> for DiskIrq {
    fn into(self) -> u8 {
        match self {
            DiskIrq::NoIrq => 0,
            DiskIrq::DataReady => 1,
            DiskIrq::CommandCompleted => 2,
            DiskIrq::CommandAcknowledged => 3,
            DiskIrq::ReachedEndOfData => 4,
            DiskIrq::DiskError => 5,
        }
    }
}

impl std::fmt::Display for DiskIrq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            DiskIrq::NoIrq => "No IRQ",
            DiskIrq::DataReady => "Data Ready",
            DiskIrq::CommandCompleted => "Command Complete",
            DiskIrq::CommandAcknowledged => "Command Acknowledged",
            DiskIrq::ReachedEndOfData => "Reached End Of Data",
            DiskIrq::DiskError => "Disk Error",
        };
        write!(f, "{}", name)
    }
}