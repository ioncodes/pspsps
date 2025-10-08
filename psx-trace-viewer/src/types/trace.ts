export interface Trace {
  id?: number
  target: string
  level: string
  fields: Record<string, unknown>
}

export type MatchMode = 'wildcard' | 'exact'

export interface TraceFilters {
  target?: string
  targetMode?: MatchMode
  level?: string
  levelMode?: MatchMode
  search?: string
  searchMode?: MatchMode
  [key: string]: string | MatchMode | undefined
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