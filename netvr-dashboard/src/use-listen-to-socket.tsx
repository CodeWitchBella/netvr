import { useState, useRef, useEffect } from 'react'
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

export function useListenToSocket(
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
