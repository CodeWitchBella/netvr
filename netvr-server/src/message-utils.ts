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
  reader: ReadableStreamDefaultReader,
  options: { actions: readonly T[]; timeout: number },
): Promise<
  'closed' | 'timeout' | 'invalid message' | { type: T; [key: string]: any }
> {
  const timeout = new Promise<void>(
    (res) => void setTimeout(res, options.timeout),
  )

  const actions = new Set(options.actions)
  while (true) {
    const result = await Promise.race([reader.read(), timeout])
    if (!result) return 'timeout'
    if (result.done) return 'closed'

    if (typeof result.value !== 'string') continue
    try {
      const obj = JSON.parse(result.value)
      if (typeof obj === 'object' && obj && actions.has(obj.action)) {
        return obj
      }
    } catch (e) {
      return 'invalid message'
    }
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
