import { promisifyWebsocket, PWebSocket } from './utils'
import ReactDOM from 'react-dom'
import { Dashboard } from './dashboard'

export async function run() {
  const events = document.querySelector('#events')!
  if (!events) throw new Error('Cant find #events')

  ReactDOM.render(<Dashboard socketUrl={getSocketUrl()} />, events)
}

function getSocketUrl() {
  const socketUrl = new URL(window.location.toString())
  socketUrl.pathname = '/ws'
  socketUrl.protocol = socketUrl.protocol === 'https:' ? 'wss:' : 'ws:'
  if (socketUrl.port === '3000') socketUrl.port = '10000'
  return socketUrl.toString()
}
