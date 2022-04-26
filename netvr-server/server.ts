/**
 * Server used for relaying messages between multiple users.
 *
 * Example invocation:
 * $ yarn task run
 *
 * To run in debug mode:
 * $ yarn task debug
 *
 * See compile.ts for instructions on how to produce executable from this file.
 */

import { index } from './src/paths.js'
import { netvrRoomOptions } from './src/netvr-handler.js'
import {
  type ConnectionInfo,
  createIdHandler,
} from './src/netvr-id-handler-layer.js'
import { wrapWebSocket } from './src/websocketstream.js'

await Deno.permissions.request({ name: 'net' })
await Deno.permissions.request({
  name: 'read',
  path: '../netvr-dashboard/dist',
})

const l = Deno.listen({ port: 10_000 })
console.log(
  `\nOpen your browser at %chttp://localhost:${l.addr.port}%c to see the management console\n`,
  'font-weight: bold; color: blue',
  'font-weight: normal; color: initial',
)

function save(data: string) {
  Deno.writeTextFile('netvr-room.json', data)
}

const room = await Deno.readTextFile('netvr-room.json').then(
  (savedData) => createIdHandler(netvrRoomOptions, { save }, savedData),
  () => createIdHandler(netvrRoomOptions, { save }),
)

async function main() {
  for await (const tcpConn of l) {
    handleConnection(tcpConn).catch((e) => void console.error(e))
  }
  console.log('server finished')
}

async function handleConnection(tcpConn: Deno.Conn) {
  for await (const event of Deno.serveHttp(tcpConn)) {
    console.log(event.request.url, event.request.headers.get('upgrade'))
    // dont block next request while current one is ongoing
    if (event.request.headers.get('upgrade') === 'websocket') {
      await serveSocket(event, {
        ip:
          'hostname' in tcpConn.remoteAddr
            ? tcpConn.remoteAddr.hostname
            : tcpConn.remoteAddr.path,
      })
    } else {
      const { pathname } = new URL(event.request.url)
      if (pathname === '/api/info' && event.request.method === 'GET') {
        event.respondWith(
          new Response(
            JSON.stringify({
              intefaces: Deno.networkInterfaces(),
              deno: { version: Deno.version, build: Deno.build },
            }),
            { headers: { 'content-type': 'application/json' } },
          ),
        )
      } else if (pathname.startsWith('/assets')) {
        await serveFile(pathname, event)
      } else {
        await serveFile(
          '/index.html',
          event,
          index.includes(pathname) ? 200 : 404,
        )
      }
    }
  }
}

async function serveSocket(
  event: Deno.RequestEvent,
  connectionInfo: ConnectionInfo,
) {
  const { socket, response } = Deno.upgradeWebSocket(event.request, {
    // there is a bug in deno which causes unrecoverable crash if this is enabled
    // https://github.com/denoland/deno/issues/14280
    idleTimeout: 0,
  })
  room.onWebSocket(wrapWebSocket(socket), connectionInfo)

  event.respondWith(response)
}

async function serveFile(
  pathname: string,
  event: Deno.RequestEvent,
  successStatus = 200,
) {
  try {
    const filepath = '../netvr-dashboard/dist' + pathname
    const fileContents = await Deno.readFile(filepath)
    const mime =
      mimetypes[pathname.substring(pathname.lastIndexOf('.') + 1)] ??
      mimetypes['bin']

    await event.respondWith(
      new Response(fileContents, {
        status: successStatus,
        headers: { 'Content-type': mime },
      }),
    )
  } catch {
    await event.respondWith(
      new Response('Not found', {
        status: 404,
        headers: { 'Content-type': mimetypes['txt'] },
      }),
    )
  }
}

const mimetypes: { [ext: string]: string } = {
  bin: 'application/octet-stream',
  css: 'text/css; charset=utf-8',
  html: 'text/html; charset=utf-8',
  jpeg: 'image/jpeg',
  jpg: 'image/jpeg',
  js: 'application/javascript; charset=utf-8',
  png: 'image/png',
  svg: 'image/svg+xml; charset=utf-8',
  txt: 'text/plain; charset=utf-8',
}
await main()
