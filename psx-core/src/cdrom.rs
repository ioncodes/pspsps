pub mod irq;
pub mod reg;

use crate::cdrom::irq::DiskIrq;
use crate::cdrom::reg::{
    AddressRegister, AdpCtlRegister, HChpCtl, HClrCtl, HIntMaskRegister, HIntSts, REG_ADDRESS_ADDR, REG_ADPCTL_ADDR, REG_ATV0_ADDR, REG_ATV1_ADDR, REG_ATV2_ADDR, REG_ATV3_ADDR, REG_CI_ADDR, REG_COMMAND_ADDR, REG_HCHPCTL_ADDR, REG_HCLRCTL_ADDR, REG_HINTMSK_ADDR_R, REG_HINTMSK_ADDR_W, REG_HINTSTS_ADDR, REG_HSTS_ADDR, REG_PARAMETER_ADDR, REG_RDDATA_ADDR, REG_RESULT_ADDR, SetModeRegister, StatusCode
};
use crate::mmu::bus::Bus8;
use proc_bitfield::with_bits;
use std::collections::VecDeque;

crate::define_addr!(CDROM_ADDR, 0x1F80_1800, 0, 0x04, 0x04);

// Timing constants (in CPU cycles)
pub const FIRST_RESP_GENERIC_DELAY: usize = 0x000c4e1;
pub const FIRST_RESP_INIT_DELAY: usize = 0x0013cce;
pub const SECOND_RESP_GETID_DELAY: usize = 0x0004a00;
pub const SECOND_RESP_PAUSE_DELAY: usize = 0x021181c / 4;
pub const SECOND_RESP_STOP_DELAY: usize = 0x0d38aca;

pub const SECTOR_READ_DELAY_SINGLE_SPEED: usize = 33_868_800 / 75;
pub const SECTOR_READ_DELAY_DOUBLE_SPEED: usize = 33_868_800 / (2 * 75);

// Error codes for failures
pub const ERROR_SEEK_FAILED: u8 = 0x04;
pub const ERROR_DRIVE_DOOR_BECAME_OPENED: u8 = 0x08;
pub const ERROR_INVALID_SUBCOMMAND: u8 = 0x10;
pub const ERROR_WRONG_NUMBER_OF_PARAMETERS: u8 = 0x20;
pub const ERROR_INVALID_COMMAND: u8 = 0x40;
pub const ERROR_CANNOT_RESPONSE: u8 = 0x80;

// CDROM sector constants
const SECTOR_SIZE: usize = 2352; // Raw CD-ROM sector size
const SECTOR_SUBHEADER_OFFSET: usize = 12; // Offset to subheader (after sync+header)
const SECTOR_SUBHEADER_SIZE: usize = 4; // File, channel, submode, coding
const SECTOR_DATA_OFFSET_MODE2: usize = 24; // Data starts after sync+header+subheader (0x18)
const SECTOR_DATA_SIZE_2048: usize = 2048; // User data size (0x800)
const SECTOR_DATA_SIZE_2340: usize = 2340; // Whole sector size (0x924)
// PSX-SPX: "Mode 2 Form 2: 018h-92Bh User Data (2324 bytes)"
const SECTOR_DATA_SIZE_FORM2: usize = 2324; // Form 2: 2324 bytes (0x914)

const SETLOC_CURRENT_LBA_OFFSET: usize = 8;
const XA_SUBMODE_REALTIME: u8 = 0x64;
const CYCLES_PER_MILLISECOND: usize = 33_868;

struct PendingInterrupt {
    irq: DiskIrq,
    response: Vec<u8>,
    cycles_until_fire: usize,
    is_read: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DriveState {
    Idle,
    Seeking { cycles_left: usize },
    Reading,
    Playing,
}

pub struct Cdrom {
    // Inserted disc data
    cdrom_bin: Vec<u8>,

    // Internal registers
    address: AddressRegister,
    adpctl: AdpCtlRegister,
    hintmsk: HIntMaskRegister,
    hclrctl: HClrCtl,
    hintsts: HIntSts,
    hchpctl: HChpCtl,

    // Audio volume registers (PSX-SPX: "00h=Off, 80h=Normal, FFh=Double")
    atv0: u8, // Left-to-Left volume
    atv1: u8, // Left-to-Right volume
    atv2: u8, // Right-to-Right volume
    atv3: u8, // Right-to-Left volume
    ci: u8,   // Channel Information register

    /// Stores command parameters written by software (max 16 bytes)
    parameter_fifo: VecDeque<u8>,
    /// Stores response bytes from commands that software reads back (max 16 bytes)
    /// This is NOT the sector data - it's command responses like status bytes, GetID info, etc.
    /// Sector data is read directly from disc on-demand via read_sector_data_byte()
    result_fifo: VecDeque<u8>,

    // Interrupt management
    interrupt_queue: VecDeque<PendingInterrupt>,

    // Drive state
    read_in_progress: bool,
    state: DriveState,
    mode: SetModeRegister,

