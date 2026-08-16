import { useEffect, useId, useMemo, useRef, useState } from 'react'
import type { CSSProperties, KeyboardEvent, ReactNode } from 'react'
import { ArrowDown, ArrowUp } from 'lucide-react'
import type { SortDirection } from '../../features/telemetry/transforms'

export interface OperationalColumn<T> {
  key: keyof T
  label: string
  width: string
  priority?: 'secondary' | 'tertiary'
  render: (row: T) => ReactNode
}

interface OperationalTableProps<T> {
  rows: T[]
  columns: OperationalColumn<T>[]
  rowKey: (row: T) => string
  selectedKey?: string
  sortKey: keyof T
  sortDirection: SortDirection
  onSort: (key: keyof T) => void
  onSelect: (row: T) => void
  emptyTitle: string
  emptyDescription: string
  ariaLabel: string
}

const ROW_HEIGHT = 45
const OVERSCAN = 6

export function OperationalTable<T>({ rows, columns, rowKey, selectedKey, sortKey, sortDirection, onSort, onSelect, emptyTitle, emptyDescription, ariaLabel }: OperationalTableProps<T>) {
  const [scrollTop, setScrollTop] = useState(0)
  const [viewportHeight, setViewportHeight] = useState(420)
  const [focusIndex, setFocusIndex] = useState(0)
  const tableId = useId()
  const viewportRef = useRef<HTMLDivElement>(null)
  const template = columns.map(({ width }) => width).join(' ')

  useEffect(() => {
    const viewport = viewportRef.current
    if (!viewport) return
    const observer = new ResizeObserver(([entry]) => setViewportHeight(entry.contentRect.height))
    observer.observe(viewport)
    return () => observer.disconnect()
  }, [])

  useEffect(() => {
    setFocusIndex((index) => Math.min(index, Math.max(0, rows.length - 1)))
    const maxScroll = Math.max(0, rows.length * ROW_HEIGHT - viewportHeight)
    if (scrollTop > maxScroll) {
      setScrollTop(maxScroll)
      if (viewportRef.current) viewportRef.current.scrollTop = maxScroll
    }
  }, [rows.length, scrollTop, viewportHeight])

  const windowed = useMemo(() => {
    const start = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN)
    const end = Math.min(rows.length, Math.ceil((scrollTop + viewportHeight) / ROW_HEIGHT) + OVERSCAN)
    return { start, rows: rows.slice(start, end) }
  }, [rows, scrollTop, viewportHeight])

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (!rows.length) return
    let next = focusIndex
    if (event.key === 'ArrowDown') next = Math.min(rows.length - 1, focusIndex + 1)
    else if (event.key === 'ArrowUp') next = Math.max(0, focusIndex - 1)
    else if (event.key === 'Home') next = 0
    else if (event.key === 'End') next = rows.length - 1
    else if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); onSelect(rows[focusIndex]); return }
    else return
    event.preventDefault()
    setFocusIndex(next)
    viewportRef.current?.scrollTo({ top: Math.max(0, next * ROW_HEIGHT - viewportHeight / 2), behavior: 'auto' })
  }

  return (
    <div className="operational-table" role="grid" aria-label={ariaLabel} aria-rowcount={rows.length + 1} aria-activedescendant={rows.length ? `${tableId}-row-${focusIndex}` : undefined} tabIndex={0} onKeyDown={onKeyDown}>
      <div className="operational-table__header" role="row" style={{ '--table-template': template } as CSSProperties}>
        {columns.map((column) => <button key={String(column.key)} type="button" role="columnheader" className="operational-table__heading" data-priority={column.priority} aria-sort={sortKey === column.key ? (sortDirection === 'asc' ? 'ascending' : 'descending') : 'none'} onClick={() => onSort(column.key)}><span>{column.label}</span>{sortKey === column.key && (sortDirection === 'asc' ? <ArrowUp size={11} /> : <ArrowDown size={11} />)}</button>)}
      </div>
      <div ref={viewportRef} className="operational-table__viewport" onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}>
        {!rows.length ? <div className="table-empty"><strong>{emptyTitle}</strong><span>{emptyDescription}</span></div> : <div className="operational-table__space" style={{ height: rows.length * ROW_HEIGHT }}>
          {windowed.rows.map((row, offset) => {
            const index = windowed.start + offset
            const key = rowKey(row)
            return <div id={`${tableId}-row-${index}`} key={key} role="row" aria-rowindex={index + 2} aria-selected={selectedKey === key} data-focused={focusIndex === index || undefined} className="operational-table__row" style={{ '--table-template': template, transform: `translateY(${index * ROW_HEIGHT}px)` } as CSSProperties} onMouseEnter={() => setFocusIndex(index)} onDoubleClick={() => onSelect(row)} onClick={() => onSelect(row)}>{columns.map((column) => <div key={String(column.key)} role="gridcell" data-priority={column.priority} title={String(row[column.key] ?? 'Unavailable')}>{column.render(row)}</div>)}</div>
          })}
        </div>}
      </div>
    </div>
  )
}
