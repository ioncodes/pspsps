import { useState, useRef } from 'react'

const TraceLoader = ({ onTraceLoad }) => {
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState(null)
  const [progress, setProgress] = useState(0)
  const [currentStep, setCurrentStep] = useState('')
  const abortControllerRef = useRef(null)

  const processFileChunked = async (file) => {
    const totalSize = file.size
    const chunkSize = 10 * 1024 * 1024
    let bytesRead = 0
    let buffer = ''
    let lineIndex = 0
    const traces = []

    const reader = file.stream().getReader()
    const decoder = new TextDecoder()

    try {
      while (true) {
        const { done, value } = await reader.read()

        if (done) {
          // Process any remaining content in buffer
          if (buffer.trim()) {
            try {
              const trace = JSON.parse(buffer.trim())
              traces.push({ ...trace, _index: lineIndex })
            } catch {
              console.warn(`Failed to parse final line ${lineIndex + 1}`)
            }
          }
          break
        }

        if (abortControllerRef.current?.signal.aborted) {
          throw new Error('Loading cancelled')
        }

        bytesRead += value.length
        buffer += decoder.decode(value, { stream: true })

        // Only process when we have ~10MB
        if (buffer.length > chunkSize || done) {
          const lines = buffer.split('\n')
          buffer = lines.pop() || '' // Keep incomplete line in buffer

          // Process all complete lines at once
          const batchTraces = []
          for (const line of lines) {
            if (line.trim()) {
              try {
                const trace = JSON.parse(line.trim())
                batchTraces.push({ ...trace, _index: lineIndex })
              } catch {
                console.warn(`Failed to parse line ${lineIndex + 1}`)
              }
              lineIndex++
            }
          }

          traces.push(...batchTraces)

          // Update progress
          const progressPercent = Math.round((bytesRead / totalSize) * 100)
          setProgress(progressPercent)
          setCurrentStep(`Processing... ${Math.round(bytesRead / 1024 / 1024)}MB / ${Math.round(totalSize / 1024 / 1024)}MB (${traces.length} entries)`)

          // Update UI
          await new Promise(resolve => setTimeout(resolve, 10))
        }
      }
    } finally {
      reader.releaseLock()
    }

    return traces
  }

  const handleFileLoad = async (event) => {
    const file = event.target.files[0]
    if (!file) return

    setIsLoading(true)
    setError(null)
    setProgress(0)
    setCurrentStep('Reading file...')

    // Create abort controller for cancellation
    abortControllerRef.current = new AbortController()

    try {
      const traces = await processFileChunked(file)

      setCurrentStep(`Complete! Loaded ${traces.length} entries`)
      setProgress(100)
      onTraceLoad(traces)
    } catch (e) {
      if (e.message !== 'Loading cancelled') {
        setError(e.message)
      }
    } finally {
      setIsLoading(false)
      abortControllerRef.current = null
    }
  }

  const handleCancel = () => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort()
      setIsLoading(false)
      setProgress(0)
      setCurrentStep('')
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
          <div className="mt-4 space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">{currentStep}</span>
              <button
                className="btn btn-outline btn-xs"
                onClick={handleCancel}
              >
                Cancel
              </button>
            </div>
            <div className="w-full bg-base-300 rounded-full h-2">
              <div
                className="bg-primary h-2 rounded-full transition-all duration-300 ease-out"
                style={{ width: `${progress}%` }}
              ></div>
            </div>
            <div className="text-xs text-base-content/70 text-center">
              {progress}% complete
            </div>
          </div>
        )}

        {error && (
          <div className="alert alert-error mt-4">
            <svg xmlns="http://www.w3.org/2000/svg" className="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />            ¨
            </svg>
            <span>{error}</span>
          </div>
        )}
      </div>
    </div>
  )
}

export default TraceLoader