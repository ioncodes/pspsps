const getLevelColor = (level) => {
  switch (level?.toUpperCase()) {
    case 'ERROR':
      return 'badge-error'
    case 'WARN':
      return 'badge-warning'
    case 'INFO':
      return 'badge-info'
    case 'DEBUG':
      return 'badge-secondary'
    case 'TRACE':
      return 'badge-primary'
    default:
      return 'badge-neutral'
  }
}

const TraceEntry = ({ trace, index, onClick }) => {
  const renderFieldValue = (value) => {
    if (typeof value === 'object' && value !== null) {
      return JSON.stringify(value, null, 2)
    }
    return String(value)
  }

  return (
    <div
      className={`card bg-base-100 border border-base-300 shadow-sm ${onClick ? 'cursor-pointer hover:bg-base-200 transition-colors' : ''}`}
      onClick={onClick ? () => onClick(trace) : undefined}
    >
      <div className="card-body p-4">
        <div className="flex items-start justify-between gap-4">
          <div className="flex-1 space-y-2">
            <div className="flex items-center gap-2 flex-wrap">
              <span className="text-xs text-base-content/50">#{index + 1}</span>
              {trace.level && (
                <div className={`badge badge-soft badge-sm ${getLevelColor(trace.level)}`}>
                  {trace.level}
                </div>
              )}
              {trace.target && (
                <div className="badge badge-outline badge-sm">
                  {trace.target}
                </div>
              )}
            </div>

            {trace.fields && Object.keys(trace.fields).length > 0 && (
              <div className="space-y-1">
                {Object.entries(trace.fields).map(([key, value]) => (
                  <div key={key} className="flex gap-2">
                    <span className="font-mono text-sm font-medium text-primary min-w-0">
                      {key}:
                    </span>
                    <span className="font-mono text-sm break-all">
                      {renderFieldValue(value)}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

export default TraceEntry