'use client'

import { useState, useEffect } from 'react'
import { FieldOptions, TraceFilters, MatchMode } from '@/types/trace'

interface TraceFiltersProps {
  onFilter: (filters: TraceFilters) => void
}

export default function TraceFiltersComponent({ onFilter }: TraceFiltersProps) {
  const [levelFilter, setLevelFilter] = useState('')
  const [levelMode, setLevelMode] = useState<MatchMode>('wildcard')
  const [targetFilter, setTargetFilter] = useState('')
  const [targetMode, setTargetMode] = useState<MatchMode>('wildcard')
  const [searchTerm, setSearchTerm] = useState('')
  const [searchMode, setSearchMode] = useState<MatchMode>('wildcard')
  const [fieldFilters, setFieldFilters] = useState<Record<string, string>>({})
  const [fieldModes, setFieldModes] = useState<Record<string, MatchMode>>({})

  const [availableFields, setAvailableFields] = useState<string[]>([])
  const [availableLevels, setAvailableLevels] = useState<string[]>([])
  const [availableTargets, setAvailableTargets] = useState<string[]>([])
  const [isLoadingFields, setIsLoadingFields] = useState(false)

  useEffect(() => {
    const loadFields = async () => {
      setIsLoadingFields(true)
      try {
        const response = await fetch('/api/traces/fields')
        const data: FieldOptions = await response.json()

        setAvailableFields(data.fields || [])
        setAvailableLevels(data.levels || [])
        setAvailableTargets(data.targets || [])
      } catch (error) {
        console.error('Error loading fields:', error)
      } finally {
        setIsLoadingFields(false)
      }
    }

    loadFields()
  }, [])

  const triggerFilter = (overrides?: Partial<TraceFilters>, updatedFieldModes?: Record<string, MatchMode>) => {
    const filters: TraceFilters = {}
    const modesMap = updatedFieldModes || fieldModes

    const level = overrides?.level !== undefined ? overrides.level : levelFilter
    const target = overrides?.target !== undefined ? overrides.target : targetFilter
    const search = overrides?.search !== undefined ? overrides.search : searchTerm

    if (level && level !== null) {
      filters.level = level
      filters.levelMode = overrides?.levelMode || levelMode
    }
    if (target && target !== null) {
      filters.target = target
      filters.targetMode = overrides?.targetMode || targetMode
    }
    if (search) {
      filters.search = search
      filters.searchMode = overrides?.searchMode || searchMode
    }

    Object.entries({ ...fieldFilters, ...overrides }).forEach(([key, value]) => {
      if (value && !['level', 'target', 'search', 'levelMode', 'targetMode', 'searchMode'].includes(key) && !key.endsWith('Mode')) {
        filters[key] = value
        filters[`${key}Mode`] = modesMap[key] || 'wildcard'
      }
    })

    onFilter(filters)
  }

  const hasActiveFilters = levelFilter || targetFilter || searchTerm || Object.values(fieldFilters).some(v => v)

  const handleFieldFilterChange = (field: string, value: string) => {
    const newFieldFilters = {
      ...fieldFilters,
      [field]: value
    }
    setFieldFilters(newFieldFilters)
    triggerFilter({ [field]: value || undefined })
  }

  const handleClearFilters = () => {
    setLevelFilter('')
    setLevelMode('wildcard')
    setTargetFilter('')
    setTargetMode('wildcard')
    setSearchTerm('')
    setSearchMode('wildcard')
    setFieldFilters({})
    setFieldModes({})
    onFilter({})
  }

  const handleFieldModeChange = (field: string, mode: MatchMode) => {
    const newFieldModes = {
      ...fieldModes,
      [field]: mode
    }
    setFieldModes(newFieldModes)
    // Pass the updated modes map so the filter includes the new mode immediately
    triggerFilter({}, newFieldModes)
  }

  return (
    <div className="card bg-base-200 shadow-xl">
      <div className="card-body">
        <div className="flex justify-between items-center mb-4">
          <h2 className="card-title">Filters</h2>
          {hasActiveFilters && (
            <button
              className="btn btn-outline btn-sm"
              onClick={handleClearFilters}
            >
              Clear All
            </button>
          )}
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <div className="form-control">
            <label className="label">
              <span className="label-text">Search</span>
            </label>
            <div className="join w-full">
              <input
                type="text"
                placeholder="Search all fields..."
                className="input input-bordered join-item flex-[3]"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                onKeyUp={(e) => {
                  if (e.key === 'Enter') {
                    triggerFilter()
                  }
                }}
                onBlur={() => triggerFilter()}
              />
              <select
                className="select select-bordered join-item w-28"
                value={searchMode}
                onChange={(e) => {
                  const mode = e.target.value as MatchMode
                  setSearchMode(mode)
                  triggerFilter({ search: searchTerm, searchMode: mode })
                }}
              >
                <option value="wildcard">Wildcard</option>
                <option value="exact">Exact</option>
              </select>
            </div>
          </div>

          <div className="form-control">
            <label className="label">
              <span className="label-text">Level</span>
            </label>
            <div className="join w-full">
              <select
                className="select select-bordered join-item flex-[3]"
                value={levelFilter}
                onChange={(e) => {
                  const newLevel = e.target.value
                  setLevelFilter(newLevel)
                  triggerFilter({ level: newLevel || undefined })
                }}
              >
                <option value="">All levels</option>
                {availableLevels.map(level => (
                  <option key={level} value={level}>{level}</option>
                ))}
              </select>
              <select
                className="select select-bordered join-item w-28"
                value={levelMode}
                onChange={(e) => {
                  const mode = e.target.value as MatchMode
                  setLevelMode(mode)
                  triggerFilter({ level: levelFilter, levelMode: mode })
                }}
              >
                <option value="wildcard">Wildcard</option>
                <option value="exact">Exact</option>
              </select>
            </div>
          </div>

          <div className="form-control">
            <label className="label">
              <span className="label-text">Target</span>
            </label>
            <div className="join w-full">
              <select
                className="select select-bordered join-item flex-[3]"
                value={targetFilter}
                onChange={(e) => {
                  const newTarget = e.target.value
                  setTargetFilter(newTarget)
                  triggerFilter({ target: newTarget || undefined })
                }}
              >
                <option value="">All targets</option>
                {availableTargets.map(target => (
                  <option key={target} value={target}>{target}</option>
                ))}
              </select>
              <select
                className="select select-bordered join-item w-28"
                value={targetMode}
                onChange={(e) => {
                  const mode = e.target.value as MatchMode
                  setTargetMode(mode)
                  triggerFilter({ target: targetFilter, targetMode: mode })
                }}
              >
                <option value="wildcard">Wildcard</option>
                <option value="exact">Exact</option>
              </select>
            </div>
          </div>
        </div>

        {isLoadingFields ? (
          <div className="mt-6 text-center">
            <span className="loading loading-spinner loading-md"></span>
            <p className="text-sm text-base-content/70 mt-2">Loading field options...</p>
          </div>
        ) : availableFields.length > 0 && (
          <div className="mt-6">
            <h3 className="text-lg font-semibold mb-3">Field Filters</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {availableFields.map(field => (
                <div key={field} className="form-control">
                  <label className="label">
                    <span className="label-text font-mono text-sm">{field}</span>
                  </label>
                  <div className="join w-full">
                    <input
                      type="text"
                      placeholder={`Filter by ${field}...`}
                      className="input input-bordered join-item flex-[3]"
                      value={fieldFilters[field] || ''}
                      onChange={(e) => handleFieldFilterChange(field, e.target.value)}
                      onKeyUp={(e) => {
                        if (e.key === 'Enter') {
                          triggerFilter()
                        }
                      }}
                      onBlur={() => triggerFilter()}
                    />
                    <select
                      className="select select-bordered join-item w-28"
                      value={fieldModes[field] || 'wildcard'}
                      onChange={(e) => {
                        const mode = e.target.value as MatchMode
                        handleFieldModeChange(field, mode)
                      }}
                    >
                      <option value="wildcard">Wildcard</option>
                      <option value="exact">Exact</option>
                    </select>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}