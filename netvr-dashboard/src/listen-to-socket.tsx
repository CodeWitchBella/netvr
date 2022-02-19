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
import { promisifyWebsocket, PWebSocket } from './utils'

function cancellableAsyncIterable<Data>(
  signal: AbortSignal,
  iterable: AsyncIterable<Data>,
): AsyncIterable<Data> {
  const iterator = iterable[Symbol.asyncIterator]()
  const rejectedOnAbort = new Promise<never>((resolve, reject) => {
    signal.addEventListener('abort', () => {
      reject(new DOMException('Aborted', 'AbortError'))
    })
  })
  return {
    [Symbol.asyncIterator](): AsyncIterator<Data> {
      return {
        next() {
          return Promise.race([iterator.next(), rejectedOnAbort])
        },
      }
    },
  }
}

function useListenToSocket(
  socket: PWebSocket,
  onMessage: (event: string | ArrayBuffer) => void,
) {
  const [error, setError] = useState(null)
  const lastOnMessage = useRef(onMessage)
  useEffect(() => {
    lastOnMessage.current = onMessage
  })
  useEffect(() => {
    const controller = new AbortController()

    ;(async () => {
      for await (const message of cancellableAsyncIterable(
        controller.signal,
        socket,
      )) {
        lastOnMessage?.current(message.data as any)
      }
    })().catch((err) => setError(err))

    return () => {
      controller.abort()
    }
  }, [socket])
  if (error) throw error
}

export const ListenToSocket = memo(function ListenToSocket({
  socket,
  onMessage,
}: {
  socket: PWebSocket
  onMessage: (event: string | ArrayBuffer) => void
}) {
  useListenToSocket(socket, onMessage)
  return null
})

function useSocketState(url: string) {
  type SocketState =
    | { socket: PWebSocket; status: 'connected' }
    | { status: 'connecting' | 'disconnected' }
  const [state, setSocket] = useReducer(
    (
      state: SocketState,
      action: { socket: PWebSocket | null; url: string },
    ): SocketState =>
      action.url !== url
        ? state
        : action.socket
        ? { status: 'connected', socket: action.socket }
        : { status: 'disconnected' },
    { status: 'connecting' as const },
  )

  useEffect(() => {
    const socketRaw = new WebSocket(url)
    socketRaw.binaryType = 'arraybuffer'
    const socket = promisifyWebsocket(socketRaw)
    socket.opened.then(
      () => void setSocket({ url, socket }),
      () => void setSocket({ url, socket: null }),
    )
    return () => {
      socketRaw.close()
    }
  }, [url])
  return state
}

const ctx = createContext<PWebSocket | null>(null)
export function SocketProvider({
  children,
  url,
}: PropsWithChildren<{ url: string }>) {
  const state = useSocketState(url)
  useEffect(() => {
    if (state.status === 'disconnected') window.location.reload()
  })
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
