import type { ButtonHTMLAttributes, HTMLAttributes, ReactNode } from 'react'
import { X } from 'lucide-react'

export function Badge({ children, tone = 'neutral' }: { children: ReactNode; tone?: 'neutral' | 'good' | 'warning' | 'danger' }) {
  return <span className={`badge badge--${tone}`}>{children}</span>
}

export function StatusDot({ status = 'online' }: { status?: 'online' | 'partial' | 'offline' }) {
  return <span className={`status-dot status-dot--${status}`} aria-label={status} />
}

export function IconButton({ className = '', ...props }: ButtonHTMLAttributes<HTMLButtonElement>) {
  return <button type="button" className={`icon-button ${className}`} {...props} />
}

export function Skeleton({ className = '' }: { className?: string }) {
  return <span className={`skeleton ${className}`} aria-hidden="true" />
}

export function EmptyState({ title, description }: { title: string; description: string }) {
  return (
    <div className="empty-state">
      <div className="empty-state__mark" aria-hidden="true" />
      <strong>{title}</strong>
      <span>{description}</span>
    </div>
  )
}

export function Dialog({ open, title, children, onClose }: { open: boolean; title: string; children: ReactNode; onClose: () => void }) {
  if (!open) return null
  return (
    <div className="dialog-backdrop" role="presentation" onMouseDown={(event) => event.target === event.currentTarget && onClose()}>
      <section className="dialog" role="dialog" aria-modal="true" aria-labelledby="dialog-title">
        <header>
          <h2 id="dialog-title">{title}</h2>
          <IconButton aria-label="Close dialog" onClick={onClose}><X size={18} /></IconButton>
        </header>
        {children}
      </section>
    </div>
  )
}

export function Drawer({ open, title, children, onClose }: { open: boolean; title: string; children: ReactNode; onClose: () => void }) {
  if (!open) return null
  return (
    <aside className="drawer" aria-label={title}>
      <header><h2>{title}</h2><IconButton aria-label="Close drawer" onClick={onClose}><X size={18} /></IconButton></header>
      {children}
    </aside>
  )
}

export function Tooltip({ label, children }: { label: string; children: ReactNode }) {
  return <span className="tooltip" data-tooltip={label}>{children}</span>
}

export function Sparkline({ points, ...props }: { points: number[] } & HTMLAttributes<SVGElement>) {
  if (points.length < 2) return null
  const max = Math.max(...points)
  const min = Math.min(...points)
  const range = Math.max(max - min, 1)
  const path = points.map((point, index) => `${(index / (points.length - 1)) * 100},${32 - ((point - min) / range) * 28}`).join(' ')
  return <svg viewBox="0 0 100 36" role="img" aria-label="Metric trend" {...props}><polyline points={path} fill="none" stroke="currentColor" strokeWidth="2" vectorEffect="non-scaling-stroke" /></svg>
}
