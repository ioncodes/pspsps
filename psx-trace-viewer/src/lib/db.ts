import { Pool } from 'pg'

let pool: Pool | undefined

export function getDb() {
  if (!pool) {
    pool = new Pool({
      connectionString: process.env.DATABASE_URL,
      max: 20,
      idleTimeoutMillis: 30000,
      connectionTimeoutMillis: 2000,
    })
  }
  return pool
}

export async function closeDb() {
  if (pool) {
    await pool.end()
    pool = undefined
  }
}