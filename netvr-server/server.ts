/**
 * Server used for relaying messages between multiple users.
 *
 * Example invocation:
 * $ deno run --allow-net --watch server.ts
 */

import { createRoom } from "./room.ts";

declare const Deno: any;

await Deno.permissions.request({ name: "net" });

const l = Deno.listen({ port: 10_000 });
console.log(l.addr);
const room = createRoom();

const socketState = {
  CONNECTING: 0, // Socket has been created. The connection is not yet open.
  OPEN: 1, // The connection is open and ready to communicate.
  CLOSING: 2, // The connection is in the process of closing.
  CLOSED: 3, // The connection is closed or couldn't be opened.
};

for await (const tcpConn of l) {
  for await (const event of Deno.serveHttp(tcpConn)) {
    const { socket, response } = Deno.upgradeWebSocket(event.request);
    room.onSocket(socket)
      .catch((e) => void console.error(e))
      .then(() => {
        console.log(
          socket.readyState === socketState.CLOSING
            ? "WebSocket.CLOSING"
            : socket.readyState === socketState.CLOSED
            ? "WebSocket.CLOSED"
            : socket.readyState === socketState.OPEN
            ? "WebSocket.OPEN"
            : socket.readyState === socketState.CONNECTING
            ? "WebSocket.CONNECTING"
            : socket.readyState,
        );
        if (
          socket.readyState !== socketState.CLOSING &&
          socket.readyState !== socketState.CLOSED
        ) {
          socket.close();
        }
      });
    event.respondWith(response);
  }
}
