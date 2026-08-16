import { ChevronLeft, ChevronRight, RefreshCw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { formatNumber } from '../../i18n'

export type SecurityPaginationNoun = 'detections' | 'events' | 'evidence' | 'history'

export function CursorPagination({ pageNumber, itemCount, nounKey, hasMore, canPrevious, loading, onPrevious, onNext, onRetry }: {
  pageNumber: number
  itemCount: number
  nounKey: SecurityPaginationNoun
  hasMore: boolean
  canPrevious: boolean
  loading: boolean
  onPrevious: () => void
  onNext: () => void
  onRetry: () => void
}) {
  const { t } = useTranslation('security')
  const noun = t(`pagination.nouns.${nounKey}`, { count: itemCount })
  return <footer className="cursor-pagination" aria-label={t('pagination.ariaLabel', { noun })} aria-busy={loading}>
    <span className="cursor-pagination__status" aria-live="polite">{t('pagination.status', { page: formatNumber(pageNumber), count: formatNumber(itemCount), noun })}</span>
    <div>
      <button type="button" className="button" onClick={onRetry} disabled={loading} aria-label={t('pagination.refresh', { noun })}><RefreshCw size={13} className={loading ? 'spin' : ''} /></button>
      <button type="button" className="button" onClick={onPrevious} disabled={!canPrevious || loading}><ChevronLeft size={14} /> {t('pagination.previous')}</button>
      <button type="button" className="button" onClick={onNext} disabled={!hasMore || loading}>{t('pagination.next')} <ChevronRight size={14} /></button>
    </div>
  </footer>
}
