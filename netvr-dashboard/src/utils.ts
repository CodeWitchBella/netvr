type Await<T extends Promise<unknown>> = T extends Promise<infer V> ? V : never
export type PWebSocket = Await<ReturnType<typeof promisifyWebsocket>>
type Message = ArrayBuffer | string

export async function promisifyWebsocket(socket: WebSocket) {
  const messageQueue: (
    | { type: 'message'; value: MessageEvent<Message> }
    | { type: 'error'; value: unknown }
    | { type: 'close' }
  )[] = []
  let finished = false

  let onQueueHasMessage: (() => void) | null = null

  socket.addEventListener('message', (e) => {
    messageQueue.push({ type: 'message', value: e })
    onQueueHasMessage?.()
  })
  socket.onerror = (e) => {
    messageQueue.push({ type: 'error', value: e })
    onQueueHasMessage?.()
    finished = true
  }
  socket.addEventListener('close', () => {
    messageQueue.push({ type: 'close' })
    onQueueHasMessage?.()
    finished = true
  })

  await new Promise<void>((resolve) => {
    socket.onopen = () => resolve()
    onQueueHasMessage = resolve
  })
  onQueueHasMessage = null

  return {
    get bufferedAmount() {
      const amount = socket.bufferedAmount
      return !Number.isInteger(amount) ? 0 : amount
    },
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
            }
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
