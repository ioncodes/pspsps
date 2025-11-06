use crate::cpu::Cpu;
use crate::mmu::Mmu;
use lazy_static::lazy_static;
use std::collections::HashMap;

type HookHandler = fn(&Cpu, &Mmu);

lazy_static! {
    pub static ref HOOKS: HashMap<u32, HookHandler> = {
        let mut hooks = HashMap::new();
        hooks.insert(0xBFC00D90, bios_printf as HookHandler);
        hooks
    };
}

fn bios_printf(cpu: &Cpu, mmu: &Mmu) {
    tracing::debug!(target: "psx_core::bios", "printf called with registers: {:?}", cpu.registers);
}
