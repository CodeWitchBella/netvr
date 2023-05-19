/** @jsxImportSource @emotion/react */
import ReactDOM from 'react-dom/client'
import { Dashboard } from './dashboard/dashboard'
import { BrowserRouter, Route, Routes } from 'react-router-dom'
import { ThemeRoot } from './components/theme'
import { ErrorBoundary } from './components/error-boundary'
import { lazy } from 'react'

const SampleViz = lazy(() => import('./other/sample-viz'))
const DelayViz = lazy(() => import('./other/delay-viz'))

export async function run() {
  const events = document.querySelector('#events')!
  if (!events) throw new Error('Cant find #events')

  const root = ReactDOM.createRoot(events)
  root.render(
    <ThemeRoot>
      <ErrorBoundary>
        <BrowserRouter>
          <Routes>
            <Route index element={<Dashboard socketUrl={getSocketUrl()} />} />
            <Route path="sample-viz" element={<SampleViz />} />
            <Route path="delay-viz" element={<DelayViz />} />
            <Route path="*" element={<NotFound />} />
          </Routes>
        </BrowserRouter>
      </ErrorBoundary>
    </ThemeRoot>,
  )
}

function getSocketUrl() {
  const socketUrl = new URL(window.location.toString())
  socketUrl.pathname = '/ws'
  socketUrl.protocol = socketUrl.protocol === 'https:' ? 'wss:' : 'ws:'
  if (socketUrl.port === '3000') socketUrl.port = '13161'
  return socketUrl.toString()
}

function NotFound() {
  return <h3 css={{ margin: 8 }}>404: Not found</h3>
}
