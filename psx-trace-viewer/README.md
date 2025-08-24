# PSX Trace Viewer

A beautiful trace explorer with filtering capabilities. To be used along with `psx-ingestor` and `tracing-subscriber` (Rust).

## Database Schema

The application expects a PostgreSQL table named `traces` with the following structure:

```sql
CREATE TABLE traces (
    id SERIAL PRIMARY KEY,
    target VARCHAR NOT NULL,
    level VARCHAR NOT NULL,
    fields JSONB NOT NULL
);
```

Please ensure that you wipe the database before ingesting a new trace. `psx-ingestor` takes care of ingesting the data, wiping pre-existing data and also creating the schema. The ingestor expects specifically `tracing-subscribers`' JSON stdout (JSON objects line by line, not an actual JSON file).

## Setup

1. **Install dependencies:**
   ```bash
   npm install
   ```

2. **Run the PostgreSQL database:**
   ```bash
   docker run --rm --name psx-trace-viewer -e POSTGRES_USER=psx -e POSTGRES_PASSWORD=psx -e POSTGRES_DB=psx -p 5432:5432 -d postgres:16
   ```

3. **Update environment variables (optional):**
   Update the `.env.local`, default is:
   ```
   DATABASE_URL=postgresql://psx:psx@localhost:5432/psx
   ```

5. **Run the development server:**
   ```bash
   npm run dev
   # or for prod
   npm run build
   npm start
   ```

6. **Open** [http://localhost:3000](http://localhost:3000) in your browser