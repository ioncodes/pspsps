export interface Trace {
  id?: number
  target: string
  level: string
  fields: Record<string, unknown>
}

export interface TraceFilters {
  target?: string
  level?: string
  search?: string
  [key: string]: string | undefined
}

export interface TraceQueryResult {
  traces: Trace[]
  total?: number
  page: number
  limit: number
  hasMore: boolean
}

export interface TraceCountResult {
  total: number
}

export interface FieldOptions {
  fields: string[]
  levels: string[]
  targets: string[]
}