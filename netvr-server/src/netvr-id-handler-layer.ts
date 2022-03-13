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
export type ConnectionInfo = { [key: string]: unknown }

export interface NetvrIdHandlerLayer {
  onWebSocket(socket: WebSocketStream, connectionInfo: ConnectionInfo): void
}

export type NetvrRoomOptions<RestoreData> = {
  newConnection: (id: number, connectionInfo: ConnectionInfo) => NetvrHandler
  restoreConnection: (
    id: number,
    connectionInfo: ConnectionInfo,
  ) => NetvrHandler
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
      socket: WebSocketStream
    }
  | {}
)

export type NetvrServerImpl = {
  save(data: string): void
}

export function createIdHandler<RestoreData>(
  getOpts: (utils: Utils) => NetvrRoomOptions<RestoreData>,
  impl: NetvrServerImpl,
  restoreData?: string | null,
): NetvrIdHandlerLayer {
  let data = restoreData ? JSON.parse(restoreData) : { clients: [] }
  console.log(data)
  let state = {
    clients: new Map<number, Client>(
      data?.clients.map((c: any) => [c.id, c]) as any,
    ),
    idGen: 0,
    saveTriggered: { current: false },
  }
  for (const client of state.clients.values()) {
    if (client.id >= state.idGen) state.idGen = client.id
  }
  let opts = getOpts(createUtils(state.clients, state.saveTriggered))
  if (restoreData) opts.restore(data.handler)
  data = { clients: [] }

  const reset = () => {
    for (const client of state.clients.values()) {
      if ('socket' in client) {
        console.log('Closing socket')
        client.writer.close()
        client.socket.close()
      }
    }
    state = {
      clients: new Map<number, Client>(),
      idGen: 0,
      saveTriggered: { current: false },
    }
    opts = getOpts(createUtils(state.clients, state.saveTriggered))

    result = idHandlerInternal(state, opts, impl, reset)
    result.save()
  }
  let result = idHandlerInternal(state, opts, impl, reset)

  return {
    onWebSocket(socket, connectionInfo) {
      result.onWebSocket(socket, connectionInfo)
    },
  }
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
  reset: () => void,
) {
  return {
    constructTime: new Date().toISOString(),
    onWebSocket(socket: WebSocketStream, connectionInfo: ConnectionInfo) {
      ;(async () => {
        const connection = await socket.connection
        const reader = connection.readable.getReader()
        const writer = connection.writable.getWriter()
        let isClosed = false
        writer.closed.then(() => {
          isClosed = true
        })

        await handleConnection({ reader, writer, socket, connectionInfo })
          .catch(async (e: any) => {
            if (e instanceof ExpectedError) {
              if (!e.clientMessage) {
                // do nothing
              } else if (!isClosed) {
                await writer.write(e.clientMessage)
              } else {
                console.warn(
                  "Can't send ExpectedError.clientMessage because connection is already closed",
                )
              }
            } else if (!isClosed) {
              // notify this client of error thrown in the server
              if (typeof e === 'object' && e && e.message) {
                await writer.write(
                  JSON.stringify({ message: e.message, stack: e.stack }),
                )
              } else {
                await writer.write(JSON.stringify({ message: 'unknown error' }))
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
            if (!isClosed) return writer.close()
          })
      })()
        .catch((e) => {
          console.error('This is somewhat bad - failed to close the socket')
          console.error(e)
        })
        .then(() => {
          try {
            socket.close()
          } catch {}
        })
    },
    save,
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
    socket,
    connectionInfo,
  }: {
    reader: ReadableStreamDefaultReader
    writer: WritableStreamDefaultWriter
    socket: WebSocketStream
    connectionInfo: ConnectionInfo
  }) {
    const setupMessage = await awaitMesageWithAction(reader, {
      timeout: 15000,
      actions: ['i already has id', 'gimme id'],
    })
    assertMessageResult(setupMessage)
    assertProtocolVersion(opts.protocolVersion, setupMessage)

    let client: Client | null = null
    if (setupMessage.action === 'i already has id') {
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
          handler: opts.restoreConnection(oldClient.id, connectionInfo),
          id: requestedId,
          token: oldClient.token,
          reader,
          writer,
          socket,
        }
        state.clients.set(client.id, client)
        handleSaveTrigger()
      } else {
        // can't use client-provided ID, give a new one.
      }
    }
    if (!client) {
      const id = ++state.idGen
      client = {
        handler: opts.newConnection(id, connectionInfo),
        id,
        token: getRandomString(64),
        reader,
        writer,
        socket,
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
              if (data.action === 'reset room') {
                reset()
              } else {
                client.handler.onJson(data)
                handleSaveTrigger()
              }
            } else {
              writer.write(
                JSON.stringify({
                  error: 'Invalid message recieved',
                }),
              )
            }
          } else {
            client.handler.onBinary(result.value)
            handleSaveTrigger()
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
      state.clients.set(client.id, {
        id: client.id,
        token: client.token,
      })
      client.handler.destroy()
      handleSaveTrigger()
    }
  }

  function save() {
    state.saveTriggered.current = false
    impl.save(
      JSON.stringify(
        {
          clients: Array.from(state.clients.values()).map((v) => ({
            id: v.id,
            token: v.token,
          })),
          handler: opts.save(),
        },
        null,
        2,
      ),
    )
  }

  function handleSaveTrigger() {
    if (state.saveTriggered.current) {
      save()
    }
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
