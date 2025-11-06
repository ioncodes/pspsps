import { NextResponse } from 'next/server'
import { getDb } from '@/lib/db'

export async function GET() {
  try {
    const db = getDb()
    
    const [fieldsResult, levelsResult, targetsResult] = await Promise.all([
      db.query(`
        SELECT DISTINCT jsonb_object_keys(fields) as field_name 
        FROM traces 
        WHERE fields IS NOT NULL 
        ORDER BY field_name
      `),
      db.query('SELECT DISTINCT level FROM traces WHERE level IS NOT NULL ORDER BY level'),
      db.query('SELECT DISTINCT target FROM traces WHERE target IS NOT NULL ORDER BY target')
    ])

    return NextResponse.json({
      fields: fieldsResult.rows.map(row => row.field_name),
      levels: levelsResult.rows.map(row => row.level),
      targets: targetsResult.rows.map(row => row.target)
    })

  } catch (error) {
    console.error('Database error:', error)
    return NextResponse.json(
      { error: 'Failed to fetch field options' },
      { status: 500 }
    )
  }
}