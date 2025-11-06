use psx_core::mmu::bus::Bus32 as _;
use psx_core::psx::Psx;
use std::time::Instant;
use tracing_subscriber::Layer as _;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

static BIOS: &[u8] = include_bytes!("../../bios/SCPH1001.BIN");
const BIN_PATH: &str = "games\\Crash Bandicoot (USA)\\Crash Bandicoot (USA).bin";

fn main() {
    let targets = tracing_subscriber::filter::Targets::new()
        .with_target("psx_core::tty", LevelFilter::INFO)
        .with_target("psx_core::cdrom", LevelFilter::DEBUG);
    let fmt_layer = tracing_subscriber::fmt::layer().without_time().with_filter(targets);
    tracing_subscriber::registry().with(fmt_layer).init();

    let mut psx = Psx::new(BIOS);
    let bin_buffer = std::fs::read(BIN_PATH).expect("Failed to read BIN file");
    psx.load_cdrom(bin_buffer);

    let mut instruction_count = 0u64;
    let start_time = Instant::now();
    let mut last_report = Instant::now();

    loop {
        let _ = psx.step().unwrap_or_else(|_| {
            println!("Registers: {:08X?}", psx.cpu.registers);
            let pc = psx.cpu.pc - 4;
            println!("PC: {:08X}", pc);
            let instr = psx.cpu.mmu.read_u32(pc);
            println!("Memory @ PC: {:08X?}", instr);
            println!("Last instruction: {:02X}", instr & 0b111111);

            std::process::exit(1)
        });
        instruction_count += 1;

        if last_report.elapsed().as_secs() >= 1 {
            let elapsed = start_time.elapsed();
            let ips = instruction_count as f64 / elapsed.as_secs_f64();
            println!(
                "Instructions executed: {}, MIPS: {:.2}",
                instruction_count,
                ips / 1_000_000.0
            );
            last_report = Instant::now();
        }
    }
}
