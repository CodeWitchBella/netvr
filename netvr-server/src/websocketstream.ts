export interface WebSocketStream {
  // new(url: string, options?: { protocols?: readonly string[] }): WebSocketStream
  connection: { readable: ReadableStream; writable: WritableStream }
}

class WebSocketStreamPonyfill implements WebSocketStream {
  constructor(public connection: WebSocketStream['connection']) {}
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

  socket.addEventListener('message', (event) => {
    writer.write(event.data)
  })
  socket.addEventListener('close', () => {
    if (!writer.closed) writer.close()
  })
  ;(async () => {
    while (true) {
      const { value, done } = await reader.read()
      if (done) {
        socket.close()
        if (!writer.closed) writer.close()
        break
      }
      socket.send(value)
    }
  })()

  return new WebSocketStreamPonyfill({
    readable: readable.readable,
    writable: writable.writable,
  })
}
