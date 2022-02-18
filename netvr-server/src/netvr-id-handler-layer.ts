import { getRandomString } from './utils.js'
import {
  awaitMesageWithAction,
  ExpectedError,
  assertMessageResult,
} from './message-utils.js'
import type { WebSocketStream } from './websocketstream.js'

export interface NetvrHandler {
  onJson(message: { action: string; [key: string]: any }): void
  onBinary(message: ArrayBuffer): void
  destroy(): void
}

export interface NetvrIdHandlerLayer {
  onWebSocket(socket: WebSocketStream): void
}

export type NetvrRoomOptions<RestoreData> = {
  newConnection: (id: number) => NetvrHandler
  restoreConnection: (id: number) => NetvrHandler
  protocolVersion: number
  save: () => RestoreData
  restore: (data: RestoreData) => void
  destroy: () => void
}

type BroadcastOptions = { omit: number }

export type Utils = {
  triggerSave(): void
  send(id: number, message: { action: string; [key: string]: any }): void
  broadcast(
    message: { action: string; [key: string]: any },
    opts?: { omit: number },
  ): void

  sendBinary(id: number, message: ArrayBuffer): void
  broadcastBinary(message: ArrayBuffer, opts?: BroadcastOptions): void
  readonly clients: Iterable<number>
}

type Client = {
  token: string
  id: number
} & (
  | {
      handler: NetvrHandler
      reader: ReadableStreamDefaultReader
      writer: WritableStreamDefaultWriter
    }
  | {}
)

export type NetvrServerImpl = {
  save(data: string): void
}

export function createIdHandler<RestoreData>(
  getOpts: (utils: Utils) => NetvrRoomOptions<RestoreData>,
  impl: NetvrServerImpl,
): NetvrIdHandlerLayer {
  const state = {
    clients: new Map<number, Client>(),
    idGen: 0,
    saveTriggered: { current: false },
  }

  const opts = getOpts(createUtils(state.clients, state.saveTriggered))

  return idHandlerInternal(state, opts, impl)
}

export function restoreIdHandler<RestoreData>(
  getOpts: (utils: Utils) => NetvrRoomOptions<RestoreData>,
  impl: NetvrServerImpl,
  restoreData: string,
): NetvrIdHandlerLayer {
  const data = JSON.parse(restoreData)
  const state = {
    clients: new Map<number, Client>(Object.entries(data.clients) as any),
    idGen: 0,
    saveTriggered: { current: false },
  }
  for (const id of state.clients.keys()) {
    if (id >= state.idGen) state.idGen = id + 1
  }

  const opts = getOpts(createUtils(state.clients, state.saveTriggered))
  opts.restore(data.handler)
  return idHandlerInternal(state, opts, impl)
}

/**
 * TODO: jsdoc
 */
