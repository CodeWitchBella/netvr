import { getRandomString } from '../utils'
import {
  awaitMesageWithAction,
  ExpectedError,
  assertMessageResult,
} from './message-utils'
import type { WebSocketStream } from './websocketstream'

export interface NetvrHandler<RestoreData> {
  save(): RestoreData
  onJson(message: { [type: string]: string }): void
  onBinary(message: ArrayBuffer): void
  destroy(): void
}

export interface NetvrRoom {
  onWebSocket(socket: WebSocketStream): void
}

export type NetvrRoomOptions<RestoreData> = {
  newConnection: (id: number) => NetvrHandler<RestoreData>
  restoreConnection: (data: RestoreData) => NetvrHandler<RestoreData>
  protocolVersion: number
}

type BroadcastOptions = { omit: number }

export type Utils = {
  send(id: number, message: { action: string; [key: string]: any }): void
  broadcast(
    message: { action: string; [key: string]: any },
    opts?: { omit: number },
  ): void

  sendBinary(id: number, message: ArrayBuffer): void
  broadcastBinary(message: ArrayBuffer, opts?: BroadcastOptions): void
}

type Client<RestoreData> = {
  token: string
  id: number
} & (
  | {
      handler: NetvrHandler<RestoreData>
      reader: ReadableStreamDefaultReader
      writer: WritableStreamDefaultWriter
    }
  | {
      restoreData: RestoreData
    }
)

/**
 * TODO: jsdoc
 */
export function createNetvrRoom<RestoreData>(
  getOpts: (utils: Utils) => NetvrRoomOptions<RestoreData>,
): NetvrRoom {
  const state = {
    clients: new Map<number, Client<RestoreData>>(),
    idGen: 0,
  }
  const opts = getOpts(createUtils(state.clients))

  return {
    onWebSocket(socket: WebSocketStream) {
      const reader = socket.connection.readable.getReader()
      const writer = socket.connection.writable.getWriter()
      handleConnection({ reader, writer })
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
        .catch((e) => {
          console.error('This is somewhat bad - failed to close the socket')
          console.error(e)
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

    let client: Client<RestoreData> | null = null
    if (setupMessage.type === 'i already has id') {
      const requestedId = setupMessage.id
      const oldClient = state.clients.get(requestedId)
      if (
        oldClient &&
        'restoreData' in oldClient &&
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
          handler: opts.restoreConnection(oldClient.restoreData),
          id: requestedId,
          token: oldClient.token,
          reader,
          writer,
        }
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
            const obj = JSON.parse(result.value)
            client.handler.onJson(obj)
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
      state.clients.set(client.id, {
        id: client.id,
        token: client.token,
        restoreData: client.handler.save(),
      })
    }
  }
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

function createUtils(clients: Map<number, Client<unknown>>): Utils {
  function sendRaw(id: number, message: ArrayBuffer | string) {
    const client = clients.get(id)
    if (!client) {
      throw new Error(`Client ${id} not found`)
    } else if ('restoreData' in client) {
      throw new Error(`Client ${id} is currently disconnected`)
    } else {
      client.writer.write(message)
    }
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
    send: (id, message) => {
      sendRaw(id, JSON.stringify(message))
    },
    sendBinary: sendRaw,
    broadcast: (message, options) => {
      broadcastRaw(JSON.stringify(message), options)
    },
    broadcastBinary: broadcastRaw,
  }
}
