type MessageOfTypeResult<T> =
  | 'closed'
  | 'timeout'
  | 'invalid message'
  | { type: T; [key: string]: any }

/**
 * Takes result of awaitMesageOfType and handles the state of it being something
 * other than correct message by notifying the client and closing the connection
 *
 * Returns true if it did something.
 */
export function assertMessageResult<T>(
  message: MessageOfTypeResult<T>,
): asserts message is { type: T; [key: string]: any } {
  if (message === 'closed') throw new ExpectedError()
  if (message === 'invalid message') {
    throw new ExpectedError({ error: 'invalid message' })
  }
  if (message === 'timeout') {
    throw new ExpectedError({ error: 'timeout' })
  }
}

/**
 * Reads messages from connection until it either gets a message with specified
 * action or times out. Returns the parsed message, or string representing status.
 *
 * Ignores valid messages which do not match any action.
 */
export async function awaitMesageWithAction<T extends string>(
  socket: WebSocket,
  options: { actions: readonly T[]; timeout: number },
) {
  const actions = new Set(options.actions)
  return new Promise<
    'closed' | 'timeout' | 'invalid message' | { type: T; [key: string]: any }
  >((resolve, reject) => {
    const timeout = setTimeout(() => {
      resolve('timeout')
      socket.removeEventListener('message', onMessage)
    }, options.timeout)

    socket.addEventListener('message', onMessage)

    function onMessage(event: MessageEvent) {
      if (typeof event.data !== 'string') return
      try {
        const obj = JSON.parse(event.data)
        if (typeof obj === 'object' && obj && actions.has(obj.action)) {
          resolve(obj)
          socket.removeEventListener('message', onMessage)
          clearTimeout(timeout)
        }
      } catch (e) {
        reject('invalid message')
        socket.removeEventListener('message', onMessage)
        clearTimeout(timeout)
      }
    }
  })
}

/**
 * wrapper around socket.onMessage which
 * - waits for previous message to finish processing before dispatching next one
 * - returns Promise which resolves on websocket close, resolves on websocket
 *   error and rejects on callback error
 * - also resolves on client timeout (same as close) and rejects on handler timeout
 */
export async function messageLoop(
  {
    socket,
    clientTimeoutMs,
    handlerTimeoutMs,
    identifier,
  }: {
    socket: WebSocket
    clientTimeoutMs: number
    handlerTimeoutMs: number
    identifier: string
  },
  onMessageIn: (event: MessageEvent) => Promise<any>,
): Promise<void> {
  let queue = Promise.resolve()
  let reject = (error: any) => {}
  let receiveTimeout = setTimeout(onTimeout, clientTimeoutMs)
  const timedOut = Symbol('timed out')
  try {
    return await new Promise<void>((resolve, realReject) => {
      reject = realReject
      socket.addEventListener('message', onMessage)
      socket.addEventListener('close', () => {
        console.log('WebSocket:close', identifier)
        resolve()
      })
      socket.addEventListener('error', (event) => {
        console.error(
          'WebSocket:error',
          identifier,
          event && 'message' in event ? (event as any).message : event,
        )
        resolve()
      })
    })
  } finally {
    clearTimeout(receiveTimeout)
    socket.removeEventListener('message', onMessage)
  }
  function onMessage(event: MessageEvent) {
    resetClientTimer()
    queue = queue
      .then(() => Promise.race([onMessageIn(event), receiveTimeoutPromise()]))
      .then((result) => {
        if (result === timedOut) {
          throw new Error('Server did not handle the message in specified time')
        }
      })
      .catch(reject)
  }

  function receiveTimeoutPromise() {
    return new Promise<typeof timedOut>(
      (resolve) => void setTimeout(resolve, handlerTimeoutMs, timedOut),
    )
  }
  function resetClientTimer() {
    clearTimeout(receiveTimeout)
    receiveTimeout = setTimeout(onTimeout, clientTimeoutMs)
  }
  function onTimeout() {
    console.log('WebSocket:timeout', identifier)
    socket.close()
  }
}

/**
 * When thrown from connection handler it causes connection to be close AND
 * before it closes the client is also informed about the reason.
 */
export class ExpectedError extends Error {
  clientMessage: string | undefined
  constructor(msg?: { [key: string]: any }) {
    super(
      'Expected error, if you see this it was thrown from unexpected place, though',
    )
    if (msg) {
      this.clientMessage = JSON.stringify(msg)
    }
    ;(Error as any).captureStackTrace?.(this, this.constructor)
  }
}