function idHandlerInternal<RestoreData>(
  state: {
    clients: Map<number, Client>
    idGen: number
    saveTriggered: { current: boolean }
  },
  opts: NetvrRoomOptions<RestoreData>,
  impl: NetvrServerImpl,
): NetvrIdHandlerLayer {
  return {
    onWebSocket(socket: WebSocketStream) {
      ;(async () => {
        const connection = await socket.connection
        const reader = connection.readable.getReader()
        const writer = connection.writable.getWriter()
        await handleConnection({ reader, writer })
          .catch((e: any) => {
            if (e instanceof ExpectedError) {
              if (!e.clientMessage) {
                // do nothing
              } else if (!writer.closed) {
                writer.write(e.clientMessage)
              } else {
                console.warn(
                  "Can't send ExpectedError.clientMessage because connection is already closed",
                )
              }
            } else if (!writer.closed) {
              // notify this client of error thrown in the server
              if (typeof e === 'object' && e && e.message) {
                writer.write(
                  JSON.stringify({ message: e.message, stack: e.stack }),
                )
              } else {
                writer.write(JSON.stringify({ message: 'unknown error' }))
              }
            } else {
              console.error('Error occured after connection was closed')
              console.error(e)
            }
          })
          .catch((e) => {
            console.error('This is somewhat bad - error handler failed')
            console.error(e)
          })
          .then(() => {
            if (!writer.closed) writer.close()
          })
      })()
        .catch((e) => {
          console.error('This is somewhat bad - failed to close the socket')
          console.error(e)
        })
        .then(() => {
          socket.close()
        })
    },
  }

  /**
   * This is where the magic happens. First it handles basic connection
   * setup/restore. Then it delegates messages to the connection object.
   * @param param0
   * @returns
   */
  async function handleConnection({
    reader,
    writer,
  }: {
    reader: ReadableStreamDefaultReader
    writer: WritableStreamDefaultWriter
  }) {
    const setupMessage = await awaitMesageWithAction(reader, {
      timeout: 15000,
      actions: ['i already has id', 'gimme id'],
    })
    assertMessageResult(setupMessage)
    assertProtocolVersion(opts.protocolVersion, setupMessage)

    let client: Client | null = null
    if (setupMessage.type === 'i already has id') {
      const requestedId = setupMessage.id
      const oldClient = state.clients.get(requestedId)
      if (
        oldClient &&
        !('handler' in oldClient) &&
        oldClient.token === setupMessage.token
      ) {
        // If unconnected client with id exists and token matches.
        // This takes care of many situations at once:
        //  - malicious client. Not important since I do not really handle it in
        //    different parts.
        //  - accidental impersonation. This might happen in multiple situations
        //    examples are in following bullet points.
        //  - server lost data, but client did not. Client will try to reconnect
        //    but since the token does not match, the ID will get reset.
        //  - file with token got copied to different machine and both machines
        //    are trying to connect. The second machine will get new ID, instead
        //    of breaking the whole thing in unpredictable ways.
        // For those reasons I decided to handle the case of duplicate IDs
        // transparently by just assigning new ID instead of throwing and error.
        writer.write(
          JSON.stringify({
            action: 'id ack',
            protocolVersion: opts.protocolVersion,
          }),
        )
        client = {
          handler: opts.restoreConnection(oldClient.id),
          id: requestedId,
          token: oldClient.token,
          reader,
          writer,
        }
        state.clients.set(client.id, client)
      } else {
        // can't use client-provided ID, give a new one.
      }
    }
    if (!client) {
      const id = ++state.idGen
      client = {
        id,
        handler: opts.newConnection(id),
        reader,
        writer,
        token: getRandomString(64),
      }
      state.clients.set(id, client)
      save()
      writer.write(
        JSON.stringify({
          action: "id's here",
          intValue: id,
          stringValue: client.token,
          protocolVersion: opts.protocolVersion,
        }),
      )
    }

    try {
      while (true) {
        const result = await reader.read()
        if (result.done) return
        try {
          if (typeof result.value === 'string') {
            const data = safeJsonParse(result.value)
            if (
              typeof data === 'object' &&
              data &&
              typeof data.action === 'string'
            ) {
              client.handler.onJson(data)
            } else {
              writer.write(
                JSON.stringify({
                  error: 'Invalid message recieved',
                }),
              )
            }
          } else {
            client.handler.onBinary(result.value)
          }
        } catch (e: any) {
          if (typeof e === 'object' && e) {
            console.error(e)
            writer.write(
              JSON.stringify({
                error: e.message,
                stack: e.stack,
              }),
            )
          }
        }
      }
    } finally {
      console.log('Disconnect')
      state.clients.set(client.id, {
        id: client.id,
        token: client.token,
      })
      client.handler.destroy()
    }
  }

  function save() {
    impl.save(
      JSON.stringify({
        clients: Array.from(state.clients.values()).map((v) => ({
          id: v.id,
          token: v.token,
        })),
        handler: opts.save(),
      }),
    )
  }
}

function safeJsonParse(text: string) {
  try {
    return JSON.parse(text)
  } catch {}
  return null
}

function assertProtocolVersion(
  protocolVersion: number,
  message: { [key: string]: unknown },
) {
  if (message.protocolVersion !== protocolVersion) {
    throw new ExpectedError({
      error: 'Protocol version mismatch.',
      server: protocolVersion,
      client:
        typeof message.protocolVersion === 'number' &&
        Number.isInteger(message.protocolVersion)
          ? message.protocolVersion
          : 0,
    })
  }
}

function createUtils(
  clients: Map<number, Client>,
  saveTriggered: { current: boolean },
): Utils {
  function sendRaw(id: number, message: ArrayBuffer | string) {
    Promise.resolve().then(() => {
      const client = clients.get(id)
      if (!client) {
        throw new Error(`Client ${id} not found`)
      } else if ('writer' in client) {
        client.writer.write(message)
      } else {
        throw new Error(`Client ${id} is currently disconnected`)
      }
    })
  }
  function broadcastRaw(
    message: ArrayBuffer | string,
    broadcastOptions?: BroadcastOptions,
  ) {
    const omit = broadcastOptions?.omit
    for (const client of clients.values()) {
      if ('handler' in client && client.id !== omit) {
        client.writer.write(message)
      }
    }
  }
  return {
    triggerSave: () => {
      saveTriggered.current = true
    },
    send: (id, message) => {
      sendRaw(id, JSON.stringify(message))
    },
    sendBinary: sendRaw,
    broadcast: (message, options) => {
      broadcastRaw(JSON.stringify(message), options)
    },
    broadcastBinary: broadcastRaw,
    get clients() {
      return getClientIds()
    },
  }
  function* getClientIds() {
    for (const client of clients.values()) {
      if ('handler' in client) yield client.id
    }
  }
}
