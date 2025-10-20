use psx_core::psx::Psx;
use std::time::Instant;
use tracing_subscriber::Layer as _;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

static BIOS: &[u8] = include_bytes!("../../bios/SCPH1000.BIN");

fn main() {
    let targets =
        tracing_subscriber::filter::Targets::new().with_target("psx_core::tty", LevelFilter::INFO);
    let fmt_layer = tracing_subscriber::fmt::layer()
        .without_time()
        .with_filter(targets);
    tracing_subscriber::registry().with(fmt_layer).init();

    let mut psx = Psx::new(BIOS);
    psx.sideload_exe(include_bytes!("../../tests/amidog/psxtest_cpu.exe").to_vec());

    let mut instruction_count = 0u64;
    let start_time = Instant::now();
    let mut last_report = Instant::now();

    loop {
        let _ = psx.step().unwrap_or_else(|_| std::process::exit(1));
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
