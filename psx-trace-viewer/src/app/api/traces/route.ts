import { NextRequest, NextResponse } from 'next/server'
import { getDb } from '@/lib/db'
import { TraceFilters } from '@/types/trace'

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url)
  const page = parseInt(searchParams.get('page') || '1')
  const limit = Math.min(parseInt(searchParams.get('limit') || '200'), 1000)
  
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
      SELECT id, target, level, fields
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

    query += ` ORDER BY id LIMIT $${paramIndex} OFFSET $${paramIndex + 1}`
    values.push(limit, (page - 1) * limit)

    console.log('Executing query:', query, values)
    const result = await db.query(query, values)
    
    const traces = result.rows.map(row => ({
      id: row.id,
      target: row.target,
      level: row.level,
      fields: row.fields
    }))

    return NextResponse.json({
      traces,
      page,
      limit,
      hasMore: traces.length === limit
    })

  } catch (error) {
    console.error('Database error:', error)
    return NextResponse.json(
      { error: 'Failed to fetch traces' },
      { status: 500 }
    )
  }
}