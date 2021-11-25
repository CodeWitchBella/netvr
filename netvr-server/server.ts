/**
 * Server used for relaying messages between multiple users.
 *
 * Example invocation:
 * $ deno run --watch --import-map ./import_map.json --allow-net --allow-read=../netvr-dashboard/dist server.ts
 * Or:
 * $ yarn deno:run
 * See compile.ts for instructions on how to produce executable from this file.
 */

import { createRoom } from './room.js'

await Deno.permissions.request({ name: 'net' })
await Deno.permissions.request({
  name: 'read',
  path: '../netvr-dashboard/dist',
})

const l = Deno.listen({ port: 10_000 })
console.log(l.addr)
console.log(Deno.build)
console.log(Deno.version)
const room = createRoom()

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
      await serveSocket(event)
    } else {
      const { pathname } = new URL(event.request.url)
      if (pathname === '/') {
        await serveFile('/index.html', event)
      } else {
        await serveFile(pathname, event)
      }
    }
  }
}

const socketState = {
  CONNECTING: 0, // Socket has been created. The connection is not yet open.
  OPEN: 1, // The connection is open and ready to communicate.
  CLOSING: 2, // The connection is in the process of closing.
  CLOSED: 3, // The connection is closed or couldn't be opened.
}

async function serveSocket(event: Deno.RequestEvent) {
  const { socket, response } = Deno.upgradeWebSocket(event.request)
  room
    .onSocket(socket)
    .catch((e) => void console.error(e))
    .then(() => {
      console.log(
        socket.readyState === socketState.CLOSING
          ? 'WebSocket.CLOSING'
          : socket.readyState === socketState.CLOSED
          ? 'WebSocket.CLOSED'
          : socket.readyState === socketState.OPEN
          ? 'WebSocket.OPEN'
          : socket.readyState === socketState.CONNECTING
          ? 'WebSocket.CONNECTING'
          : socket.readyState,
      )
      if (
        socket.readyState !== socketState.CLOSING &&
        socket.readyState !== socketState.CLOSED
      ) {
        socket.close()
      }
    })
  event.respondWith(response)
}

async function serveFile(pathname: string, event: Deno.RequestEvent) {
  try {
    const filepath = '../netvr-dashboard/dist' + pathname
    const fileContents = await Deno.readFile(filepath)
    const mime =
      mimetypes[pathname.substr(pathname.lastIndexOf('.') + 1)] ??
      mimetypes['bin']

    await event.respondWith(
      new Response(fileContents, {
        status: 200,
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
