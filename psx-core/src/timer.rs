use crate::mmu::bus::{Bus8, Bus16, Bus32};
use proc_bitfield::bitfield;

// PSX-SPX: "1F801100h+N*10h - Timer 0..2 Current Counter Value (R/W)"
crate::define_addr!(TIMER0_COUNTER_ADDR, 0x1F80_1100, 0, 0x04, 0x10);
crate::define_addr!(TIMER1_COUNTER_ADDR, 0x1F80_1100, 1, 0x04, 0x10);
crate::define_addr!(TIMER2_COUNTER_ADDR, 0x1F80_1100, 2, 0x04, 0x10);

// PSX-SPX: "1F801104h+N*10h - Timer 0..2 Counter Mode (R/W)"
crate::define_addr!(TIMER0_MODE_ADDR, 0x1F80_1104, 0, 0x04, 0x10);
crate::define_addr!(TIMER1_MODE_ADDR, 0x1F80_1104, 1, 0x04, 0x10);
crate::define_addr!(TIMER2_MODE_ADDR, 0x1F80_1104, 2, 0x04, 0x10);

// PSX-SPX: "1F801108h+N*10h - Timer 0..2 Counter Target Value (R/W)"
crate::define_addr!(TIMER0_TARGET_ADDR, 0x1F80_1108, 0, 0x04, 0x10);
crate::define_addr!(TIMER1_TARGET_ADDR, 0x1F80_1108, 1, 0x04, 0x10);
crate::define_addr!(TIMER2_TARGET_ADDR, 0x1F80_1108, 2, 0x04, 0x10);

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct TimerMode(pub u32): Debug, FromStorage, IntoStorage, DerefStorage {
        pub sync_enable: bool @ 0,
        pub sync_mode: u8 @ 1..=2,
        pub reset_counter: bool @ 3,
        pub irq_at_target: bool @ 4,
        pub irq_at_overflow: bool @ 5,
        pub irq_repeat: bool @ 6,
        pub irq_toggle_mode: bool @ 7,
        pub clock_source: u8 @ 8..=9,
        pub irq_request: bool @ 10,
        pub reached_target: bool @ 11,
        pub reached_overflow: bool @ 12,
    }
}

pub struct Timer<const TIMER_ID: u8> {
    pub counter: u16,
    pub mode: TimerMode,
    pub target: u16,
    divider: u32,
    sync_reached: bool,
}

impl<const TIMER_ID: u8> Timer<TIMER_ID> {
    pub fn new() -> Self {
        Self {
            counter: 0,
            mode: TimerMode(0),
            target: 0,
            divider: 0,
            sync_reached: false,
        }
    }

    pub fn tick(&mut self, cycles: usize, in_hblank: bool, in_vblank: bool) -> bool {
        let mut irq_triggered = false;

        // Determine which blanking signal to use based on timer
        let in_blank = match TIMER_ID {
            0 => in_hblank, // Timer 0 uses HBL
            1 => in_vblank, // Timer 1 uses VBL
            _ => false,     // Timer 2 doesn't use blanking
        };

        let clock_divider = Self::get_clock_divider(&self.mode);
        self.divider += cycles as u32;

        while self.divider >= clock_divider {
            self.divider -= clock_divider;

            if !self.should_increment(in_blank) {
                continue;
            }

            let old_counter = self.counter;
            self.counter = self.counter.wrapping_add(1);

            // Check for target
            if self.counter == self.target {
                self.mode.set_reached_target(true);

                if self.mode.irq_at_target() {
                    irq_triggered = self.trigger_irq();
                }

                if self.mode.reset_counter() {
                    self.counter = 0;
                }
            }

            // Check for overflow
            if old_counter == 0xFFFF && self.counter == 0 {
                self.mode.set_reached_overflow(true);

                if self.mode.irq_at_overflow() {
                    irq_triggered = self.trigger_irq();
                }
            }
        }

        irq_triggered
    }

    fn get_clock_divider(mode: &TimerMode) -> u32 {
        match TIMER_ID {
            0 => match mode.clock_source() {
                0 | 2 => 1,
                1 | 3 => 1, // TODO: dotclock
                _ => 1,
            },
            1 => match mode.clock_source() {
                0 | 2 => 1,
                1 | 3 => 1, // TODO: hblank
                _ => 1,
            },
            2 => match mode.clock_source() {
                0 | 1 => 1,
                2 | 3 => 8, // System clock / 8
                _ => 1,
            },
            _ => 1,
        }
    }

