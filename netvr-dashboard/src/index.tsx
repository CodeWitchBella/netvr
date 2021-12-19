import ReactDOM from 'react-dom'
import { Dashboard } from './dashboard'
import { BrowserRouter, Route, Routes } from 'react-router-dom'
import { NotFound } from './not-found'
import { Menu } from './menu'
import { lazy, Suspense } from 'react'

const Thesis = lazy(() => import('./thesis/thesis'))
export async function run() {
  const events = document.querySelector('#events')!
  if (!events) throw new Error('Cant find #events')

  ReactDOM.render(
    <BrowserRouter>
      <Menu />
      <Routes>
        <Route index element={<Dashboard socketUrl={getSocketUrl()} />} />
        <Route
          path="/thesis"
          element={
            <Suspense fallback={null}>
              <Thesis />
            </Suspense>
          }
        />
        <Route element={<NotFound />} />
      </Routes>
    </BrowserRouter>,
    events,
  )
}

function getSocketUrl() {
  const socketUrl = new URL(window.location.toString())
  socketUrl.pathname = '/ws'
  socketUrl.protocol = socketUrl.protocol === 'https:' ? 'wss:' : 'ws:'
  if (socketUrl.port === '3000') socketUrl.port = '10000'
  return socketUrl.toString()
}
