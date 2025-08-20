use crate::cpu::{Cpu, lut};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type HookHandler = fn(&mut Cpu);

lazy_static! {
    pub(crate) static ref HOOKS: HashMap<u32, HookHandler> = {
        let mut hooks = HashMap::new();
        hooks.insert(0xBFC0D460, bios_putchar as HookHandler); // TODO: just hook the BIOS putchar function? would make it compatible with other BIOS
        hooks.insert(0x000000A0, bios_function_call_a as HookHandler);
        hooks.insert(0x800000A0, bios_function_call_a as HookHandler);
        hooks.insert(0x000000B0, bios_function_call_b as HookHandler);
        hooks.insert(0x800000B0, bios_function_call_b as HookHandler);
        hooks.insert(0x000000C0, bios_function_call_c as HookHandler);
        hooks.insert(0x800000C0, bios_function_call_c as HookHandler);
        hooks
    };
    pub static ref TTY_BUFFER: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    pub static ref TTY_LINE_BUFFER: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

fn bios_putchar(cpu: &mut Cpu) {
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

fn bios_function_call_a(cpu: &mut Cpu) {
    tracing::debug!(target: "psx_core::bios", "BIOS @ A({:08X})", cpu.read_register(9));
}

fn bios_function_call_b(cpu: &mut Cpu) {
    tracing::debug!(target: "psx_core::bios", "BIOS @ B({:08X})", cpu.read_register(9));
}

fn bios_function_call_c(cpu: &mut Cpu) {
    tracing::debug!(target: "psx_core::bios", "BIOS @ C({:08X})", cpu.read_register(9));
}
