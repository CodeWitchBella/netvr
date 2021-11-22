import { useEffect, useReducer, useRef, useState } from 'react'
import type { PWebSocket } from './utils'

function cancellableAsyncIterable<Data>(
  signal: AbortSignal,
  iterable: AsyncIterable<Data>
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
  onMessage: (event: string | ArrayBuffer) => void
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
        socket
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

export function Dashboard({ socket }: { socket: PWebSocket }) {
  const [events, dispatch] = useReducer(
    (state: { key: number; message: any; timestamp: string }[], action: any) =>
      state
        .concat({
          message: action,
          key: (state[state.length - 1]?.key ?? 0) + 1,
          timestamp: new Date().toISOString(),
        })
        .slice(-10),
    []
  )
  useListenToSocket(socket, dispatch)
  useSendKeepAlive(socket)

  return (
    <div className="events">
      {events.map((event) => (
        <Message
          message={event.message}
          key={event.key}
          timestamp={event.timestamp}
        />
      ))}
    </div>
  )
}

function Message({ message, timestamp }: { message: any; timestamp: string }) {
  return (
    <div className="event">
      <div>🔽 {timestamp}</div>
      <pre>
        {typeof message === 'string'
          ? JSON.stringify(JSON.parse(message), null, 2)
          : stringify(message)}
      </pre>
    </div>
  )
}

function stringify(data: ArrayBuffer) {
  let view = new DataView(data, 0, data.byteLength)
  const length = view.getInt32(0, true)
  let res = ''
  for (let i = 0; i < length; ++i) {
    view = new DataView(data, 4 + 79 * i, 79)
    res += `#${view.getInt32(0, true)}`
    res += getTypePosRot(view, 4)
    res += getTypePosRot(view, 29)
    res += getTypePosRot(view, 54)
  }
  return res
}

function getTypePosRot(view: DataView, offset: number) {
  return `
    type: ${view.getUint8(offset)}
    position: ${getVector3(view, offset + 1)}
    rotation: ${getVector3(view, offset + 13)}`
}

function getVector3(view: DataView, offset: number) {
  const fixed = 3
  return `${view.getFloat32(offset, true).toFixed(fixed)}, ${view
    .getFloat32(offset + 4, true)
    .toFixed(fixed)}, ${view.getFloat32(offset + 8, true).toFixed(fixed)}`
}
