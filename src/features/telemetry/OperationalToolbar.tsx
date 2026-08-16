import { Search } from 'lucide-react'
import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'

export interface FilterOption<T extends string> { value: T; label: string }

export function OperationalToolbar<T extends string>({ query, onQueryChange, filter, onFilterChange, options, placeholder, meta }: { query: string; onQueryChange: (value: string) => void; filter: T; onFilterChange: (value: T) => void; options: FilterOption<T>[]; placeholder: string; meta?: ReactNode }) {
  const { t } = useTranslation('telemetry')
  return (
    <div className="operational-toolbar">
      <label className="table-search"><Search size={15} /><input value={query} onChange={(event) => onQueryChange(event.target.value)} placeholder={placeholder} aria-label={placeholder} /></label>
      <div className="filter-chips" aria-label={t('toolbar.filtersAria')}>{options.map((option) => <button type="button" key={option.value} data-active={filter === option.value || undefined} aria-pressed={filter === option.value} onClick={() => onFilterChange(option.value)}>{option.label}</button>)}</div>
      {meta && <div className="operational-toolbar__meta">{meta}</div>}
    </div>
  )
}
