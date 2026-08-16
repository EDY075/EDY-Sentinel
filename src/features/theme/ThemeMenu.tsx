import { Check, Palette } from 'lucide-react'
import type { ThemeName } from '../../types/system'

const themes: Array<{ id: ThemeName; name: string; description: string }> = [
  { id: 'sentinel-blue', name: 'Sentinel Blue', description: 'Focused and precise' },
  { id: 'cyber-green', name: 'Cyber Green', description: 'Operational and calm' },
  { id: 'terminal', name: 'Terminal', description: 'Minimal and technical' },
  { id: 'spectrum', name: 'Spectrum', description: 'Vivid, controlled depth' },
]

export function ThemeMenu({ theme, onChange }: { theme: ThemeName; onChange: (theme: ThemeName) => void }) {
  return (
    <div className="theme-menu" aria-label="Theme selector">
      <div className="theme-menu__title"><Palette size={15} /> Interface theme</div>
      {themes.map((item) => (
        <button key={item.id} type="button" className="theme-option" data-selected={theme === item.id} onClick={() => onChange(item.id)}>
          <span className={`theme-swatch theme-swatch--${item.id}`} />
          <span><strong>{item.name}</strong><small>{item.description}</small></span>
          {theme === item.id && <Check size={16} />}
        </button>
      ))}
    </div>
  )
}
