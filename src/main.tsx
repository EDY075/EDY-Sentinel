import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { TelemetryProvider } from './features/telemetry/TelemetryProvider.tsx'
import { initializeI18n } from './i18n'

async function bootstrap(): Promise<void> {
  await initializeI18n()
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <TelemetryProvider><App /></TelemetryProvider>
    </StrictMode>,
  )
}

void bootstrap()
