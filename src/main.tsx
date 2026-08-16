import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { TelemetryProvider } from './features/telemetry/TelemetryProvider.tsx'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <TelemetryProvider><App /></TelemetryProvider>
  </StrictMode>,
)
