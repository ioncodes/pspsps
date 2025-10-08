import { NextRequest, NextResponse } from 'next/server'
import { getDb } from '@/lib/db'
import { TraceFilters } from '@/types/trace'

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url)
  const page = parseInt(searchParams.get('page') || '1')
  const limit = Math.min(parseInt(searchParams.get('limit') || '200'), 1000)
  
  const filters: TraceFilters = {
    target: searchParams.get('target') || undefined,
    targetMode: (searchParams.get('targetMode') as 'wildcard' | 'exact') || 'wildcard',
    level: searchParams.get('level') || undefined,
    levelMode: (searchParams.get('levelMode') as 'wildcard' | 'exact') || 'wildcard',
    search: searchParams.get('search') || undefined,
    searchMode: (searchParams.get('searchMode') as 'wildcard' | 'exact') || 'wildcard',
  }

  const fieldFilters: Record<string, string> = {}
  const fieldModes: Record<string, 'wildcard' | 'exact'> = {}
  for (const [key, value] of searchParams.entries()) {
    if (key.startsWith('field_') && !key.endsWith('Mode') && value) {
      fieldFilters[key.substring(6)] = value
    }
    if (key.startsWith('field_') && key.endsWith('Mode')) {
      const fieldName = key.substring(6, key.length - 4)
      fieldModes[fieldName] = value as 'wildcard' | 'exact'
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
      const targetMode = filters.targetMode || 'wildcard'
      if (targetMode === 'exact') {
        query += ` AND target = $${paramIndex}`
        values.push(filters.target)
      } else {
        query += ` AND target ILIKE $${paramIndex}`
        values.push(`%${filters.target}%`)
      }
      paramIndex++
    }

    if (filters.level) {
      const levelMode = filters.levelMode || 'wildcard'
      if (levelMode === 'exact') {
        query += ` AND level = $${paramIndex}`
        values.push(filters.level)
      } else {
        query += ` AND level ILIKE $${paramIndex}`
        values.push(`%${filters.level}%`)
      }
      paramIndex++
    }

    if (filters.search) {
      const searchMode = filters.searchMode || 'wildcard'
      if (searchMode === 'exact') {
        query += ` AND (target = $${paramIndex} OR level = $${paramIndex} OR fields::text = $${paramIndex})`
        values.push(filters.search)
      } else {
        query += ` AND (target ILIKE $${paramIndex} OR level ILIKE $${paramIndex} OR fields::text ILIKE $${paramIndex})`
        values.push(`%${filters.search}%`)
      }
      paramIndex++
    }

    for (const [field, value] of Object.entries(fieldFilters)) {
      const mode = fieldModes[field] || 'wildcard'
      if (mode === 'exact') {
        query += ` AND fields ->> $${paramIndex} = $${paramIndex + 1}`
        values.push(field, value)
      } else {
        query += ` AND fields ->> $${paramIndex} ILIKE $${paramIndex + 1}`
        values.push(field, `%${value}%`)
      }
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