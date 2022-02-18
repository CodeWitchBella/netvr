import { useCallback, useEffect, useState } from 'react'

export type PWebSocket = ReturnType<typeof promisifyWebsocket>
type Message = ArrayBuffer | string

export function promisifyWebsocket(socket: WebSocket) {
  const messageQueue: (
    | { type: 'message'; value: MessageEvent<Message> }
    | { type: 'error'; value: unknown }
    | { type: 'close' }
  )[] = []
  let finished = false

  let onQueueHasMessage: (() => void) | null = null
  let onQueueHasMessageForOpened: (() => void) | null = null

  socket.addEventListener('message', (e) => {
    messageQueue.push({ type: 'message', value: e })
    onQueueHasMessage?.()
    onQueueHasMessageForOpened?.()
  })
  socket.onerror = (e) => {
    messageQueue.push({ type: 'error', value: e })
    onQueueHasMessage?.()
    onQueueHasMessageForOpened?.()
    finished = true
  }
  socket.addEventListener('close', () => {
    messageQueue.push({ type: 'close' })
    onQueueHasMessage?.()
    onQueueHasMessageForOpened?.()
    finished = true
  })

  const opened = new Promise<void>((resolve, reject) => {
    socket.onopen = () => resolve()
    onQueueHasMessageForOpened = () => {
      if (socket.readyState === 1 /* OPEN */) resolve()
      else reject(new Error('Failed to open websocket'))
    }
  }).then(() => {
    onQueueHasMessageForOpened = null
  })

  return {
    url: socket.url,
    get bufferedAmount() {
      const amount = socket.bufferedAmount
      return !Number.isInteger(amount) ? 0 : amount
    },
    opened,
    send(data: string | ArrayBufferLike | Blob | ArrayBufferView): void {
      socket.send(data)
    },
    get closed() {
      return new Promise<void>((resolve) => {
        socket.addEventListener('close', () => resolve())
      })
    },
    [Symbol.asyncIterator](): AsyncIterator<MessageEvent<Message>> {
      return {
        next() {
          const promise = new Promise<IteratorResult<MessageEvent<Message>>>(
            (resolve, reject) => {
              onQueueHasMessage = () => {
                onQueueHasMessage = null
                const message = messageQueue.splice(0, 1)[0]
                if (message.type === 'message') {
                  resolve({ value: message.value })
                } else if (message.type === 'error') {
                  reject(message.value)
                } else {
                  resolve({ done: true, value: null })
                }
              }
            },
          )
          if (messageQueue.length > 0) {
            // run this even if finished to properly consume queue first
            onQueueHasMessage?.()
          } else if (finished) {
            // if next is called after resolve with { done: true } error out
            return Promise.reject(new Error('WebSocket is closed'))
          }
          return promise
        },
      }
    },
  }
}

export function useLocalStorage<Value extends string = string>(
  key: string,
  defaultValue: Value,
  validate: (v: string) => v is Value,
) {
  const [state, setState] = useState<{ key: string; value: string | null }>(
    () => ({
      key,
      value: localStorage.getItem(key),
    }),
  )
  useEffect(() => {
    setState((prev) =>
      prev.key === key ? prev : { key, value: localStorage.getItem(key) },
    )
    window.addEventListener('storage', listener)
    return () => window.removeEventListener('storage', listener)
    function listener(event: StorageEvent) {
      if (event.key === key) {
        console.log(event)
        const newValue = event.newValue
        setState((prev) =>
          prev.value === newValue ? prev : { key, value: newValue },
        )
      }
    }
  }, [key])

  return [
    state.value === null
      ? defaultValue
      : validate(state.value)
      ? state.value
      : defaultValue,
    useCallback(
      (value) => {
        if (validate(value)) {
          localStorage.setItem(key, value)
          setState({ key, value })
        }
      },
      [key, validate],
    ),
  ] as const
}
