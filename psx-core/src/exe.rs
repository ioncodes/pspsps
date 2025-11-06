pub struct Exe {
    pub header: Vec<u8>,
    pub data: Vec<u8>,
    pub entry_point: u32,
    pub map_address: u32,
    pub initial_gp: u32,
    pub initial_sp: u32,
    pub initial_fp: u32,
    pub sp_offset: u32,
    pub fp_offset: u32,
    pub license: String,
}

impl Exe {
    pub fn parse(exe_buffer: Vec<u8>) -> Self {
        let entry_point = u32::from_le_bytes(exe_buffer[0x10..0x14].try_into().unwrap());
        let map_address = u32::from_le_bytes(exe_buffer[0x18..0x1c].try_into().unwrap());
        let initial_gp = u32::from_le_bytes(exe_buffer[0x14..0x18].try_into().unwrap());
        let initial_sp_fp = u32::from_le_bytes(exe_buffer[0x30..0x34].try_into().unwrap());
        let sp_fp_offset = u32::from_le_bytes(exe_buffer[0x34..0x38].try_into().unwrap());
        let license = exe_buffer[0x4C..]
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as char)
            .collect::<String>();

        Self {
            header: exe_buffer[0x00..0x800].to_vec(),
            data: exe_buffer[0x800..].to_vec(),
            entry_point,
            map_address,
            initial_gp,
            initial_sp: initial_sp_fp,
            initial_fp: initial_sp_fp,
            sp_offset: sp_fp_offset,
            fp_offset: sp_fp_offset,
            license,
        }
    }

    pub fn sp(&self) -> u32 {
        self.initial_sp + self.sp_offset
    }

    pub fn fp(&self) -> u32 {
        self.initial_fp + self.fp_offset
    }
}
