import { ChevronLeft, ChevronRight, RefreshCw } from 'lucide-react'

export function CursorPagination({ pageNumber, itemCount, noun, hasMore, canPrevious, loading, onPrevious, onNext, onRetry }: {
  pageNumber: number
  itemCount: number
  noun: string
  hasMore: boolean
  canPrevious: boolean
  loading: boolean
  onPrevious: () => void
  onNext: () => void
  onRetry: () => void
}) {
  return <footer className="cursor-pagination" aria-label={`${noun} pagination`} aria-busy={loading}>
    <span className="cursor-pagination__status" aria-live="polite">Page {pageNumber} · {itemCount} {noun}</span>
    <div>
      <button type="button" className="button" onClick={onRetry} disabled={loading} aria-label={`Refresh ${noun}`}><RefreshCw size={13} className={loading ? 'spin' : ''} /></button>
      <button type="button" className="button" onClick={onPrevious} disabled={!canPrevious || loading}><ChevronLeft size={14} /> Previous</button>
      <button type="button" className="button" onClick={onNext} disabled={!hasMore || loading}>Next <ChevronRight size={14} /></button>
    </div>
  </footer>
}
