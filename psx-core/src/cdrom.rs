pub mod irq;
pub mod reg;

use crate::cdrom::irq::DiskIrq;
use crate::cdrom::reg::{
    AddressRegister, AdpCtlRegister, HClrCtl, HIntMaskRegister, HIntSts, REG_ADDRESS_ADDR, REG_ADPCTL_ADDR,
    REG_COMMAND_ADDR, REG_HCHPCTL_ADDR, REG_HCLRCTL_ADDR, REG_HINTMSK_ADDR_R, REG_HINTMSK_ADDR_W, REG_HINTSTS_ADDR,
    REG_HSTS_ADDR, REG_PARAMETER_ADDR, REG_RDDATA_ADDR, REG_RESULT_ADDR, SetModeRegister, StatusCode,
};
use crate::mmu::bus::Bus8;
use proc_bitfield::with_bits;
use std::collections::VecDeque;

crate::define_addr!(CDROM_ADDR, 0x1F80_1800, 0, 0x04, 0x04);

pub const FIRST_RESP_GENERIC_DELAY: usize = 0x000c4e1;
pub const FIRST_RESP_INIT_DELAY: usize = 0x0013cce;
pub const SECOND_RESP_GETID_DELAY: usize = 0x0004a00;
pub const SECOND_RESP_PAUSE_DELAY: usize = 0x021181c;
pub const SECOND_RESP_STOP_DELAY: usize = 0x0d38aca;

pub const SECTOR_READ_DELAY_SINGLE_SPEED: usize = 33_868_800 / 75;
pub const SECTOR_READ_DELAY_DOUBLE_SPEED: usize = 33_868_800 / (2 * 75);

pub const ERROR_SEEK_FAILED: u8 = 0x04;
pub const ERROR_DRIVE_DOOR_BECAME_OPENED: u8 = 0x08;
pub const ERROR_INVALID_SUBCOMMAND: u8 = 0x10;
pub const ERROR_WRONG_NUMBER_OF_PARAMETERS: u8 = 0x20;
pub const ERROR_INVALID_COMMAND: u8 = 0x40;
pub const ERROR_CANNOT_RESPONSE: u8 = 0x80;

const SECTOR_SIZE: usize = 2352;

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
    cdrom_cue: Vec<u8>,
    cdrom_bin: Vec<u8>,
    address: AddressRegister,
    adpctl: AdpCtlRegister,
    hintmsk: HIntMaskRegister,
    hclrctl: HClrCtl,
    hintsts: HIntSts,
    parameter_fifo: VecDeque<u8>,
    result_fifo: VecDeque<u8>,
    data_fifo: VecDeque<u8>,
    interrupt_queue: VecDeque<PendingInterrupt>,
    requested_cursor: usize,
    current_cursor: usize,
    read_in_progress: bool,
    state: DriveState,
    mode: SetModeRegister,
}

