import { promisifyWebsocket, PWebSocket } from './utils'
import ReactDOM from 'react-dom'
import { Dashboard } from './dashboard'

export async function run() {
  const events = document.querySelector('#events')!
  if (!events) throw new Error('Cant find #events')
  const socket = await promisifyWebsocket(createSocket())
  ;(window as any).socket = socket
  socket.send(JSON.stringify({ action: 'gimme id' }))

  ReactDOM.render(<Dashboard socket={socket} />, events)

  await socket.closed

  function createSocket() {
    const socketUrl = new URL(window.location.toString())
    socketUrl.pathname = '/ws'
    socketUrl.protocol = socketUrl.protocol === 'https:' ? 'wss:' : 'ws:'
    if (socketUrl.port === '3000') socketUrl.port = '10000'
    const socket = new WebSocket(socketUrl.toString())
    socket.binaryType = 'arraybuffer'
    return socket
  }
}
