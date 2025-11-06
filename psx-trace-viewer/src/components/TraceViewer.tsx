'use client'

import { useState } from 'react'
import { Trace, TraceFilters } from '@/types/trace'
import TraceEntry from './TraceEntry'
import ContextModal from './ContextModal'

interface TraceViewerProps {
  traces: Trace[]
  total: number
  isCountLoading: boolean
  page: number
  limit: number
  hasMore: boolean
  onFilter: (filters: TraceFilters, page?: number, limit?: number) => void
  onPageSizeChange?: (page: number, limit: number) => void
}

export default function TraceViewer({ traces, total, isCountLoading, page, limit, onFilter, onPageSizeChange }: TraceViewerProps) {
  const [pageSize, setPageSize] = useState(limit)
  const [isLoadingMore, setIsLoadingMore] = useState(false)
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [selectedTrace, setSelectedTrace] = useState<Trace | null>(null)
  const [jumpToPage, setJumpToPage] = useState('')

  const totalPages = Math.ceil(total / pageSize)
  const startIndex = (page - 1) * pageSize
  const endIndex = Math.min(startIndex + pageSize, total)

  const handlePageChange = async (newPage: number) => {
    if (newPage >= 1 && newPage <= totalPages) {
      setIsLoadingMore(true)
      await onFilter({}, newPage, pageSize)
      setIsLoadingMore(false)
    }
  }

  const handlePageSizeChange = (newSize: number) => {
    setPageSize(newSize)
    if (onPageSizeChange) {
      onPageSizeChange(1, newSize)
    } else {
      onFilter({}, 1, newSize)
    }
  }

  const handleTraceClick = (trace: Trace) => {
    setSelectedTrace(trace)
    setIsModalOpen(true)
  }

  const handleJumpToPage = () => {
    const pageNum = parseInt(jumpToPage)
    if (!isNaN(pageNum) && pageNum >= 1 && pageNum <= totalPages) {
      handlePageChange(pageNum)
      setJumpToPage('')
    }
  }

  return (
    <div className="card bg-base-200 shadow-xl">
      <div className="card-body">
        <div className="flex justify-between items-center mb-4">
          <h2 className="card-title">
            Trace Entries
            <div className="badge badge-primary">
              {isCountLoading ? (
                <span className="flex items-center gap-1">
                  <span className="loading loading-spinner loading-xs"></span>
                  loading...
                </span>
              ) : (
                `${total.toLocaleString()} total`
              )}
            </div>
          </h2>

          <div className="flex items-center gap-2">
            <span className="text-sm">Show:</span>
            <select
              className="select select-bordered select-sm"
              value={pageSize}
              onChange={(e) => handlePageSizeChange(Number(e.target.value))}
            >
              <option value={25}>25</option>
              <option value={50}>50</option>
              <option value={100}>100</option>
              <option value={200}>200</option>
              <option value={500}>500</option>
            </select>
          </div>
        </div>

        {traces.length === 0 ? (
          <div className="text-center py-8">
            <div className="text-base-content/50">No traces match the current filters</div>
          </div>
        ) : (
          <>
            <div className="overflow-x-auto">
              <div className="space-y-2">
                {traces.map((trace, index) => (
                  <TraceEntry
                    key={trace.id || startIndex + index}
                    trace={trace}
                    index={startIndex + index}
                    onClick={() => handleTraceClick(trace)}
                  />
                ))}
              </div>
            </div>

            {(totalPages > 1 || isLoadingMore) && (
              <div className="flex justify-center items-center gap-2 mt-6">
                {isLoadingMore ? (
                  <span className="loading loading-spinner loading-md"></span>
                ) : (
                  <>
                    <button
                      className="btn btn-sm"
                      disabled={page === 1}
                      onClick={() => handlePageChange(1)}
                      title="First page"
                    >
                      ««
                    </button>
                    <div className="join">
                      <button
                        className="join-item btn"
                        disabled={page === 1}
                        onClick={() => handlePageChange(page - 1)}
                      >
                        «
                      </button>

                      {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
                        let pageNum
                        if (totalPages <= 5) {
                          pageNum = i + 1
                        } else if (page <= 3) {
                          pageNum = i + 1
                        } else if (page >= totalPages - 2) {
                          pageNum = totalPages - 4 + i
                        } else {
                          pageNum = page - 2 + i
                        }

                        return (
                          <button
                            key={pageNum}
                            className={`join-item btn ${page === pageNum ? 'btn-active' : ''}`}
                            onClick={() => handlePageChange(pageNum)}
                          >
                            {pageNum}
                          </button>
                        )
                      })}

                      <button
                        className="join-item btn"
                        disabled={page === totalPages}
                        onClick={() => handlePageChange(page + 1)}
                      >
                        »
                      </button>
                    </div>
                    <button
                      className="btn btn-sm"
                      disabled={page === totalPages}
                      onClick={() => handlePageChange(totalPages)}
                      title="Last page"
                    >
                      »»
                    </button>
                  </>
                )}
              </div>
            )}

            <div className="text-center text-sm text-base-content/70 mt-4">
              Showing {startIndex + 1}-{endIndex} of {total} entries
            </div>

            {totalPages > 1 && (
              <div className="flex justify-center items-center gap-2 mt-4">
                <span className="text-sm">Jump to page:</span>
                <input
                  type="number"
                  min="1"
                  max={totalPages}
                  placeholder="Page #"
                  className="input input-bordered input-sm w-24"
                  value={jumpToPage}
                  onChange={(e) => setJumpToPage(e.target.value)}
                  onKeyUp={(e) => {
                    if (e.key === 'Enter') {
                      handleJumpToPage()
                    }
                  }}
                />
                <button
                  className="btn btn-sm btn-primary"
                  onClick={handleJumpToPage}
                  disabled={!jumpToPage || isLoadingMore}
                >
                  Go
                </button>
              </div>
            )}
          </>
        )}
      </div>

      {selectedTrace && (
        <ContextModal
          isOpen={isModalOpen}
          onClose={() => setIsModalOpen(false)}
          selectedTrace={selectedTrace}
        />
      )}
    </div>
  )
}