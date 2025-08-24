import { useState, useEffect } from 'react'
import TraceFilters from './TraceFilters'
import TraceEntry from './TraceEntry'
import ContextModal from './ContextModal'

const TraceViewer = ({ traces, filteredTraces, onFilter }) => {
  const [currentPage, setCurrentPage] = useState(1)
  const [pageSize, setPageSize] = useState(50)
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [selectedTrace, setSelectedTrace] = useState(null)
  const [contextTraces, setContextTraces] = useState([])

  const totalPages = Math.ceil(filteredTraces.length / pageSize)
  const startIndex = (currentPage - 1) * pageSize
  const endIndex = Math.min(startIndex + pageSize, filteredTraces.length)
  const currentTraces = filteredTraces.slice(startIndex, endIndex)

  useEffect(() => {
    setCurrentPage(1)
  }, [filteredTraces])

  const handlePageChange = (newPage) => {
    if (newPage >= 1 && newPage <= totalPages) {
      setCurrentPage(newPage)
    }
  }

  const handleTraceClick = (clickedTrace) => {
    const originalIndex = clickedTrace._index
    const contextSize = 200

    // Get surrounding entries from the original traces array (ignoring filters)
    const startIdx = Math.max(0, originalIndex - contextSize)
    const endIdx = Math.min(traces.length - 1, originalIndex + contextSize)

    const context = traces.slice(startIdx, endIdx + 1)

    setSelectedTrace(clickedTrace)
    setContextTraces(context)
    setIsModalOpen(true)
  }

  const handleCloseModal = () => {
    setIsModalOpen(false)
    setSelectedTrace(null)
    setContextTraces([])
  }

  return (
    <div className="space-y-6">
      <TraceFilters traces={traces} onFilter={onFilter} />

      <div className="card bg-base-200 shadow-xl">
        <div className="card-body">
          <div className="flex justify-between items-center mb-4">
            <h2 className="card-title">
              Trace Entries
              <div className="badge badge-primary">{filteredTraces.length}</div>
            </h2>

            <div className="flex items-center gap-2">
              <span className="text-sm">Show:</span>
              <select
                className="select select-bordered select-sm"
                value={pageSize}
                onChange={(e) => setPageSize(Number(e.target.value))}
              >
                <option value={25}>25</option>
                <option value={50}>50</option>
                <option value={100}>100</option>
                <option value={200}>200</option>
              </select>
            </div>
          </div>

          {filteredTraces.length === 0 ? (
            <div className="text-center py-8">
              <div className="text-base-content/50">No traces match the current filters</div>
            </div>
          ) : (
            <>
              <div className="overflow-x-auto">
                <div className="space-y-2">
                  {currentTraces.map((trace, index) => (
                    <TraceEntry
                      key={trace._index}
                      trace={trace}
                      index={startIndex + index}
                      onClick={handleTraceClick}
                    />
                  ))}
                </div>
              </div>

              {totalPages > 1 && (
                <div className="flex justify-center mt-6">
                  <div className="join">
                    <button
                      className="join-item btn"
                      disabled={currentPage === 1}
                      onClick={() => handlePageChange(currentPage - 1)}
                    >
                      «
                    </button>

                    {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
                      let pageNum
                      if (totalPages <= 5) {
                        pageNum = i + 1
                      } else if (currentPage <= 3) {
                        pageNum = i + 1
                      } else if (currentPage >= totalPages - 2) {
                        pageNum = totalPages - 4 + i
                      } else {
                        pageNum = currentPage - 2 + i
                      }

                      return (
                        <button
                          key={pageNum}
                          className={`join-item btn ${currentPage === pageNum ? 'btn-active' : ''}`}
                          onClick={() => handlePageChange(pageNum)}
                        >
                          {pageNum}
                        </button>
                      )
                    })}

                    <button
                      className="join-item btn"
                      disabled={currentPage === totalPages}
                      onClick={() => handlePageChange(currentPage + 1)}
                    >
                      »
                    </button>
                  </div>
                </div>
              )}

              <div className="text-center text-sm text-base-content/70 mt-4">
                Showing {startIndex + 1}-{endIndex} of {filteredTraces.length} entries
              </div>
            </>
          )}
        </div>
      </div>

      <ContextModal
        isOpen={isModalOpen}
        onClose={handleCloseModal}
        contextTraces={contextTraces}
        selectedTrace={selectedTrace}
      />
    </div>
  )
}

export default TraceViewer