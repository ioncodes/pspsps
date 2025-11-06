'use client'

import { useState, useCallback, useEffect } from 'react'
import { Trace, TraceFilters as TraceFiltersType, TraceQueryResult, TraceCountResult } from '@/types/trace'
import TraceFiltersComponent from '@/components/TraceFilters'
import TraceViewer from '@/components/TraceViewer'

export default function Home() {
  const [traces, setTraces] = useState<Trace[]>([])
  const [total, setTotal] = useState(0)
  const [isCountLoading, setIsCountLoading] = useState(false)
  const [currentPage, setCurrentPage] = useState(1)
  const [currentLimit, setCurrentLimit] = useState(200)
  const [hasMore, setHasMore] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [currentFilters, setCurrentFilters] = useState<TraceFiltersType>({})

  const fetchTraces = useCallback(async (filters: TraceFiltersType, page = 1, limit = 200, shouldUpdateCount = true) => {
    setIsLoading(true)
    try {
      const params = new URLSearchParams({
        page: page.toString(),
        limit: limit.toString()
      })

      if (filters.level) params.append('level', filters.level)
      if (filters.target) params.append('target', filters.target)
      if (filters.search) params.append('search', filters.search)

      Object.entries(filters).forEach(([key, value]) => {
        if (value && !['level', 'target', 'search'].includes(key)) {
          params.append(`field_${key}`, value)
        }
      })

      const response = await fetch(`/api/traces?${params}`)
      const data: TraceQueryResult = await response.json()

      setTraces(data.traces || [])
      setCurrentPage(data.page)
      setCurrentLimit(data.limit)
      setHasMore(data.hasMore)

      // Fetch count in background (non-blocking) only when filters change
      if (shouldUpdateCount) {
        setIsCountLoading(true)
        fetch(`/api/traces/count?${params}`)
          .then(res => res.json())
          .then((countData: TraceCountResult) => {
            setTotal(countData.total)
          })
          .catch(error => {
            console.error('Error fetching count:', error)
            setTotal(0)
          })
          .finally(() => {
            setIsCountLoading(false)
          })
      }

    } catch (error) {
      console.error('Error fetching traces:', error)
    } finally {
      setIsLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchTraces({}, 1, 200)
  }, [fetchTraces])

  const handleFilter = useCallback((filters: TraceFiltersType, page = 1, limit = currentLimit) => {
    // Only update count when it's a new filter (page = 1) or when filters actually changed
    const shouldUpdateCount = page === 1
    setCurrentFilters(filters)
    fetchTraces(filters, page, limit, shouldUpdateCount)
  }, [fetchTraces, currentLimit])

  const handlePageSizeChange = useCallback((page: number, limit: number) => {
    // Page size changes should NOT update the count - use current filters
    fetchTraces(currentFilters, page, limit, false)
  }, [fetchTraces, currentFilters])

  return (
    <div className="flex flex-col min-h-screen">
      <div className="navbar bg-base-300 shadow-sm">
        <div className="btn btn-ghost text-xl">
          PSX Trace Viewer
        </div>
      </div>

      <div className="flex-1 p-4 space-y-6">
        <TraceFiltersComponent onFilter={handleFilter} />

        {isLoading ? (
          <div className="flex justify-center items-center h-64">
            <span className="loading loading-spinner loading-lg"></span>
          </div>
        ) : (
          <TraceViewer
            traces={traces}
            total={total}
            isCountLoading={isCountLoading}
            page={currentPage}
            limit={currentLimit}
            hasMore={hasMore}
            onFilter={handleFilter}
            onPageSizeChange={handlePageSizeChange}
          />
        )}
      </div>

      <footer className="footer footer-center bg-base-300 text-base-content p-4">
        <aside>
          <p>Made with ❤️ by <a className="link" href="https://github.com/ioncodes/">Layle</a></p>
        </aside>
      </footer>
    </div>
  )
}
