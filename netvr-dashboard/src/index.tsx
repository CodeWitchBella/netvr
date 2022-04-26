import ReactDOM from 'react-dom/client'
import { Dashboard } from './dashboard'
import { BrowserRouter, Route, Routes } from 'react-router-dom'
import { NotFound } from './not-found'
import { ThemeProvider } from './use-theme'
import { ErrorBoundary } from './error-boundary'

export async function run() {
  const events = document.querySelector('#events')!
  if (!events) throw new Error('Cant find #events')

  const root = ReactDOM.createRoot(events)
  root.render(
    <ThemeProvider>
      <ErrorBoundary>
        <BrowserRouter>
          <Routes>
            <Route index element={<Dashboard socketUrl={getSocketUrl()} />} />
            <Route path="*" element={<NotFound />} />
          </Routes>
        </BrowserRouter>
      </ErrorBoundary>
    </ThemeProvider>,
  )
}

function getSocketUrl() {
  const socketUrl = new URL(window.location.toString())
  socketUrl.pathname = '/ws'
  socketUrl.protocol = socketUrl.protocol === 'https:' ? 'wss:' : 'ws:'
  if (socketUrl.port === '3000') socketUrl.port = '10000'
  return socketUrl.toString()
}
