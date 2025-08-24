'use client'

import { useState, useEffect } from 'react'
import { FieldOptions, TraceFilters } from '@/types/trace'

interface TraceFiltersProps {
  onFilter: (filters: TraceFilters) => void
}

export default function TraceFiltersComponent({ onFilter }: TraceFiltersProps) {
  const [levelFilter, setLevelFilter] = useState('')
  const [targetFilter, setTargetFilter] = useState('')
  const [searchTerm, setSearchTerm] = useState('')
  const [fieldFilters, setFieldFilters] = useState<Record<string, string>>({})

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

  const triggerFilter = (overrides?: Partial<TraceFilters>) => {
    const filters: TraceFilters = {}

    const level = overrides?.level !== undefined ? overrides.level : levelFilter
    const target = overrides?.target !== undefined ? overrides.target : targetFilter
    const search = overrides?.search !== undefined ? overrides.search : searchTerm

    if (level && level !== null) filters.level = level
    if (target && target !== null) filters.target = target
    if (search) filters.search = search

    Object.entries({ ...fieldFilters, ...overrides }).forEach(([key, value]) => {
      if (value && !['level', 'target', 'search'].includes(key)) {
        filters[key] = value
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
    setTargetFilter('')
    setSearchTerm('')
    setFieldFilters({})
    onFilter({})
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
            <input
              type="text"
              placeholder="Search all fields..."
              className="input input-bordered w-full"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              onKeyUp={(e) => {
                if (e.key === 'Enter') {
                  triggerFilter()
                }
              }}
              onBlur={() => triggerFilter()}
            />
          </div>

          <div className="form-control">
            <label className="label">
              <span className="label-text">Level</span>
            </label>
            <select
              className="select select-bordered w-full"
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
          </div>

          <div className="form-control">
            <label className="label">
              <span className="label-text">Target</span>
            </label>
            <select
              className="select select-bordered w-full"
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
                  <input
                    type="text"
                    placeholder={`Filter by ${field}...`}
                    className="input input-bordered w-full"
                    value={fieldFilters[field] || ''}
                    onChange={(e) => handleFieldFilterChange(field, e.target.value)}
                    onKeyUp={(e) => {
                      if (e.key === 'Enter') {
                        triggerFilter()
                      }
                    }}
                    onBlur={() => triggerFilter()}
                  />
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}