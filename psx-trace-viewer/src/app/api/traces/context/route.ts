import { NextRequest, NextResponse } from 'next/server'
import { getDb } from '@/lib/db'

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url)
  const traceId = searchParams.get('id')
  
  if (!traceId) {
    return NextResponse.json(
      { error: 'Trace ID is required' },
      { status: 400 }
    )
  }

  try {
    const db = getDb()
    
    // First get the trace to find its position
    const traceQuery = `
      SELECT id FROM traces WHERE id = $1
    `
    const traceResult = await db.query(traceQuery, [traceId])
    
    if (traceResult.rows.length === 0) {
      return NextResponse.json(
        { error: 'Trace not found' },
        { status: 404 }
      )
    }

    // Get context around this trace (200 before, 200 after)
    const contextQuery = `
      SELECT id, target, level, fields
      FROM traces 
      WHERE id BETWEEN $1 AND $2
      ORDER BY id
    `
    
    const contextStart = Math.max(1, parseInt(traceId) - 200)
    const contextEnd = parseInt(traceId) + 200
    
    const result = await db.query(contextQuery, [contextStart, contextEnd])
    
    const traces = result.rows.map(row => ({
      id: row.id,
      target: row.target,
      level: row.level,
      fields: row.fields
    }))

    return NextResponse.json({
      traces,
      selectedId: parseInt(traceId)
    })

  } catch (error) {
    console.error('Database error:', error)
    return NextResponse.json(
      { error: 'Failed to fetch context traces' },
      { status: 500 }
    )
  }
}