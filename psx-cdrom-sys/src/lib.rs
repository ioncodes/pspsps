#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdrom_create() {
        unsafe {
            let cdrom = psx_cdrom_create();
            assert!(!cdrom.is_null());
            psx_cdrom_destroy(cdrom);
        }
    }
}