impl Cdrom {
    pub fn new() -> Self {
        Self {
            cdrom_cue: Vec::new(),
            cdrom_bin: Vec::new(),
            address: AddressRegister(0),
            adpctl: AdpCtlRegister(0),
            hintmsk: HIntMaskRegister(0),
            hclrctl: HClrCtl(0),
            hintsts: HIntSts(0),
            parameter_fifo: VecDeque::new(),
            result_fifo: VecDeque::new(),
            data_fifo: VecDeque::new(),
            interrupt_queue: VecDeque::new(),
            requested_cursor: 0,
            current_cursor: 0,
            read_in_progress: false,
            state: DriveState::Idle,
            mode: SetModeRegister(0x00),
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
                    self.current_cursor = self.requested_cursor;
                    self.state = DriveState::Reading;

                    tracing::debug!(
                        target: "psx_core::cdrom",
                        cursor = self.current_cursor,
                        "Seek complete, starting read",
                    );

                    // Immediately queue first sector read
                    self.load_sector_to_data_fifo();
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

                    tracing::debug!(
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
                    // Data request only for Data Ready (sector available) interrupts
                    self.address.set_data_request(pending.is_read && !self.data_fifo.is_empty());

                    // Trigger the interrupt
                    self.trigger_irq(pending.irq);

                    // If this was a data ready interrupt and we're still reading,
                    // queue the next sector
                    if pending.is_read && self.read_in_progress && self.state == DriveState::Reading {
                        self.load_sector_to_data_fifo();
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

    fn load_sector_to_data_fifo(&mut self) {
        self.data_fifo.clear();

        let whole_sector = self.mode.sector_size();
        let data_offset = if whole_sector { 12 } else { 24 };
        let data_size = if whole_sector { 0x924 } else { 0x800 };

        let sector_start = self.current_cursor;
        let data_start = sector_start + data_offset;
        let data_end = data_start + data_size;

        // Load sector data into FIFO (bounds check to prevent reading past disc end)
        if data_end <= self.cdrom_bin.len() {
            for i in data_start..data_end {
                self.data_fifo.push_back(self.cdrom_bin[i]);
            }

            tracing::debug!(
                target: "psx_core::cdrom",
                cursor = self.current_cursor,
                sector = self.current_cursor / SECTOR_SIZE,
                mode = format!("{:02X}", self.mode.0),
                whole_sector,
                data_offset,
                data_size,
                "Loaded sector to data FIFO",
            );
        } else {
            tracing::warn!(
                target: "psx_core::cdrom",
                cursor = self.current_cursor,
                bin_size = self.cdrom_bin.len(),
                "Attempted to read past end of disc",
            );
        }

        // Queue INT1 with status byte, using timing based on read speed
        let status = self.status();
        let sector_delay = if self.mode.double_speed() {
            SECTOR_READ_DELAY_DOUBLE_SPEED
        } else {
            SECTOR_READ_DELAY_SINGLE_SPEED
        };

        self.queue_interrupt(DiskIrq::DataReady, vec![status.0], sector_delay, true);
        // Ensure data request is deasserted until INT1 actually fires
        self.address.set_data_request(false);

        // Advance to next sector
        self.current_cursor += SECTOR_SIZE;
    }

    pub fn insert_disk(&mut self, cue: Vec<u8>, bin: Vec<u8>) {
        self.cdrom_cue = cue;
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
            // Init - Command 0Ah --> INT3(stat late) --> INT2(stat)
            0x0A => {
                self.execute_init();
            }
            // Setmode - Command 0Eh,mode --> INT3(stat)
            0x0E => {
                self.execute_setmode();
            }
            // ReadN - Command 06h --> INT3(stat) --> INT1(stat) --> datablock
            0x06 => {
                self.execute_readn();
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

        self.requested_cursor = block_addr * SECTOR_SIZE;

        tracing::debug!(
            target: "psx_core::cdrom",
            min = minutes,
            sec = seconds,
            frame = frames,
            lba = block_addr,
            cursor = self.requested_cursor,
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
            cursor = self.current_cursor,
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

        // Stop any ongoing reads and clear queued read-related interrupts/data
        self.state = DriveState::Idle;
        self.read_in_progress = false;
        self.data_fifo.clear();
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

    fn execute_init(&mut self) {
        tracing::debug!(
            target: "psx_core::cdrom",
            "Init - resetting CD-ROM to initial state",
        );

        // Abort all pending operations
        self.interrupt_queue.clear();
        self.read_in_progress = false;
        self.state = DriveState::Idle;

        // Set default mode: 2048-byte user data reads (sector_size=0)
        self.mode = SetModeRegister(0x00);

        // Reset position to start
        self.requested_cursor = 0;
        self.current_cursor = 0;

        // Clear data FIFO
        self.data_fifo.clear();

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
            "ReadTOC - reading table of contents",
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
            cursor = self.requested_cursor,
            "SeekL",
        );

        self.current_cursor = self.requested_cursor;

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

        tracing::debug!(
            target: "psx_core::cdrom",
            mode = format!("{:08b}", mode_byte),
            speed = if self.mode.double_speed() { "double" } else { "normal" },
            xa_adpcm = if self.mode.xa_adpcm() { "on" } else { "off" },
            sector_size = if self.mode.sector_size() { "924" } else { "2048+12" },
            ignore_bit = if self.mode.ignore_bit() { "on" } else { "off" },
            xa_filter = if self.mode.xa_filter() { "on" } else { "off" },
            report = if self.mode.report() { "on" } else { "off" },
            autopause = if self.mode.autopause() { "on" } else { "off" },
            cdda = if self.mode.cdda() { "on" } else { "off" },
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

    fn execute_readn(&mut self) {
        tracing::debug!(
            target: "psx_core::cdrom",
            from = self.current_cursor,
            to = self.requested_cursor,
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
        let distance = if self.requested_cursor > self.current_cursor {
            self.requested_cursor - self.current_cursor
        } else {
            self.current_cursor - self.requested_cursor
        };

        // Rough seek timing: ~1ms per sector at minimum, with some overhead
        // For simplicity, using a fixed delay for now (should be distance-based)
        let seek_cycles = if distance == 0 {
            // Already at position, minimal delay
            FIRST_RESP_GENERIC_DELAY
        } else {
            // Realistic seek delay based on distance
            // Approximately 0.1-2 seconds for typical seeks
            // Using ~1ms per sector difference as a rough estimate
            let sectors_diff = distance / SECTOR_SIZE;
            (sectors_diff.max(1) * 33_868).min(SECOND_RESP_PAUSE_DELAY) // ~1ms per sector, max 2 seconds
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
        !self.cdrom_cue.is_empty()
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
            },
            REG_RDDATA_ADDR => {
                // RDDATA reads from the data FIFO, not result FIFO
                let value = self.data_fifo.pop_front().unwrap_or_else(|| {
                    tracing::warn!(
                        target: "psx_core::cdrom",
                        "RDDATA/data FIFO underflow on read",
                    );
                    0xFF
                });
                // If we've drained the sector buffer, deassert data request
                if self.data_fifo.is_empty() {
                    self.address.set_data_request(false);
                }
                tracing::trace!(
                    target: "psx_core::cdrom",
                    bank = self.address.current_bank(),
                    rdata = format!("{:02X}", value),
                    remaining = self.data_fifo.len(),
                    "CDROM RDDATA read",
                );
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
                tracing::debug!(
                    target: "psx_core::cdrom",
                    bank = self.address.current_bank(),
                    "Bank switched",
                );
            }
            REG_PARAMETER_ADDR if self.address.current_bank() == 0 => {
                self.parameter_fifo.push_back(value);
                // Parameter FIFO is non-empty now
                self.address.set_parameter_empty(false);
                tracing::debug!(
                    target: "psx_core::cdrom",
                    fifo = %format!("{:02X?}", self.parameter_fifo),
                    "Parameter FIFO write",
                );
            }
            REG_COMMAND_ADDR if self.address.current_bank() == 0 => {
                tracing::debug!(
                    target: "psx_core::cdrom",
                    command = format!("{:02X}", value),
                    "Command received",
                );
                self.execute_command(value);
            }
            REG_HCHPCTL_ADDR if self.address.current_bank() == 0 => {
                // Request Register - controls sector buffer read/write
                tracing::trace!(
                    target: "psx_core::cdrom",
                    value = format!("{:02X}", value),
                    want_data = (value & 0x80) != 0,
                    "HCHPCTL write",
                );
                // Bit 7: Want Data (sector buffer read request)
                // For now, we don't need to do anything special as our data FIFO
                // is always ready for reading when data is available
            }
            REG_HCLRCTL_ADDR if self.address.current_bank() == 1 => {
                // Clear specified IRQ flags
                let old_flags: u8 = self.hintsts.irq_flags().into();
                self.hintsts.set_irq_flags((old_flags & !value).into());

                if self.hclrctl.ack_bfwrdy_interrupt() || self.hclrctl.ack_bfempt_interrupt() {
                    tracing::error!(target: "psx_core::cdrom", "Unimplemented ACK for CDROM IRQ");
                }
            }
            REG_HINTMSK_ADDR_W if self.address.current_bank() == 1 => {
                self.hintmsk.0 = value & 0b0001_1111;
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
