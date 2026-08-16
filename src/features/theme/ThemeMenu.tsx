import { useEffect, useRef } from 'react'
import { Check, Palette } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { ThemeName } from '../../types/system'

const themes: Array<{ id: ThemeName; key: string }> = [
  { id: 'sentinel-blue', key: 'sentinelBlue' },
  { id: 'cyber-green', key: 'cyberGreen' },
  { id: 'terminal', key: 'terminal' },
  { id: 'spectrum', key: 'spectrum' },
]

export function ThemeMenu({ theme, onChange, onClose }: { theme: ThemeName; onChange: (theme: ThemeName) => void; onClose: () => void }) {
  const { t } = useTranslation('shell')
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
    <div ref={menuRef} className="theme-menu" role="menu" aria-label={t('theme.selector')} onKeyDown={onKeyDown}>
      <div className="theme-menu__title"><Palette size={15} /> {t('theme.title')}</div>
      {themes.map((item) => (
        <button key={item.id} type="button" role="menuitemradio" aria-checked={theme === item.id} className="theme-option" data-selected={theme === item.id} onClick={() => onChange(item.id)}>
          <span className={`theme-swatch theme-swatch--${item.id}`} />
          <span><strong>{t(`theme.options.${item.key}.name`)}</strong><small>{t(`theme.options.${item.key}.description`)}</small></span>
          {theme === item.id && <Check size={16} />}
        </button>
      ))}
    </div>
  )
}
