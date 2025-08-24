import { useEffect, useRef } from 'react'
import TraceEntry from './TraceEntry'

const ContextModal = ({ isOpen, onClose, contextTraces, selectedTrace }) => {
  const selectedRef = useRef(null)

  const selectedIndex = contextTraces?.findIndex(trace => trace._index === selectedTrace?._index) ?? -1

  useEffect(() => {
    if (isOpen && selectedRef.current) {
      selectedRef.current.scrollIntoView({
        behavior: 'smooth',
        block: 'center'
      })
    }
  }, [isOpen])

  if (!isOpen) return null

  return (
    <div className="modal modal-open">
      <div className="modal-box w-11/12 max-w-5xl max-h-[90vh] overflow-hidden flex flex-col">
        <div className="flex justify-between items-center mb-4">
          <h3 className="font-bold text-lg">Context View</h3>
          <button
            className="btn btn-sm btn-circle btn-ghost"
            onClick={onClose}
          >
            ✕
          </button>
        </div>

        <div className="text-sm mb-4">
          Showing {contextTraces.length} entries around selected trace
        </div>

        <div className="flex-1 overflow-y-auto space-y-2">
          {contextTraces.map((trace, index) => (
            <div
              key={trace._index}
              className="relative"
              ref={selectedIndex === index ? selectedRef : null}
            >
              <TraceEntry
                trace={trace}
                index={trace._index}
              />
              {selectedIndex === index && (
                <div className="absolute -top-2 right-2">
                  <div className="badge badge-primary badge-sm">Selected</div>
                </div>
              )}
            </div>
          ))}
        </div>

        <div className="modal-action">
          <button className="btn" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
      <div className="modal-backdrop" onClick={onClose}></div>
    </div>
  )
}

export default ContextModal