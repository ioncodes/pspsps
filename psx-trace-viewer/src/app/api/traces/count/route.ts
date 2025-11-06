import { NextRequest, NextResponse } from 'next/server'
import { getDb } from '@/lib/db'
import { TraceFilters } from '@/types/trace'

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url)
  
  const filters: TraceFilters = {
    target: searchParams.get('target') || undefined,
    level: searchParams.get('level') || undefined,
    search: searchParams.get('search') || undefined,
  }

  const fieldFilters: Record<string, string> = {}
  for (const [key, value] of searchParams.entries()) {
    if (key.startsWith('field_') && value) {
      fieldFilters[key.substring(6)] = value
    }
  }

  try {
    const db = getDb()
    
    let query = `
      SELECT COUNT(*) as total_count
      FROM traces 
      WHERE 1=1
    `
    const values: unknown[] = []
    let paramIndex = 1

    if (filters.target) {
      query += ` AND target ILIKE $${paramIndex}`
      values.push(`%${filters.target}%`)
      paramIndex++
    }

    if (filters.level) {
      query += ` AND level = $${paramIndex}`
      values.push(filters.level)
      paramIndex++
    }

    if (filters.search) {
      query += ` AND (target ILIKE $${paramIndex} OR level ILIKE $${paramIndex} OR fields::text ILIKE $${paramIndex})`
      values.push(`%${filters.search}%`)
      paramIndex++
    }

    for (const [field, value] of Object.entries(fieldFilters)) {
      query += ` AND fields ->> $${paramIndex} ILIKE $${paramIndex + 1}`
      values.push(field, `%${value}%`)
      paramIndex += 2
    }

    const result = await db.query(query, values)
    const total = result.rows.length > 0 ? parseInt(result.rows[0].total_count) : 0

    return NextResponse.json({
      total
    })

  } catch (error) {
    console.error('Database error:', error)
    return NextResponse.json(
      { error: 'Failed to count traces' },
      { status: 500 }
    )
  }
}