    // Sector reading position tracking
    sector_offset: usize,      // Current byte offset within the sector being read
    subheader: [u8; 4],        // Current sector's subheader (file, channel, submode, coding)
    sector_lba: usize,         // Target LBA set by SetLoc command
    sector_lba_current: usize, // Current LBA being read (updated as sectors are consumed)
    data_ready: bool,          // True when a sector has been fully read

    last_command: u8, // Last executed command (0x06=ReadN, 0x1B=ReadS, etc.)
}

impl Cdrom {
    pub fn new() -> Self {
        Self {
            cdrom_bin: Vec::new(),
            address: AddressRegister(0),
            adpctl: AdpCtlRegister(0),
            hintmsk: HIntMaskRegister(0),
            hclrctl: HClrCtl(0),
            hintsts: HIntSts(0),
            hchpctl: HChpCtl(0),
            atv0: 0x80, // PSX-SPX: Default is 80h (normal volume)
            atv1: 0x80,
            atv2: 0x80,
            atv3: 0x80,
            ci: 0,
            parameter_fifo: VecDeque::new(),
            result_fifo: VecDeque::new(),
            interrupt_queue: VecDeque::new(),
            read_in_progress: false,
            state: DriveState::Idle,
            mode: SetModeRegister(0x00),
            sector_offset: 0,
            subheader: [0; 4],
            sector_lba: 0,
            sector_lba_current: 0,
            data_ready: false,
            last_command: 0,
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.address.set_parameter_empty(self.parameter_fifo.is_empty());
        self.address.set_parameter_write_ready(self.parameter_fifo.len() < 16);

        // Handle state transitions
        match self.state {
            DriveState::Seeking { cycles_left } => {
                if cycles_left <= cycles {
                    // Seek complete, transition to Reading state
                    self.sector_lba_current = self.sector_lba;
                    self.state = DriveState::Reading;

                    tracing::debug!(
                        target: "psx_core::cdrom",
                        sector_lba = self.sector_lba_current,
                        "Seek complete, starting read",
                    );

                    // Prepare the first sector read
                    self.prepare_next_sector_read();
                } else {
                    // Still seeking, decrement cycles
                    self.state = DriveState::Seeking {
                        cycles_left: cycles_left - cycles,
                    };
                }
            }
            DriveState::Reading => {
                // Continue reading sectors while in Reading state
                // This is handled after interrupt processing
            }
            _ => {}
        }

        // Process pending interrupts
        // Only process the first interrupt if no interrupt is currently active
        if self.hintsts.irq_flags() == DiskIrq::NoIrq {
            if let Some(pending) = self.interrupt_queue.front_mut() {
                if pending.cycles_until_fire <= cycles {
                    // Time to fire this interrupt
                    let pending = self.interrupt_queue.pop_front().unwrap();

                    tracing::trace!(
                        target: "psx_core::cdrom",
                        irq = %pending.irq,
                        response = format!("{:02X?}", pending.response),
                        "Triggering CDROM interrupt",
                    );

                    // Populate result FIFO with response data
                    for byte in pending.response.iter().copied() {
                        self.result_fifo.push_back(byte);
                    }

                    // Clear BUSY and update flags based on interrupt type
                    self.address.set_busy_status(false);
                    // Result ready only if we actually have response bytes
                    self.address.set_result_read_ready(!self.result_fifo.is_empty());
                    // Data request flag set for Data Ready (INT1) interrupts when reading
                    self.address.set_data_request(pending.is_read && self.read_in_progress);

                    // Trigger the interrupt
                    self.trigger_irq(pending.irq);

                    // If this was a data ready interrupt and we're still reading,
                    // prepare the next sector read
                    if pending.is_read && self.read_in_progress && self.state == DriveState::Reading {
                        let should_read_next_sector = match self.last_command {
                            0x1B => {
                                let submode = self.subheader[2];
                                // Skip real-time XA sectors when filter is enabled
                                if submode == XA_SUBMODE_REALTIME && self.mode.xa_filter() {
                                    self.sector_lba += 1;
                                    false
                                } else {
                                    true
                                }
                            }
                            _ => true, // ReadN and others: Always read all sectors
                        };

                        if should_read_next_sector {
                            self.prepare_next_sector_read();
                        }
                    }
                } else {
                    // Decrement the cycle counter
                    pending.cycles_until_fire -= cycles;
                }
            }
        }
    }

    pub fn check_and_clear_irq(&mut self) -> bool {
        let irq = self.hintsts.irq_flags();
        if irq != DiskIrq::NoIrq && self.hintmsk.enable_irq_on_intsts() & irq as u8 != 0 {
            return true;
        }

        false
    }

    fn queue_interrupt(&mut self, irq: DiskIrq, response: Vec<u8>, delay_cycles: usize, is_read: bool) {
        self.interrupt_queue.push_back(PendingInterrupt {
            irq,
            response,
            cycles_until_fire: delay_cycles,
            is_read,
        });
    }

    /// Read a single data byte from the current sector position (on-demand)
    fn read_sector_data_byte(&mut self) -> u8 {
        // Read subheader at start of sector
        if self.sector_offset == 0 {
            let subheader_addr = (self.sector_lba * SECTOR_SIZE) + SECTOR_SUBHEADER_OFFSET + SECTOR_SUBHEADER_SIZE;
            if subheader_addr + SECTOR_SUBHEADER_SIZE <= self.cdrom_bin.len() {
                self.subheader
                    .copy_from_slice(&self.cdrom_bin[subheader_addr..subheader_addr + SECTOR_SUBHEADER_SIZE]);

                tracing::trace!(
                    target: "psx_core::cdrom",
                    lba = self.sector_lba,
                    subheader = format!("{:02X?}", self.subheader),
                    "Read sector subheader",
                );
            }
        }

        // PSX-SPX: "Bit 5: Sector Size (0=800h=DataOnly, 1=924h=WholeSectorExceptSyncBytes)"
        let whole_sector = self.mode.sector_size();

        // PSX-SPX: Whole-sector mode returns 0x924 bytes starting at +0x0C; otherwise 2048/2324 bytes at +0x18
        let data_offset = if whole_sector { 0x0C } else { SECTOR_DATA_OFFSET_MODE2 };

        // PSX-SPX: "Sub-Header Submode Byte 012h, Bit 5: Form indicator - 0=Form1/800h-byte data, 1=Form2, 914h-byte data"
        let submode = self.subheader[2];
        let is_form2 = (submode >> 5) & 1; // Check bit 5 for Form 2

        // Calculate sector size based on form and whole_sector mode
        let sector_size_max = if whole_sector {
            SECTOR_DATA_SIZE_2340 // Return whole sector (2340 bytes)
        } else if is_form2 == 1 {
            SECTOR_DATA_SIZE_FORM2 // Form 2: return 2324 bytes
        } else {
            SECTOR_DATA_SIZE_2048 // Form 1: return 2048 bytes
        };

        // Calculate actual byte position in the disc image
        let byte_addr = (self.sector_lba * SECTOR_SIZE) + data_offset + self.sector_offset;

        let byte = if byte_addr < self.cdrom_bin.len() {
            self.cdrom_bin[byte_addr]
        } else {
            tracing::warn!(
                target: "psx_core::cdrom",
                byte_addr,
                bin_size = self.cdrom_bin.len(),
                "Read past end of disc",
            );
            // TODO: i think we repeat the last few bytes?
            0xFF
        };

        // Advance offset
        self.sector_offset += 1;

        // Check for sector boundary
        let reached_end = self.sector_offset >= sector_size_max;

        if reached_end {
            // Sector complete, advance to next sector
            self.sector_offset = 0;
            self.sector_lba += 1;
            self.data_ready = true;

            tracing::trace!(
                target: "psx_core::cdrom",
                lba = self.sector_lba - 1,
                next_lba = self.sector_lba,
                "Sector read complete, advancing to next",
            );
        }

        byte
    }

    /// Prepare for next sector read (called when INT1 fires)
    fn prepare_next_sector_read(&mut self) {
        self.data_ready = false;

        // Queue INT1 with Read status bit set
        let mut status = self.status();
        status.set_read(true); // Force Read bit ON for INT1 response

        let sector_delay = if self.mode.double_speed() {
            SECTOR_READ_DELAY_DOUBLE_SPEED
        } else {
            SECTOR_READ_DELAY_SINGLE_SPEED
        };

        tracing::trace!(
            target: "psx_core::cdrom",
            lba = self.sector_lba,
            status = format!("{:02X}", status.0),
            "Queueing INT1 for next sector",
        );

        // INT1 response must reflect Data Request in status byte (bit5)
        self.queue_interrupt(DiskIrq::DataReady, vec![status.0 | 0x20], sector_delay, true);

        self.hchpctl.set_request_sector_buffer_read(false);
    }

    pub fn insert_disk(&mut self, bin: Vec<u8>) {
        self.cdrom_bin = bin;

        tracing::info!(
            target: "psx_core::cdrom",
            size = self.cdrom_bin.len(),
            "CD-ROM disk inserted",
        );
    }

    fn execute_command(&mut self, command: u8) {
        // Set BUSY flag immediately when command is received
        self.address.set_busy_status(true);

        // Track the command for later reference (e.g., ReadS vs ReadN)
        self.last_command = command;

        match command {
            // 0x01 	GetStat 	INT3: status
            0x01 => {
                let status = self.status();
                self.queue_interrupt(
                    DiskIrq::CommandAcknowledged,
                    vec![status.0],
                    FIRST_RESP_GENERIC_DELAY,
                    false,
                );
            }
            // Setloc - Command 02h,amm,ass,asect --> INT3(stat)
            0x02 => {
                self.execute_setloc();
            }
            // Stop - Command 08h --> INT3(stat) --> INT2(stat)
            0x08 => {
                self.execute_stop();
            }
            // Init - Command 0Ah --> INT3(stat late) --> INT2(stat)
            0x0A => {
                self.execute_init();
            }
            // Mute - Command 0Bh --> INT3(stat)
            0x0B => {
                // PSX-SPX: "Turn off audio streaming to SPU (affects both CD-DA and XA-ADPCM)"
                // TODO: For now, just acknowledge the command
                let status = self.status();
                self.queue_interrupt(
                    DiskIrq::CommandAcknowledged,
                    vec![status.0],
                    FIRST_RESP_GENERIC_DELAY,
                    false,
                );
            }
            // Demute - Command 0Ch --> INT3(stat)
            0x0C => {
                // PSX-SPX: "Turn on audio streaming to SPU (affects both CD-DA and XA-ADPCM)"
                // TODO: For now, just acknowledge the command
                let status = self.status();
                self.queue_interrupt(
                    DiskIrq::CommandAcknowledged,
                    vec![status.0],
                    FIRST_RESP_GENERIC_DELAY,
                    false,
                );
            }
            // Setmode - Command 0Eh,mode --> INT3(stat)
            0x0E => {
                self.execute_setmode();
            }
            // Play - Command 03h --> INT3(stat) --> INT2(stat) (CD-DA playback)
            0x03 => {
                self.execute_play();
            }
            // ReadN - Command 06h --> INT3(stat) --> INT1(stat) --> datablock
            0x06 => {
                self.execute_readn();
            }
            // ReadS - Command 1Bh --> INT3(stat) --> INT1(stat) --> datablock (with XA filtering)
            0x1B => {
                self.execute_readn(); // Same as ReadN but filtering happens in tick()
            }
            // Pause - Command 09h --> INT3(stat) --> INT2(stat)
            0x09 => {
                self.execute_pause();
            }
            // SeekL - Command 15h --> INT3(stat) --> INT2(stat)
            0x15 => {
                self.execute_seekl();
            }
            // 0x19 	Test * 	sub, ... 	INT3: ...
            0x19 => {
                let subcommand = self.parameter_fifo.pop_front().unwrap();
                self.execute_subcommand(subcommand);
            }
            // 0x13 GetTN - Get first and last track numbers
            0x13 => {
                self.execute_get_tn();
            }
            // 0x14 GetTD - Get absolute/relative time for track
            0x14 => {
                self.execute_get_td();
            }
            // 0x1a 	GetID * 		INT3: status 	INT2/INT5: status, flag, type, atip, "SCEx"
            0x1A => {
                self.execute_get_id();
            }
            // ReadTOC - Command 1Eh --> INT3(stat late) --> INT2(stat)
            0x1E => {
                self.execute_read_toc();
            }
            _ => {
                tracing::error!(
                    target: "psx_core::cdrom",
                    command = format!("{:02X}", command),
                    "Unimplemented CDROM command",
                );
            }
        }

        // Clear parameter FIFO after processing command
        self.parameter_fifo.clear();
    }

    fn execute_subcommand(&mut self, subcommand: u8) {
        match subcommand {
            // 20h      -   INT3(yy,mm,dd,ver) ;Get cdrom BIOS date/version (yy,mm,dd,ver)
            0x20 => {
                // (unknown)        ;DTL-H2000 (with SPC700 instead HC05)
                // 94h,09h,19h,C0h  ;PSX (PU-7)               19 Sep 1994, version vC0 (a)
                // 94h,11h,18h,C0h  ;PSX (PU-7)               18 Nov 1994, version vC0 (b)
                // 94h,11h,28h,01h  ;PSX (DTL-H2000)          28 Nov 1994, version v01 (debug)
                // 95h,05h,16h,C1h  ;PSX (LATE-PU-8)          16 May 1995, version vC1 (a)
                // 95h,07h,24h,C1h  ;PSX (LATE-PU-8)          24 Jul 1995, version vC1 (b)
                // 95h,07h,24h,D1h  ;PSX (LATE-PU-8,debug ver)24 Jul 1995, version vD1 (debug)
                // 96h,08h,15h,C2h  ;PSX (PU-16, Video CD)    15 Aug 1996, version vC2 (VCD)
                // 96h,08h,18h,C1h  ;PSX (LATE-PU-8,yaroze)   18 Aug 1996, version vC1 (yaroze)
                // 96h,09h,12h,C2h  ;PSX (PU-18) (japan)      12 Sep 1996, version vC2 (a.jap)
                // 97h,01h,10h,C2h  ;PSX (PU-18) (us/eur)     10 Jan 1997, version vC2 (a)
                // 97h,08h,14h,C2h  ;PSX (PU-20)              14 Aug 1997, version vC2 (b)
                // 98h,06h,10h,C3h  ;PSX (PU-22)              10 Jun 1998, version vC3 (a)
                // 99h,02h,01h,C3h  ;PSX/PSone (PU-23, PM-41) 01 Feb 1999, version vC3 (b)
                // A1h,03h,06h,C3h  ;PSone/late (PM-41(2))    06 Jun 2001, version vC3 (c)
                // (unknown)        ;PS2,   xx xxx xxxx, late PS2 models...?
                self.queue_interrupt(
                    DiskIrq::CommandAcknowledged,
                    vec![0x94, 0x09, 0x19, 0xC0], // Sept 19, 1994, version vC0 (a)
                    FIRST_RESP_GENERIC_DELAY,
                    false,
                );
            }
            _ => {
                tracing::error!(
                    target: "psx_core::cdrom",
                    subcommand = format!("{:02X}", subcommand),
                    "Unimplemented CDROM subcommand",
                );
            }
        }
    }

    fn execute_get_tn(&mut self) {
        let status = self.status();

        let first_track = 0x01;
        let last_track = 0x01;

        self.queue_interrupt(
            DiskIrq::CommandAcknowledged,
            vec![status.0, first_track, last_track],
            FIRST_RESP_GENERIC_DELAY,
            false,
        );

        tracing::debug!(
            target: "psx_core::cdrom",
            first = format!("{:02X}", first_track),
            last = format!("{:02X}", last_track),
            "GetTN"
        );
    }

    fn execute_get_td(&mut self) {
        let track = self.parameter_fifo.pop_front().unwrap_or(0);
        let status = self.status();

        let (mm_bcd, ss_bcd) = match track {
            1 => (0x00, 0x02),
            _ => (0x00, 0x00),
        };

        self.queue_interrupt(
            DiskIrq::CommandAcknowledged,
            vec![status.0, mm_bcd, ss_bcd],
            FIRST_RESP_GENERIC_DELAY,
            false,
        );

        tracing::debug!(
            target: "psx_core::cdrom",
            track = format!("{:02X}", track),
            mm = format!("{:02X}", mm_bcd),
            ss = format!("{:02X}", ss_bcd),
            "GetTD"
        );
    }

    fn execute_get_id(&mut self) {
        // Drive Status           1st Response   2nd Response
        // Door Open              INT5(11h,80h)  N/A
        // Spin-up                INT5(01h,80h)  N/A
        // Detect busy            INT5(03h,80h)  N/A
        // No Disk                INT3(stat)     INT5(08h,40h, 00h,00h, 00h,00h,00h,00h)
        // Audio Disk             INT3(stat)     INT5(0Ah,90h, 00h,00h, 00h,00h,00h,00h)
        // Unlicensed:Mode1       INT3(stat)     INT5(0Ah,80h, 00h,00h, 00h,00h,00h,00h)
        // Unlicensed:Mode2       INT3(stat)     INT5(0Ah,80h, 20h,00h, 00h,00h,00h,00h)
        // Unlicensed:Mode2+Audio INT3(stat)     INT5(0Ah,90h, 20h,00h, 00h,00h,00h,00h)
        // Debug/Yaroze:Mode2     INT3(stat)     INT2(02h,00h, 20h,00h, 20h,20h,20h,20h)
        // Licensed:Mode2         INT3(stat)     INT2(02h,00h, 20h,00h, 53h,43h,45h,4xh)
        // Modchip:Audio/Mode1    INT3(stat)     INT2(02h,00h, 00h,00h, 53h,43h,45h,4xh)

        let status = self.status();

        if status.shell_open() {
            let mut error_stat = status;
            error_stat.set_error(true);
            error_stat.set_id_error(true);

            let response = vec![error_stat.0, ERROR_DRIVE_DOOR_BECAME_OPENED];
            self.queue_interrupt(DiskIrq::DiskError, response, FIRST_RESP_GENERIC_DELAY, false);

            return;
        }

        // First response: INT3 acknowledgment with status
        self.queue_interrupt(
            DiskIrq::CommandAcknowledged,
            vec![status.0],
            FIRST_RESP_GENERIC_DELAY,
            false,
        );

        // Second response: INT2 (success) or INT5 (error)
        if self.disk_inserted() {
            // Success: Return disk identification (Licensed:Mode2, SCEA region)
            let response = vec![
                0x02, 0x00, 0x20, 0x00, b'S', b'C', b'E', b'A', // Region: SCEA (America/NTSC)
            ];
            self.queue_interrupt(DiskIrq::CommandCompleted, response, SECOND_RESP_GETID_DELAY, false);
        } else {
            // Error: Disk missing
            let mut error_stat = status;
            error_stat.set_error(true);
            error_stat.set_id_error(true);

            let response = vec![error_stat.0, ERROR_INVALID_COMMAND];
            self.queue_interrupt(DiskIrq::DiskError, response, SECOND_RESP_GETID_DELAY, false);
        }
    }

    fn execute_setloc(&mut self) {
        let minutes = self.parameter_fifo.pop_front().unwrap() as usize;
        let seconds = self.parameter_fifo.pop_front().unwrap() as usize;
        let frames = self.parameter_fifo.pop_front().unwrap() as usize;

        // "Note that each of these parameters is encoded as BCD values, not binary."
        let minutes = ((minutes >> 4) * 10) + (minutes & 0x0F);
        let seconds = ((seconds >> 4) * 10) + (seconds & 0x0F);
        let frames = ((frames >> 4) * 10) + (frames & 0x0F);

        let block_addr = (((minutes * 60) + seconds) * 75 + frames) - 150;

        self.sector_lba = block_addr;
        self.sector_lba_current = block_addr + SETLOC_CURRENT_LBA_OFFSET;
        self.sector_offset = 0;

        tracing::debug!(
            target: "psx_core::cdrom",
            min = minutes,
            sec = seconds,
            frame = frames,
            lba = block_addr,
            lba_current = self.sector_lba_current,
            "Setloc",
        );

        let status = self.status();
        self.queue_interrupt(
            DiskIrq::CommandAcknowledged,
            vec![status.0],
            FIRST_RESP_GENERIC_DELAY,
            false,
        );
    }

    fn execute_pause(&mut self) {
        tracing::debug!(
            target: "psx_core::cdrom",
            sector_lba = self.sector_lba_current,
            state = ?self.state,
            "Pause",
        );

        let status_before = self.status();
        self.queue_interrupt(
            DiskIrq::CommandAcknowledged,
            vec![status_before.0],
            FIRST_RESP_GENERIC_DELAY,
            false,
        );

        // Stop any ongoing reads and clear queued read-related interrupts
        self.state = DriveState::Idle;
        self.read_in_progress = false;
        self.sector_offset = 0;
        self.data_ready = false;
        self.address.set_data_request(false);
        // Drop any pending DataReady interrupts
        self.interrupt_queue.retain(|p| !p.is_read);

        let status_after = self.status();
        self.queue_interrupt(
            DiskIrq::CommandCompleted,
            vec![status_after.0],
            SECOND_RESP_PAUSE_DELAY,
            false,
        );
    }

    fn execute_stop(&mut self) {
        tracing::debug!(
            target: "psx_core::cdrom",
            "Stop",
        );

        self.state = DriveState::Idle;
        self.read_in_progress = false;
        self.sector_offset = 0;
        self.data_ready = false;
        self.address.set_data_request(false);
        self.interrupt_queue.retain(|p| !p.is_read);

        let status = self.status();
        self.queue_interrupt(
            DiskIrq::CommandAcknowledged,
            vec![status.0],
            FIRST_RESP_GENERIC_DELAY,
            false,
        );
        
        self.queue_interrupt(
            DiskIrq::CommandCompleted,
            vec![status.0],
            SECOND_RESP_STOP_DELAY,
            false,
        );
    }

    fn execute_init(&mut self) {
        tracing::debug!(
            target: "psx_core::cdrom",
            "Init",
        );

        // Abort all pending operations
        self.interrupt_queue.clear();
        self.read_in_progress = false;
        self.state = DriveState::Idle;

        // Set default mode
        self.mode = SetModeRegister(0x20);

        // Reset position to start
        self.sector_offset = 0;
        self.sector_lba = 0;
        self.sector_lba_current = 0;

        let status = self.status();

        self.queue_interrupt(
            DiskIrq::CommandAcknowledged,
            vec![status.0],
            FIRST_RESP_INIT_DELAY,
            false,
        );
        self.queue_interrupt(
            DiskIrq::CommandCompleted,
            vec![status.0],
            FIRST_RESP_INIT_DELAY + SECOND_RESP_GETID_DELAY,
            false,
        );
    }

    fn execute_read_toc(&mut self) {
        tracing::debug!(
            target: "psx_core::cdrom",
            "ReadTOC",
        );

        let status = self.status();

        self.queue_interrupt(
            DiskIrq::CommandAcknowledged,
            vec![status.0],
            FIRST_RESP_INIT_DELAY,
            false,
        );
        self.queue_interrupt(
            DiskIrq::CommandCompleted,
            vec![status.0],
            FIRST_RESP_INIT_DELAY + SECOND_RESP_GETID_DELAY,
            false,
        );
    }

    fn execute_seekl(&mut self) {
        tracing::debug!(
            target: "psx_core::cdrom",
            sector_lba = self.sector_lba,
            "SeekL",
        );

        // Update current position to the requested position
        self.sector_lba_current = self.sector_lba;
        self.sector_offset = 0;

        let status = self.status();
        self.queue_interrupt(
            DiskIrq::CommandAcknowledged,
            vec![status.0],
            FIRST_RESP_GENERIC_DELAY,
            false,
        );
        self.queue_interrupt(
            DiskIrq::CommandCompleted,
            vec![status.0],
            SECOND_RESP_PAUSE_DELAY,
            false,
        );
    }

    fn execute_setmode(&mut self) {
        let mode_byte = self.parameter_fifo.pop_front().unwrap();

        // Store the mode value
        self.mode = SetModeRegister(mode_byte);

        tracing::trace!(
            target: "psx_core::cdrom",
            mode = format!("{:08b}", mode_byte),
            speed = if self.mode.double_speed() { "double" } else { "normal" },
            xa_adpcm = self.mode.xa_adpcm(),
            sector_size = if self.mode.sector_size() { "924" } else { "2048+12" },
            ignore_bit = self.mode.ignore_bit(),
            xa_filter = self.mode.xa_filter(),
            report = self.mode.report(),
            autopause = self.mode.autopause(),
            cdda = self.mode.cdda(),
            "Setmode",
        );

        let status = self.status();
        self.queue_interrupt(
            DiskIrq::CommandAcknowledged,
            vec![status.0],
            FIRST_RESP_GENERIC_DELAY,
            false,
        );
    }

    fn execute_play(&mut self) {
        tracing::debug!(
            target: "psx_core::cdrom",
            from_lba = self.sector_lba_current,
            to_lba = self.sector_lba,
            "Play",
        );

        let status_before = self.status();
        self.queue_interrupt(
            DiskIrq::CommandAcknowledged,
            vec![status_before.0],
            FIRST_RESP_GENERIC_DELAY,
            false,
        );

        self.read_in_progress = false;
        self.address.set_data_request(false);
        self.interrupt_queue.retain(|p| !p.is_read);
        self.state = DriveState::Playing;

        let status_after = self.status();
        self.queue_interrupt(
            DiskIrq::CommandCompleted,
            vec![status_after.0],
            SECOND_RESP_PAUSE_DELAY,
            false,
        );
    }

    fn execute_readn(&mut self) {
        tracing::debug!(
            target: "psx_core::cdrom",
            from_lba = self.sector_lba_current,
            to_lba = self.sector_lba,
            "ReadN",
        );

        // Immediately send INT3 acknowledgment
        let status = self.status();
        self.queue_interrupt(
            DiskIrq::CommandAcknowledged,
            vec![status.0],
            FIRST_RESP_GENERIC_DELAY,
            false,
        );

        // Calculate seek distance and time
        let distance = if self.sector_lba > self.sector_lba_current {
            self.sector_lba - self.sector_lba_current
        } else {
            self.sector_lba_current - self.sector_lba
        };

        // Seek timing: ~1ms per sector difference
        let seek_cycles = if distance == 0 {
            FIRST_RESP_GENERIC_DELAY
        } else {
            // ~1ms per sector, capped at ~2 seconds max
            (distance.max(1) * CYCLES_PER_MILLISECOND).min(SECOND_RESP_PAUSE_DELAY)
        };

        // Enter seeking state
        self.state = DriveState::Seeking {
            cycles_left: seek_cycles,
        };
        self.read_in_progress = true;
    }

    fn trigger_irq(&mut self, irq: DiskIrq) {
        tracing::trace!(target: "psx_core::cdrom", %irq, "Raised CDROM IRQ");
        self.hintsts.set_irq_flags(irq);
    }

    fn status(&mut self) -> StatusCode {
        let mut status = StatusCode(0);
        status.set_shell_open(!self.disk_inserted());
        status.set_spindle_motor(self.disk_inserted()); // Motor on when disk is present

        // Set state bits (only one can be set at a time)
        match self.state {
            DriveState::Idle => {
                // All state bits (5-7) are 0
            }
            DriveState::Seeking { .. } => {
                status.set_seek(true); // Bit 6
            }
            DriveState::Reading => {
                status.set_read(true); // Bit 5
            }
            DriveState::Playing => {
                status.set_play(true); // Bit 7
            }
        }

        status
    }

    fn disk_inserted(&self) -> bool {
        !self.cdrom_bin.is_empty()
    }
}

impl Bus8 for Cdrom {
    fn read_u8(&mut self, address: u32) -> u8 {
        match address {
            REG_HSTS_ADDR => {
                tracing::trace!(
                    target: "psx_core::cdrom",
                    bank = self.address.current_bank(),
                    param_empty = self.address.parameter_empty(),
                    param_write_ready = self.address.parameter_write_ready(),
                    result_ready = self.address.result_read_ready(),
                    data_request = self.address.data_request(),
                    busy_status = self.address.busy_status(),
                    adcpm_busy = self.address.adpcm_busy(),
                    "CDROM HSTS/ADDRESS read",
                );
                self.address.0
            }
            REG_RESULT_ADDR => {
                let value = self.result_fifo.pop_front().unwrap_or_else(|| {
                    tracing::warn!(
                        target: "psx_core::cdrom",
                        "Result FIFO underflow on read",
                    );
                    0xFF
                });
                // Clear result ready when FIFO becomes empty
                if self.result_fifo.is_empty() {
                    self.address.set_result_read_ready(false);
                }
                value
            }
            REG_RDDATA_ADDR => {
                // RDDATA reads sector data on-demand
                let value = self.read_sector_data_byte();

                tracing::trace!(
                    target: "psx_core::cdrom",
                    bank = self.address.current_bank(),
                    rdata = format!("{:02X}", value),
                    sector_offset = self.sector_offset,
                    lba = self.sector_lba,
                    "CDROM RDDATA read",
                );

                // If we've completed a sector, trigger next read if still in read mode
                if self.data_ready && self.read_in_progress && self.state == DriveState::Reading {
                    // The prepare_next_sector_read will be called in tick() when INT1 fires
                    // For now just deassert data request until next INT1
                    self.address.set_data_request(false);
                }

                value
            }
            REG_HINTMSK_ADDR_R if self.address.current_bank() == 0 || self.address.current_bank() == 2 => {
                tracing::trace!(
                    target: "psx_core::cdrom",
                    bank = self.address.current_bank(),
                    hintmsk = format!("{:08b}", self.hintmsk.0),
                    "CDROM HINTMSK read",
                );
                self.hintmsk.0
            }
            REG_HINTSTS_ADDR if self.address.current_bank() == 1 || self.address.current_bank() == 3 => {
                tracing::trace!(
                    target: "psx_core::cdrom",
                    bank = self.address.current_bank(),
                    hintsts = format!("{:08b}", self.hintsts.0),
                    "CDROM HINTSTS read",
                );
                with_bits!(self.hintsts.0, 5..=7 = 1)
            }
            _ => {
                tracing::error!(
                    target: "psx_core::cdrom",
                    bank = self.address.current_bank(),
                    address = format!("{:08X}", address),
                    "CDROM read not implemented",
                );
                0xFF
            }
        }
    }

