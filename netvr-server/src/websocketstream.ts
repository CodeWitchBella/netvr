export interface WebSocketStream {
  // new(url: string, options?: { protocols?: readonly string[] }): WebSocketStream
  connection: Promise<{ readable: ReadableStream; writable: WritableStream }>
  close(): void
}

class WebSocketStreamPonyfill implements WebSocketStream {
  #socket: WebSocket
  constructor(
    socket: WebSocket,
    public connection: WebSocketStream['connection'],
  ) {
    this.#socket = socket
  }
  close() {
    this.#socket.close()
  }
}

/**
 * Naive ponyfill for WebSocketStream API. I hope that before I turn in the
 * thesis this will not be needed on Clouflare Workers. It's already
 * unneccessary on deno (using --unstable). Also this is in no way
 * spec-compliant, nor is it efficient.
 *
 * @param socket WebSocket
 * @returns WebSocketStream
 */
export function wrapWebSocket(socket: WebSocket): WebSocketStream {
  const readable = new TransformStream()
  const writable = new TransformStream()

  const writer = readable.writable.getWriter()
  const reader = writable.readable.getReader()

  let closed = false
  socket.addEventListener('message', (event) => {
    if (!closed) writer.write(event.data)
  })
  socket.addEventListener('close', () => {
    closed = true
    writer.close().catch(() => {})
  })
  ;(async () => {
    while (true) {
      const { value, done } = await reader.read()
      if (done) {
        socket.close()
        break
      }
      socket.send(value)
    }
  })()

  return new WebSocketStreamPonyfill(
    socket,
    Promise.resolve({
      readable: readable.readable,
      writable: writable.writable,
    }),
  )
}
