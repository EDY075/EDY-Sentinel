import { useEffect, useRef } from 'react'
import { Check, Palette } from 'lucide-react'
import type { ThemeName } from '../../types/system'

const themes: Array<{ id: ThemeName; name: string; description: string }> = [
  { id: 'sentinel-blue', name: 'Sentinel Blue', description: 'Focused and precise' },
  { id: 'cyber-green', name: 'Cyber Green', description: 'Operational and calm' },
  { id: 'terminal', name: 'Terminal', description: 'Minimal and technical' },
  { id: 'spectrum', name: 'Spectrum', description: 'Vivid, controlled depth' },
]

export function ThemeMenu({ theme, onChange, onClose }: { theme: ThemeName; onChange: (theme: ThemeName) => void; onClose: () => void }) {
  const menuRef = useRef<HTMLDivElement>(null)
  const onKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    const items = Array.from(menuRef.current?.querySelectorAll<HTMLButtonElement>('[role="menuitemradio"]') ?? [])
    const current = items.indexOf(document.activeElement as HTMLButtonElement)
    let next = current
    if (event.key === 'ArrowDown') next = (current + 1) % items.length
    else if (event.key === 'ArrowUp') next = (current - 1 + items.length) % items.length
    else if (event.key === 'Home') next = 0
    else if (event.key === 'End') next = items.length - 1
    else if (event.key === 'Escape') { event.preventDefault(); onClose(); return }
    else return
    event.preventDefault()
    items[next]?.focus()
  }
  useEffect(() => {
    const menu = menuRef.current
    menu?.querySelector<HTMLButtonElement>('[aria-checked="true"]')?.focus()
    const onPointerDown = (event: PointerEvent) => {
      if (menu && !menu.parentElement?.contains(event.target as Node)) onClose()
    }
    document.addEventListener('pointerdown', onPointerDown)
    return () => document.removeEventListener('pointerdown', onPointerDown)
  }, [onClose])
  return (
    <div ref={menuRef} className="theme-menu" role="menu" aria-label="Theme selector" onKeyDown={onKeyDown}>
      <div className="theme-menu__title"><Palette size={15} /> Interface theme</div>
      {themes.map((item) => (
        <button key={item.id} type="button" role="menuitemradio" aria-checked={theme === item.id} className="theme-option" data-selected={theme === item.id} onClick={() => onChange(item.id)}>
          <span className={`theme-swatch theme-swatch--${item.id}`} />
          <span><strong>{item.name}</strong><small>{item.description}</small></span>
          {theme === item.id && <Check size={16} />}
        </button>
      ))}
    </div>
  )
}
