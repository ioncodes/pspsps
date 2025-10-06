(Get-Content .\trace.json -Encoding Unicode) | Set-Content -Encoding UTF8 .\trace.json
cargo run --bin psx-trace-ingestor -- .\trace.json