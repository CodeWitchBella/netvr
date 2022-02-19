/**
 * Server used for relaying messages between multiple users. Alternative
 * entrypoint intended for running on cloudflare workers.
 *
 * Run using:
 * $ yarn miniflare --watch --debug --port 10000
 * or
 * $ yarn dev
 */

import { getAssetFromKV } from '@cloudflare/kv-asset-handler'
import manifestJSON from '__STATIC_CONTENT_MANIFEST'
import { index } from './src/paths.js'
import { netvrRoomOptions } from './src/netvr-handler.js'
import {
  createIdHandler,
  type NetvrIdHandlerLayer,
} from './src/netvr-id-handler-layer.js'
import { wrapWebSocket } from './src/websocketstream.js'
const manifest = JSON.parse(manifestJSON)

export default {
  async fetch(request: Request, env: any, ctx: ExecutionContext) {
    const upgradeHeader = request.headers.get('Upgrade')
    if (upgradeHeader === 'websocket') {
      const id = env.WEBSOCKET.idFromName('test')
      const object = env.WEBSOCKET.get(id)
      return object.fetch(request.url, request)
    }
    const originalUrl = new URL(request.url)
    const url = new URL(request.url)
    if (!url.pathname.startsWith('/assets')) url.pathname = '/'
    try {
      const response = await getAssetFromKV(
        {
          request: new Request(url.toString(), request),
          waitUntil: (promise: any) => ctx.waitUntil(promise),
        },
        { ASSET_NAMESPACE: env.__STATIC_CONTENT, ASSET_MANIFEST: manifest },
      )
      if (url.pathname === '/' && !index.includes(originalUrl.pathname)) {
        return new Response(response.body, { ...response, status: 404 })
      }
      return response
    } catch (e: any) {
      if (e.status === 404) return new Response('Not found', { status: 404 })
      return new Response(e.message, { status: e.status })
    }
  },
}

export class DurableObjectWebSocket {
  state
  room?: NetvrIdHandlerLayer

  save = (data: string) => {
    this.state.storage.put('data', data)
  }

  constructor(state: any, env: any) {
    this.state = state
  }

  async fetch(request: Request) {
    if (!this.room) {
      const data = await this.state.storage.get('data')
      this.room = createIdHandler(netvrRoomOptions, { save: this.save }, data)
    }

    const ip = request.headers.get('CF-Connecting-IP')

    const [client, server] = Object.values(new WebSocketPair())

    this.room.onWebSocket(wrapWebSocket(server))
    server.accept()
    server.send(JSON.stringify({ ip }))

    return new Response(null, {
      status: 101,
      webSocket: client,
    })

    let data = await request.text()
    let storagePromise = this.state.storage.put(ip, data)
    await storagePromise
    return new Response(ip + ' stored ' + data)
  }
}
