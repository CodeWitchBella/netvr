import { useEffect, useReducer, useRef, useState } from 'react'
import type { PWebSocket } from './utils'

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

function useSendKeepAlive(socket: PWebSocket) {
  useEffect(() => {
    const interval = setInterval(() => {
      socket.send(JSON.stringify({ action: 'keep alive' }))
    }, 250)
    return () => {
      clearInterval(interval)
    }
  }, [socket])
}

type MessageData<T extends 'binary' | 'json'> = {
  type: T
  key: number
  message: T extends 'binary' ? ArrayBuffer : any
  timestamp: string
}

export function Dashboard({ socket }: { socket: PWebSocket }) {
  const [devices, dispatchDevices] = useReducer(
    (state: { id: number; [key: string]: any }[], action: any) => {
      return state
    },
    [],
  )
  const [log, dispatchLog] = useReducer(
    (
      state: {
        events: MessageData<'json'>[]
        binaryEvents: MessageData<'binary'>[]
        keyGen: number
      },
      action: any,
    ) => {
      const timestamp = new Date().toISOString()
      if (typeof action === 'string') {
        return {
          ...state,
          events: state.events.concat({
            type: 'json',
            message: JSON.parse(action),
            key: state.keyGen,
            timestamp,
          }),
          keyGen: state.keyGen + 1,
        }
      }
      return {
        ...state,
        keyGen: state.keyGen + 1,
        binaryEvents: state.binaryEvents
          .concat({
            type: 'binary',
            message: action,
            key: state.keyGen,
            timestamp,
          })
          .slice(-10),
      }
    },
    { events: [], binaryEvents: [], keyGen: 1 },
  )
  useListenToSocket(socket, (message) => {
    dispatchLog(message)
    dispatchDevices(message)
  })
  useSendKeepAlive(socket)

  return (
    <div className="events">
      {([] as readonly (MessageData<'binary'> | MessageData<'json'>)[])
        .concat(log.events)
        .concat(log.binaryEvents)
        .sort((a, b) => -a.timestamp.localeCompare(b.timestamp))
        .map((event) => (
          <Message
            message={event.message}
            key={event.key}
            timestamp={event.timestamp}
            type={event.type}
          />
        ))}
    </div>
  )
}

function Message({
  message,
  timestamp,
  type,
}: {
  message: any
  timestamp: string
  type: 'binary' | 'json'
}) {
  return (
    <div className="event">
      <div>🔽 {timestamp}</div>
      <pre>
        {type === 'binary' ? message + '' : JSON.stringify(message, null, 2)}
      </pre>
    </div>
  )
}
