use crate::cpu::{Cpu, lut};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

type HookHandler = fn(&mut Cpu);

static HOOKS: OnceLock<HashMap<u32, HookHandler>> = OnceLock::new();
static TTY_BUFFER: OnceLock<Mutex<String>> = OnceLock::new();
static TTY_LINE_BUFFER: OnceLock<Mutex<String>> = OnceLock::new();

pub fn cpu_hooks() -> &'static HashMap<u32, HookHandler> {
    HOOKS.get_or_init(|| {
        let mut hooks = HashMap::new();
        hooks.insert(0xBFC0D460, bios_putchar as HookHandler); // TODO: just hook the BIOS putchar function? would make it compatible with other BIOS
        hooks.insert(0x000000A0, bios_function_call_a as HookHandler);
        hooks.insert(0x800000A0, bios_function_call_a as HookHandler);
        hooks.insert(0x000000B0, bios_function_call_b as HookHandler);
        hooks.insert(0x800000B0, bios_function_call_b as HookHandler);
        hooks.insert(0x000000C0, bios_function_call_c as HookHandler);
        hooks.insert(0x800000C0, bios_function_call_c as HookHandler);
        hooks
    })
}

pub fn tty_buffer() -> &'static Mutex<String> {
    TTY_BUFFER.get_or_init(|| Mutex::new(String::new()))
}

pub fn tty_line_buffer() -> &'static Mutex<String> {
    TTY_LINE_BUFFER.get_or_init(|| Mutex::new(String::new()))
}

fn bios_putchar(cpu: &mut Cpu) {
    let value = cpu.registers[lut::register_to_index("$a0")] as u8 as char;

    tty_buffer().lock().unwrap().push(value);

    if value == '\n' {
        let mut line_buffer = tty_line_buffer().lock().unwrap();
        tracing::info!(target: "psx_core::tty", "{}", line_buffer);
        line_buffer.clear();
    } else {
        tty_line_buffer().lock().unwrap().push(value);
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