    fn should_increment(&mut self, in_blank: bool) -> bool {
        if !self.mode.sync_enable() {
            return true;
        }

        if TIMER_ID == 2 {
            // PSX-SPX: "Timer 2: Mode 0,3 = Stop counter, Mode 1,2 = Free Run"
            return matches!(self.mode.sync_mode(), 1 | 2);
        }

        // Timer 0/1 sync modes
        match self.mode.sync_mode() {
            0 => !in_blank, // Pause during blank
            1 => {
                // Reset at blank
                if in_blank && !self.sync_reached {
                    self.counter = 0;
                    self.sync_reached = true;
                } else if !in_blank {
                    self.sync_reached = false;
                }
                true
            }
            2 => {
                // Reset at blank and pause outside
                if in_blank && !self.sync_reached {
                    self.counter = 0;
                    self.sync_reached = true;
                } else if !in_blank {
                    self.sync_reached = false;
                }

                in_blank
            }
            3 => {
                // Pause until first blank, then free run
                if in_blank {
                    self.sync_reached = true;
                }
                
                self.sync_reached
            }
            _ => true,
        }
    }

    fn trigger_irq(&mut self) -> bool {
        // One-shot mode: Only trigger if IRQ was not already triggered
        if !self.mode.irq_repeat() && !self.mode.irq_request() {
            return false;
        }

        // PSX-SPX: "Bit7=0: Short Bit10=0 Pulse"
        if !self.mode.irq_toggle_mode() {
            self.mode.set_irq_request(false);
            true
        } else {
            // PSX-SPX: "Bit7=1: Toggle Bit10 on/off"
            self.mode.set_irq_request(!self.mode.irq_request());
            !self.mode.irq_request()
        }
    }

    pub fn read_mode(&mut self) -> u32 {
        let value = self.mode.0;

        // 11    Reached Target Value    (0=No, 1=Yes) (Reset after Reading)        (R)
        // 12    Reached FFFFh Value     (0=No, 1=Yes) (Reset after Reading)        (R)
        self.mode.set_reached_target(false);
        self.mode.set_reached_overflow(false);

        value
    }

    pub fn write_mode(&mut self, value: u32) {
        self.mode = TimerMode(value);
        self.counter = 0;
        self.divider = 0;
        self.sync_reached = false;

        // 10    Interrupt Request       (0=Yes, 1=No) (Set after Writing)    (W=1) (R)
        self.mode.set_irq_request(true);
    }
}

pub struct Timers {
    pub timer0: Timer<0>,
    pub timer1: Timer<1>,
    pub timer2: Timer<2>,
}

impl Timers {
    pub fn new() -> Self {
        Self {
            timer0: Timer::new(),
            timer1: Timer::new(),
            timer2: Timer::new(),
        }
    }

    pub fn tick(&mut self, cycles: usize, in_hblank: bool, in_vblank: bool) -> (bool, bool, bool) {
        let tmr0_irq = self.timer0.tick(cycles, in_hblank, in_vblank);
        let tmr1_irq = self.timer1.tick(cycles, in_hblank, in_vblank);
        let tmr2_irq = self.timer2.tick(cycles, in_hblank, in_vblank);
        (tmr0_irq, tmr1_irq, tmr2_irq)
    }
}

impl Bus8 for Timers {
    fn read_u8(&mut self, address: u32) -> u8 {
        let offset = address & 0b11;
        match address {
            TIMER0_COUNTER_ADDR_START..=TIMER0_COUNTER_ADDR_END => (self.timer0.counter as u32 >> (offset * 8)) as u8,
            TIMER0_MODE_ADDR_START..=TIMER0_MODE_ADDR_END => (self.timer0.read_mode() >> (offset * 8)) as u8,
            TIMER0_TARGET_ADDR_START..=TIMER0_TARGET_ADDR_END => (self.timer0.target as u32 >> (offset * 8)) as u8,

            TIMER1_COUNTER_ADDR_START..=TIMER1_COUNTER_ADDR_END => (self.timer1.counter as u32 >> (offset * 8)) as u8,
            TIMER1_MODE_ADDR_START..=TIMER1_MODE_ADDR_END => (self.timer1.read_mode() >> (offset * 8)) as u8,
            TIMER1_TARGET_ADDR_START..=TIMER1_TARGET_ADDR_END => (self.timer1.target as u32 >> (offset * 8)) as u8,

            TIMER2_COUNTER_ADDR_START..=TIMER2_COUNTER_ADDR_END => (self.timer2.counter as u32 >> (offset * 8)) as u8,
            TIMER2_MODE_ADDR_START..=TIMER2_MODE_ADDR_END => (self.timer2.read_mode() >> (offset * 8)) as u8,
            TIMER2_TARGET_ADDR_START..=TIMER2_TARGET_ADDR_END => (self.timer2.target as u32 >> (offset * 8)) as u8,

            _ => unreachable!(),
        }
    }

