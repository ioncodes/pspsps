import { useState } from 'react'

const TraceLoader = ({ onTraceLoad }) => {
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState(null)

  const parseTraceFile = (content) => {
    try {
      const lines = content.split('\n').filter(line => line.trim())
      const traces = lines.map((line, index) => {
        try {
          const trace = JSON.parse(line.trim())
          return { ...trace, _index: index }
        } catch {
          console.warn(`Failed to parse line ${index + 1}:`, line)
          return null
        }
      }).filter(Boolean)

      return traces
    } catch (e) {
      throw new Error(`Failed to parse trace file: ${e.message}`)
    }
  }

  const handleFileLoad = async (event) => {
    const file = event.target.files[0]
    if (!file) return

    setIsLoading(true)
    setError(null)

    try {
      const content = await file.text()
      const traces = parseTraceFile(content)
      onTraceLoad(traces)
    } catch (e) {
      setError(e.message)
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="card bg-base-200 shadow-xl mb-6">
      <div className="card-body">
        <h2 className="card-title">Load Trace File</h2>
        <p className="text-base-content/70">
          Select any .json file generated using tracing-subscriber to view the logs.
        </p>

        <div className="form-control w-full max-w-xs">
          <label className="label">
            <span className="label-text">Choose trace file</span>
          </label>
          <input
            type="file"
            accept=".json"
            onChange={handleFileLoad}
            className="file-input file-input-bordered w-full max-w-xs"
          />
        </div>

        {isLoading && (
          <div className="flex items-center gap-2 mt-4">
            <span className="loading loading-spinner loading-sm"></span>
            <span>Loading trace file...</span>
          </div>
        )}

        {error && (
          <div className="alert alert-error mt-4">
            <svg xmlns="http://www.w3.org/2000/svg" className="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span>{error}</span>
          </div>
        )}
      </div>
    </div>
  )
}

export default TraceLoader