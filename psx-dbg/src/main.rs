use psx_core::cpu::decoder::Instruction;

static BIOS: &[u8] = include_bytes!("../../bios/SCPH1000.BIN");

fn main() {
    let mut cpu = psx_core::cpu::Cpu::new();

    for chunk in BIOS.chunks_exact(4) {
        let instr_raw = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let instr = Instruction::decode(instr_raw);
        println!("{} [{}]", instr, cpu);
        (instr.handler)(&instr, &mut cpu);
    }
}
