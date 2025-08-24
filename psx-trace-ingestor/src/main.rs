use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use tokio_postgres::{Client, NoTls};

const BATCH_SIZE: usize = 5_000_000;

#[derive(Debug)]
struct TraceRecord {
    level: String,
    fields: Value,
    target: String,
}

impl TraceRecord {
    fn from_json(line: &str) -> Option<Self> {
        let obj: Value = serde_json::from_str(line).ok()?;
        Some(TraceRecord {
            level: obj.get("level")?.as_str().unwrap_or("").to_string(),
            fields: obj
                .get("fields")
                .cloned()
                .unwrap_or(Value::Object(serde_json::Map::new())),
            target: obj.get("target")?.as_str().unwrap_or("").to_string(),
        })
    }
}

async fn init_db(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    client.execute("DROP TABLE IF EXISTS traces", &[]).await?;

    client
        .execute(
            "CREATE TABLE IF NOT EXISTS traces (
            id SERIAL PRIMARY KEY,
            level TEXT NOT NULL,
            fields JSONB NOT NULL,
            target TEXT NOT NULL
        )",
            &[],
        )
        .await?;
    Ok(())
}

async fn ingest_batch(
    client: &Client, batch: &[TraceRecord],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut csv_data = Vec::new();

    for record in batch {
        let fields_json = serde_json::to_string(&record.fields)?;
        writeln!(
            csv_data,
            "{}\t{}\t{}",
            record.level, fields_json, record.target
        )?;
    }

    let copy_stmt = "COPY traces (level, fields, target) FROM STDIN WITH (FORMAT csv, DELIMITER E'\\t', QUOTE E'\\b')";
    let sink: tokio_postgres::CopyInSink<bytes::Bytes> = client.copy_in(copy_stmt).await?;

    use futures_util::SinkExt;
    let mut sink = Box::pin(sink);
    sink.send(bytes::Bytes::from(csv_data))
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    sink.close()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    Ok(())
}

async fn ingest_file(client: &Client, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let mut batch = Vec::with_capacity(BATCH_SIZE);
    let mut total = 0;

    for line in reader.lines() {
        let line = line?;
        if let Some(record) = TraceRecord::from_json(&line) {
            batch.push(record);

            if batch.len() >= BATCH_SIZE {
                ingest_batch(client, &batch).await?;
                total += batch.len();
                println!("Pushed {} records", total);
                batch.clear();
            }
        }
    }

    if !batch.is_empty() {
        ingest_batch(client, &batch).await?;
        total += batch.len();
    }

    println!("Done: {} records", total);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=psx password=psx dbname=psx", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client.execute("SET synchronous_commit = OFF", &[]).await?;

    init_db(&client).await?;

    let input_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("Usage: psx-trace-ingestor <input_file>"));

    let time_start = std::time::Instant::now();
    ingest_file(&client, &input_file).await?;
    let duration = time_start.elapsed();
    println!("Ingestion completed in: {:?}", duration);

    Ok(())
}
