import { useState, useEffect, useMemo } from 'react'

const TraceFilters = ({ traces, onFilter }) => {
  const [levelFilter, setLevelFilter] = useState('')
  const [targetFilter, setTargetFilter] = useState('')
  const [fieldFilters, setFieldFilters] = useState({})
  const [searchTerm, setSearchTerm] = useState('')
  
  const availableFields = useMemo(() => {
    const fieldsSet = new Set()
    traces.forEach(trace => {
      if (trace.fields) {
        Object.keys(trace.fields).forEach(field => fieldsSet.add(field))
      }
    })
    return Array.from(fieldsSet).sort()
  }, [traces])

  const availableLevels = useMemo(() => {
    const levelsSet = new Set()
    traces.forEach(trace => {
      if (trace.level) {
        levelsSet.add(trace.level)
      }
    })
    return Array.from(levelsSet).sort()
  }, [traces])

  const availableTargets = useMemo(() => {
    const targetsSet = new Set()
    traces.forEach(trace => {
      if (trace.target) {
        targetsSet.add(trace.target)
      }
    })
    return Array.from(targetsSet).sort()
  }, [traces])

  useEffect(() => {
    const filtered = traces.filter(trace => {
      if (levelFilter && trace.level !== levelFilter) {
        return false
      }
      
      if (targetFilter && trace.target !== targetFilter) {
        return false
      }

      for (const [field, value] of Object.entries(fieldFilters)) {
        if (value && trace.fields && trace.fields[field]) {
          const traceValue = String(trace.fields[field]).toLowerCase()
          const filterValue = value.toLowerCase()
          
          if (!traceValue.includes(filterValue)) {
            return false
          }
        } else if (value && (!trace.fields || !trace.fields[field])) {
          return false
        }
      }

      if (searchTerm) {
        const searchLower = searchTerm.toLowerCase()
        const traceString = JSON.stringify(trace).toLowerCase()
        if (!traceString.includes(searchLower)) {
          return false
        }
      }

      return true
    })
    
    onFilter(filtered)
  }, [traces, levelFilter, targetFilter, fieldFilters, searchTerm, onFilter])

  const handleFieldFilterChange = (field, value) => {
    setFieldFilters(prev => ({
      ...prev,
      [field]: value
    }))
  }

  const clearAllFilters = () => {
    setLevelFilter('')
    setTargetFilter('')
    setFieldFilters({})
    setSearchTerm('')
  }

  const hasActiveFilters = levelFilter || targetFilter || Object.values(fieldFilters).some(v => v) || searchTerm

  return (
    <div className="card bg-base-200 shadow-xl">
      <div className="card-body">
        <div className="flex justify-between items-center mb-4">
          <h2 className="card-title">Filters</h2>
          {hasActiveFilters && (
            <button 
              className="btn btn-outline btn-sm"
              onClick={clearAllFilters}
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
            />
          </div>

          <div className="form-control">
            <label className="label">
              <span className="label-text">Level</span>
            </label>
            <select 
              className="select select-bordered w-full"
              value={levelFilter}
              onChange={(e) => setLevelFilter(e.target.value)}
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
              onChange={(e) => setTargetFilter(e.target.value)}
            >
              <option value="">All targets</option>
              {availableTargets.map(target => (
                <option key={target} value={target}>{target}</option>
              ))}
            </select>
          </div>
        </div>

        {availableFields.length > 0 && (
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

export default TraceFilters