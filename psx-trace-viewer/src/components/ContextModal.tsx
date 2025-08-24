'use client'

import { useEffect, useRef, useState } from 'react'
import { Trace } from '@/types/trace'
import TraceEntry from './TraceEntry'

interface ContextModalProps {
  isOpen: boolean
  onClose: () => void
  selectedTrace: Trace
}

export default function ContextModal({ 
  isOpen, 
  onClose, 
  selectedTrace 
}: ContextModalProps) {
  const selectedEntryRef = useRef<HTMLDivElement>(null)
  const [contextTraces, setContextTraces] = useState<Trace[]>([])
  const [loading, setLoading] = useState(false)
  const [selectedIndex, setSelectedIndex] = useState(0)

  useEffect(() => {
    if (isOpen && selectedTrace?.id) {
      setLoading(true)
      fetch(`/api/traces/context?id=${selectedTrace.id}`)
        .then(res => res.json())
        .then(data => {
          setContextTraces(data.traces)
          const index = data.traces.findIndex((t: Trace) => t.id === selectedTrace.id)
          setSelectedIndex(index)
        })
        .catch(console.error)
        .finally(() => setLoading(false))
    }
  }, [isOpen, selectedTrace?.id])

  useEffect(() => {
    if (isOpen && selectedEntryRef.current && !loading) {
      setTimeout(() => {
        selectedEntryRef.current?.scrollIntoView({
          behavior: 'smooth',
          block: 'center'
        })
      }, 100)
    }
  }, [isOpen, loading])

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-base-100 rounded-lg shadow-xl w-full max-w-6xl max-h-[90vh] flex flex-col">
        <div className="flex justify-between items-center p-4 border-b border-base-300">
          <h3 className="text-lg font-semibold">
            Context View - Entry #{selectedTrace.id}
          </h3>
          <button
            onClick={onClose}
            className="btn btn-sm btn-circle btn-ghost"
          >
            ✕
          </button>
        </div>
        
        <div className="flex-1 overflow-y-auto p-4">
          {loading ? (
            <div className="text-center">
              <span className="loading loading-spinner loading-md"></span>
              <p className="text-sm text-base-content/70 mt-2">Loading context...</p>
            </div>
          ) : (
            <div className="space-y-2">
              {contextTraces.map((trace, index) => {
                const isSelected = index === selectedIndex
                
                return (
                  <div
                    key={trace.id}
                    ref={isSelected ? selectedEntryRef : null}
                  >
                    <TraceEntry
                      trace={trace}
                      index={index}
                      isSelected={isSelected}
                    />
                  </div>
                )
              })}
            </div>
          )}
        </div>
        
        <div className="p-4 border-t border-base-300 text-sm text-base-content/70 text-center">
          Showing {contextTraces.length} entries (unfiltered context)
        </div>
      </div>
    </div>
  )
}