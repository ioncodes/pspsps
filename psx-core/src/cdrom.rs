use crate::mmu::bus::{Bus8, Bus16, Bus32};
use psx_cdrom_sys::*;
use std::sync::atomic::{AtomicBool, Ordering};

// CDROM register addresses
pub const CDROM_ADDR_START: u32 = 0x1F80_1800;
pub const CDROM_ADDR_END: u32 = 0x1F80_1803;

// Global flag for CDROM IRQ (used by C callback)
static CDROM_IRQ_PENDING: AtomicBool = AtomicBool::new(false);

// C callback function that the CDROM library will call
extern "C" fn cdrom_irq_callback(_udata: *mut std::ffi::c_void) {
    CDROM_IRQ_PENDING.store(true, Ordering::Release);
}

pub struct Cdrom {
    /// Pointer to the cdrom instance
    cdrom_ptr: *mut psx_cdrom_t,
}

impl Cdrom {
    pub fn new() -> Self {
        unsafe {
            let cdrom_ptr = psx_cdrom_create();
            assert!(!cdrom_ptr.is_null(), "Failed to create CDROM instance");

            psx_cdrom_init(cdrom_ptr, Some(cdrom_irq_callback), std::ptr::null_mut());

            Self { cdrom_ptr }
        }
    }

    /// Update the CDROM controller with the given number of CPU cycles
    pub fn update(&mut self, cycles: i32) {
        unsafe {
            psx_cdrom_update(self.cdrom_ptr, cycles);
        }
    }

    /// Open a disc image file
    pub fn open(&mut self, path: &str) -> bool {
        let c_path = std::ffi::CString::new(path).expect("Failed to convert path to CString");
        unsafe {
            let result = psx_cdrom_open(self.cdrom_ptr, c_path.as_ptr());
            result == 0
        }
    }

    /// Get audio samples from the CDROM into the provided buffer
    pub fn get_audio_samples(&mut self, buffer: &mut [u8]) {
        unsafe {
            psx_cdrom_get_audio_samples(
                self.cdrom_ptr,
                buffer.as_mut_ptr() as *mut std::ffi::c_void,
                buffer.len(),
            );
        }
    }

    /// Check if there's a pending IRQ and clear it
    pub fn check_and_clear_irq(&self) -> bool {
        CDROM_IRQ_PENDING.swap(false, Ordering::AcqRel)
    }
}

impl Drop for Cdrom {
    fn drop(&mut self) {
        unsafe {
            if !self.cdrom_ptr.is_null() {
                psx_cdrom_destroy(self.cdrom_ptr);
            }
        }
    }
}

impl Bus8 for Cdrom {
    fn read_u8(&mut self, address: u32) -> u8 {
        debug_assert!(
            address >= CDROM_ADDR_START && address <= CDROM_ADDR_END,
            "CDROM read address out of range: {:08X}",
            address
        );

        unsafe { psx_cdrom_read8(self.cdrom_ptr, address) as u8 }
    }

    fn write_u8(&mut self, address: u32, value: u8) {
        debug_assert!(
            address >= CDROM_ADDR_START && address <= CDROM_ADDR_END,
            "CDROM write address out of range: {:08X}",
            address
        );

        unsafe {
            psx_cdrom_write8(self.cdrom_ptr, address, value as u32);
        }
    }
}

impl Bus16 for Cdrom {
    fn read_u16(&mut self, address: u32) -> u16 {
        debug_assert!(
            address >= CDROM_ADDR_START && address <= CDROM_ADDR_END,
            "CDROM read address out of range: {:08X}",
            address
        );

        unsafe { psx_cdrom_read16(self.cdrom_ptr, address) as u16 }
    }

    fn write_u16(&mut self, address: u32, value: u16) {
        debug_assert!(
            address >= CDROM_ADDR_START && address <= CDROM_ADDR_END,
            "CDROM write address out of range: {:08X}",
            address
        );

        unsafe {
            psx_cdrom_write16(self.cdrom_ptr, address, value as u32);
        }
    }
}

impl Bus32 for Cdrom {
    fn read_u32(&mut self, address: u32) -> u32 {
        debug_assert!(
            address >= CDROM_ADDR_START && address <= CDROM_ADDR_END,
            "CDROM read address out of range: {:08X}",
            address
        );

        unsafe { psx_cdrom_read32(self.cdrom_ptr, address) }
    }

    fn write_u32(&mut self, address: u32, value: u32) {
        debug_assert!(
            address >= CDROM_ADDR_START && address <= CDROM_ADDR_END,
            "CDROM write address out of range: {:08X}",
            address
        );

        unsafe {
            psx_cdrom_write32(self.cdrom_ptr, address, value);
        }
    }
}
