import { NextRequest, NextResponse } from 'next/server'
import { getDb } from '@/lib/db'
import { TraceFilters } from '@/types/trace'

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url)
  
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
      SELECT COUNT(*) as total_count
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