    fn write_u8(&mut self, address: u32, value: u8) {
        // (w32) write full 32bits (left-shifted if address isn't word-aligned)
        let offset = address & 0b11;
        let aligned_address = address & !0b11;
        let shifted_value = (value as u32) << (offset * 8);
        self.write_u32(aligned_address, shifted_value);
    }
}

impl Bus16 for Timers {
    fn read_u16(&mut self, address: u32) -> u16 {
        match address {
            TIMER0_COUNTER_ADDR_START..=TIMER0_COUNTER_ADDR_END => self.timer0.counter,
            TIMER0_MODE_ADDR_START..=TIMER0_MODE_ADDR_END => self.timer0.read_mode() as u16,
            TIMER0_TARGET_ADDR_START..=TIMER0_TARGET_ADDR_END => self.timer0.target,

            TIMER1_COUNTER_ADDR_START..=TIMER1_COUNTER_ADDR_END => self.timer1.counter,
            TIMER1_MODE_ADDR_START..=TIMER1_MODE_ADDR_END => self.timer1.read_mode() as u16,
            TIMER1_TARGET_ADDR_START..=TIMER1_TARGET_ADDR_END => self.timer1.target,

            TIMER2_COUNTER_ADDR_START..=TIMER2_COUNTER_ADDR_END => self.timer2.counter,
            TIMER2_MODE_ADDR_START..=TIMER2_MODE_ADDR_END => self.timer2.read_mode() as u16,
            TIMER2_TARGET_ADDR_START..=TIMER2_TARGET_ADDR_END => self.timer2.target,

            _ => unreachable!(),
        }
    }

    fn write_u16(&mut self, address: u32, value: u16) {
        // (w32) write full 32bits (left-shifted if address isn't word-aligned)
        let offset = address & 0b11;
        let aligned_address = address & !0b11;
        let shifted_value = (value as u32) << (offset * 8);
        self.write_u32(aligned_address, shifted_value);
    }
}

impl Bus32 for Timers {
    fn read_u32(&mut self, address: u32) -> u32 {
        match address {
            TIMER0_COUNTER_ADDR_START..=TIMER0_COUNTER_ADDR_END => self.timer0.counter as u32,
            TIMER0_MODE_ADDR_START..=TIMER0_MODE_ADDR_END => self.timer0.read_mode(),
            TIMER0_TARGET_ADDR_START..=TIMER0_TARGET_ADDR_END => self.timer0.target as u32,

            TIMER1_COUNTER_ADDR_START..=TIMER1_COUNTER_ADDR_END => self.timer1.counter as u32,
            TIMER1_MODE_ADDR_START..=TIMER1_MODE_ADDR_END => self.timer1.read_mode(),
            TIMER1_TARGET_ADDR_START..=TIMER1_TARGET_ADDR_END => self.timer1.target as u32,

            TIMER2_COUNTER_ADDR_START..=TIMER2_COUNTER_ADDR_END => self.timer2.counter as u32,
            TIMER2_MODE_ADDR_START..=TIMER2_MODE_ADDR_END => self.timer2.read_mode(),
            TIMER2_TARGET_ADDR_START..=TIMER2_TARGET_ADDR_END => self.timer2.target as u32,

            _ => unreachable!(),
        }
    }

    fn write_u32(&mut self, address: u32, value: u32) {
        match address {
            TIMER0_COUNTER_ADDR_START..=TIMER0_COUNTER_ADDR_END => {
                self.timer0.counter = value as u16;
            }
            TIMER0_MODE_ADDR_START..=TIMER0_MODE_ADDR_END => {
                self.timer0.write_mode(value);
            }
            TIMER0_TARGET_ADDR_START..=TIMER0_TARGET_ADDR_END => {
                self.timer0.target = value as u16;
            }

            TIMER1_COUNTER_ADDR_START..=TIMER1_COUNTER_ADDR_END => {
                self.timer1.counter = value as u16;
            }
            TIMER1_MODE_ADDR_START..=TIMER1_MODE_ADDR_END => {
                self.timer1.write_mode(value);
            }
            TIMER1_TARGET_ADDR_START..=TIMER1_TARGET_ADDR_END => {
                self.timer1.target = value as u16;
            }

            TIMER2_COUNTER_ADDR_START..=TIMER2_COUNTER_ADDR_END => {
                self.timer2.counter = value as u16;
            }
            TIMER2_MODE_ADDR_START..=TIMER2_MODE_ADDR_END => {
                self.timer2.write_mode(value);
            }
            TIMER2_TARGET_ADDR_START..=TIMER2_TARGET_ADDR_END => {
                self.timer2.target = value as u16;
            }

            _ => unreachable!(),
        }
    }
}