    fn write_u8(&mut self, address: u32, value: u8) {
        match address {
            REG_ADDRESS_ADDR => {
                // Only bits 0-1 select bank; other bits are status and read-only
                let prev = self.address.0;
                self.address.0 = (prev & !0b11) | (value & 0b11);
                tracing::trace!(
                    target: "psx_core::cdrom",
                    bank = self.address.current_bank(),
                    "Bank switched",
                );
            }
            REG_PARAMETER_ADDR if self.address.current_bank() == 0 => {
                self.parameter_fifo.push_back(value);
                // Parameter FIFO is non-empty now
                self.address.set_parameter_empty(false);
                tracing::trace!(
                    target: "psx_core::cdrom",
                    fifo = %format!("{:02X?}", self.parameter_fifo),
                    "Parameter FIFO write",
                );
            }
            REG_COMMAND_ADDR if self.address.current_bank() == 0 => {
                tracing::trace!(
                    target: "psx_core::cdrom",
                    command = format!("{:02X}", value),
                    "Command received",
                );
                self.execute_command(value);
            }
            REG_HCHPCTL_ADDR if self.address.current_bank() == 0 => {
                self.hchpctl = HChpCtl(value);

                tracing::trace!(
                    target: "psx_core::cdrom",
                    value = format!("{:02X}", value),
                    request_sector_buffer_read = self.hchpctl.request_sector_buffer_read(),
                    "HCHPCTL write",
                );
            }
            REG_HCLRCTL_ADDR if self.address.current_bank() == 1 => {
                // Store register value first
                self.hclrctl = HClrCtl(value);

                // Clear specified IRQ flags
                let old_flags: u8 = self.hintsts.irq_flags().into();
                self.hintsts.set_irq_flags((old_flags & !value).into());

                // Acknowledge buffer-related interrupts per HCLRCTL
                if self.hclrctl.ack_bfwrdy_interrupt() {
                    self.hintsts.set_sound_map_buffer_write_ready(false);
                }
                
                if self.hclrctl.ack_bfempt_interrupt() {
                    self.hintsts.set_sound_map_buffer_empty(false);
                }
            }
            REG_HINTMSK_ADDR_W if self.address.current_bank() == 1 => {
                self.hintmsk.0 = value & 0b0001_1111;
            }
            REG_CI_ADDR if self.address.current_bank() == 2 => {
                self.ci = value;
                tracing::trace!(
                    target: "psx_core::cdrom",
                    value = format!("{:02X}", value),
                    "CI (Channel Information) write"
                );
            }
            REG_ATV0_ADDR if self.address.current_bank() == 2 => {
                self.atv0 = value;
                tracing::trace!(
                    target: "psx_core::cdrom",
                    value = format!("{:02X}", value),
                    "ATV0 (Left-to-Left volume) write"
                );
            }
            REG_ATV1_ADDR if self.address.current_bank() == 2 => {
                self.atv1 = value;
                tracing::trace!(
                    target: "psx_core::cdrom",
                    value = format!("{:02X}", value),
                    "ATV1 (Left-to-Right volume) write"
                );
            }
            REG_ATV2_ADDR if self.address.current_bank() == 3 => {
                self.atv2 = value;
                tracing::trace!(
                    target: "psx_core::cdrom",
                    value = format!("{:02X}", value),
                    "ATV2 (Right-to-Right volume) write"
                );
            }
            REG_ATV3_ADDR if self.address.current_bank() == 3 => {
                self.atv3 = value;
                tracing::trace!(
                    target: "psx_core::cdrom",
                    value = format!("{:02X}", value),
                    "ATV3 (Right-to-Left volume) write"
                );
            }
            REG_ADPCTL_ADDR if self.address.current_bank() == 3 => self.adpctl = AdpCtlRegister(value),
            _ => tracing::error!(
                target: "psx_core::cdrom",
                address = format!("{:08X}", address),
                value = format!("{:02X}", value),
                bank = self.address.current_bank(),
                "CDROM write not implemented"
            ),
        }
    }
}
