use crate::cpu::{Cpu, lut};
use crate::mmu::Mmu;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type HookHandler = fn(&Cpu, &Mmu);

lazy_static! {
    pub(crate) static ref HOOKS: HashMap<u32, HookHandler> = {
        let mut hooks = HashMap::new();
        hooks.insert(0xBFC0D460, bios_putchar as HookHandler);
        hooks
    };
    pub static ref TTY_BUFFER: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    pub static ref TTY_LINE_BUFFER: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

fn bios_putchar(cpu: &Cpu, _mmu: &Mmu) {
    let value = cpu.registers[lut::register_to_index("$a0")] as u8 as char;

    TTY_BUFFER.lock().unwrap().push(value);

    if value == '\n' {
        let line = TTY_LINE_BUFFER.lock().unwrap().clone();
        tracing::info!(target: "psx_core::tty", "{}", line);
        TTY_LINE_BUFFER.lock().unwrap().clear();
    } else {
        TTY_LINE_BUFFER.lock().unwrap().push(value);
    }
}
