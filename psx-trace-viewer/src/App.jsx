import { useState, useCallback } from 'react'
import TraceLoader from './components/TraceLoader'
import TraceViewer from './components/TraceViewer'

function App() {
  const [traces, setTraces] = useState([])
  const [filteredTraces, setFilteredTraces] = useState([])

  const handleTraceLoad = useCallback((loadedTraces) => {
    setTraces(loadedTraces)
    setFilteredTraces(loadedTraces)
  }, [])

  const handleFilter = useCallback((filtered) => {
    setFilteredTraces(filtered)
  }, [])

  return (
    <div data-theme="dark" className="flex flex-col min-h-screen">
      <div class="navbar bg-base-300 shadow-sm">
        <a href="/" class="btn btn-ghost text-xl hover:bg-primary hover:text-primary-content">PSX Trace Viewer</a>
      </div>

      <div className="flex-1 flex items-center justify-center p-4">
        {traces.length == 0 && (
          <TraceLoader onTraceLoad={handleTraceLoad} />
        )}

        {traces.length > 0 && (
          <TraceViewer
            traces={traces}
            filteredTraces={filteredTraces}
            onFilter={handleFilter}
          />
        )}
      </div>

      <footer class="footer sm:footer-horizontal footer-center bg-base-300 text-base-content p-4">
        <aside>
          <p>Made with ❤️ by <a className="link" href="https://github.com/ioncodes/">Layle</a></p>
        </aside>
      </footer>
    </div>
  )
}

export default App
