use crate::cpu::Cpu;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

type HookHandler = fn(&mut Cpu);

static HOOKS: OnceLock<HashMap<u32, HookHandler>> = OnceLock::new();
static TTY_BUFFER: OnceLock<Mutex<String>> = OnceLock::new();
static TTY_LINE_BUFFER: OnceLock<Mutex<String>> = OnceLock::new();

pub fn cpu_hooks() -> &'static HashMap<u32, HookHandler> {
    HOOKS.get_or_init(|| {
        let mut hooks = HashMap::new();
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

fn bios_function_call_a(cpu: &mut Cpu) {
    if cpu.read_register(9) == 0x3C {
        bios_putchar(cpu);
    }

    tracing::debug!(target: "psx_core::bios", function = %format!("A({:08X})", cpu.read_register(9)), "BIOS function called");
}

fn bios_function_call_b(cpu: &mut Cpu) {
    if cpu.read_register(9) == 0x3D {
        bios_putchar(cpu);
    }

    tracing::debug!(target: "psx_core::bios", function = %format!("B({:08X})", cpu.read_register(9)), "BIOS function called");
}

fn bios_function_call_c(cpu: &mut Cpu) {
    tracing::debug!(target: "psx_core::bios", function = %format!("C({:08X})", cpu.read_register(9)), "BIOS function called");
}

fn bios_putchar(cpu: &mut Cpu) {
    let mut value = cpu.read_register(crate::regidx("$a0")) as u8 as char;
    if value == '\r' {
        value = '\n';
    }

    tty_buffer().lock().unwrap().push(value);

    if value == '\n' {
        let mut line_buffer = tty_line_buffer().lock().unwrap();
        tracing::info!(target: "psx_core::tty", "{}", line_buffer);
        line_buffer.clear();
    } else {
        tty_line_buffer().lock().unwrap().push(value);
    }
}
