(Get-Content .\trace.dump -Encoding Unicode) | Set-Content -Encoding UTF8 .\trace.dump
cargo run --bin psx-trace-ingestor -- .\trace.dump