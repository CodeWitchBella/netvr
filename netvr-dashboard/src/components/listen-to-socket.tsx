import {
  useState,
  useRef,
  useEffect,
  memo,
  useReducer,
  PropsWithChildren,
  createContext,
  useContext,
} from 'react'

function useListenToSocket(
  socket: WebSocket,
  onMessage: (event: string | ArrayBuffer) => void,
) {
  const lastOnMessage = useRef(onMessage)
  useEffect(() => {
    lastOnMessage.current = onMessage
  })
  useEffect(() => {
    socket.addEventListener('message', onMessage)

    return () => {
      socket.removeEventListener('message', onMessage)
    }

    function onMessage({ data }: MessageEvent) {
      lastOnMessage?.current(data)
    }
  }, [socket])
}

export const ListenToSocket = memo(function ListenToSocket({
  socket,
  onMessage,
}: {
  socket: WebSocket
  onMessage: (event: string | ArrayBuffer) => void
}) {
  useListenToSocket(socket, onMessage)
  return null
})

function useSocketState(url: string, onDisconnected: () => void) {
  type SocketState =
    | { socket: WebSocket; status: 'connected' }
    | { status: 'connecting' | 'disconnected' }
  const [state, setSocket] = useReducer(
    (
      state: SocketState,
      action: { socket: WebSocket | null; url: string },
    ): SocketState =>
      action.url !== url
        ? state
        : action.socket
        ? { status: 'connected', socket: action.socket }
        : { status: 'disconnected' },
    { status: 'connecting' as const },
  )
  const [error, setError] = useState<Error | null>(null)

  const onDisconnectedRef = useRef(onDisconnected)
  useEffect(() => {
    onDisconnectedRef.current = onDisconnected
  })

  useEffect(() => {
    let isOpen = false
    let connectionTimeout = -1
    let socket = connect()
    return () => void cleanup(socket)

    function cleanup(target: WebSocket) {
      clearTimeout(connectionTimeout)
      target.removeEventListener('open', onOpen)
      target.removeEventListener('close', onClose)
      target.removeEventListener('error', onError)

      if (target.readyState < 2 /* open or connecting */) {
        target.close()
      }
    }

    function connect() {
      console.log('Connecting to', url)
      const ret = new WebSocket(url)
      isOpen = false
      ret.binaryType = 'arraybuffer'

      ret.addEventListener('open', onOpen)
      ret.addEventListener('close', onClose)
      ret.addEventListener('error', onError)

      connectionTimeout = setTimeout(() => {
        console.log('WebSocket:connectionTimeout')
        cleanup(socket)
        socket = connect()
      }, 5000)
      return ret
    }

    function onOpen(event: Event) {
      const target = event.currentTarget as WebSocket
      if (target !== socket) return

      console.log('WebSocket:open')
      clearTimeout(connectionTimeout)
      isOpen = true
      setSocket({ url: target.url, socket: target })
    }

    function onClose(event: Event) {
      const target = event.currentTarget as WebSocket
      if (target !== socket) return

      console.log('WebSocket:close')
      setSocket({ url, socket: null })
      onDisconnectedRef.current?.()
    }

    function onError(event: Event) {
      const target = event.currentTarget as WebSocket
      if (target !== socket) return

      console.log('WebSocket:error')
      if (!isOpen) {
        // reconnect
        cleanup(socket)
        socket = connect()
      } else if (
        event &&
        typeof event === 'object' &&
        'message' in event &&
        typeof (event as any).message === 'string'
      ) {
        setError(new Error((event as any).message))
      } else {
        console.error(event)
        setError(new Error('Error in websocket'))
      }
    }
  }, [url])
  if (error) throw error
  return state
}

const ctx = createContext<WebSocket | null>(null)
export function SocketProvider({
  children,
  url,
  onDisconnected,
}: PropsWithChildren<{ url: string; onDisconnected: () => void }>) {
  const state = useSocketState(url, onDisconnected)
  console.log(state)
  if (state.status === 'disconnected') return <div>Disconnected.</div>
  if (state.status === 'connected')
    return <ctx.Provider value={state.socket}>{children}</ctx.Provider>
  if (state.status === 'connecting') return <div>Connecting...</div>
  return null
}

export function useSocket() {
  const value = useContext(ctx)
  if (!value) throw new Error('Missing SocketProvider')
  return value
}
