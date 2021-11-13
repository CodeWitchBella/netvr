/**
 * Server used for relaying messages between multiple users.
 *
 * Example invocation:
 * $ deno run --unstable --allow-net --watch server.ts
 */

import { getRandomString, promisifyWebsocket, PWebSocket } from "./utils.ts";

const l = Deno.listen({ port: 10_000 });

let idgen = 0;

const peersById = new Map<number, Peer>();
const peerTokens = new Map<number, string>();

class Peer {
  token;
  constructor(
    public id: number,
    public socket: PWebSocket,
  ) {
    this.token = peerTokens.get(id) ?? getRandomString(64);
    peerTokens.set(id, this.token);
  }
}

for await (const tcpConn of l) {
  for await (const event of Deno.serveHttp(tcpConn)) {
    const { socket, response } = Deno.upgradeWebSocket(event.request);
    onSocket(socket)
      .catch((e) => void console.error(e))
      .then(() => {
        console.log(
          socket.readyState === WebSocket.CLOSING
            ? "WebSocket.CLOSING"
            : socket.readyState === WebSocket.CLOSED
            ? "WebSocket.CLOSED"
            : socket.readyState === WebSocket.OPEN
            ? "WebSocket.OPEN"
            : socket.readyState,
        );
        if (
          socket.readyState !== WebSocket.CLOSING &&
          socket.readyState !== WebSocket.CLOSED
        ) {
          socket.close();
        }
      });
    event.respondWith(response);
  }
}

/**
 * Function to handle incoming connection from client.
 *
 * @param conn object representing open tcp connection
 */
async function onSocket(socketIn: WebSocket) {
  const socket = await promisifyWebsocket(socketIn);
  let thisPeer: Peer | null = null;
  try {
    for await (const event of socket) {
      if (event.data instanceof ArrayBuffer) {
        //console.log(new Uint8Array(event.data));
        // binary data
        continue;
      }
      console.log("event.data:", event.data);
      const message = JSON.parse(event.data);
      if (message.action === "keep alive") {
        // ignore
      } else if (message.action === "gimme id") {
        thisPeer = createPeer(++idgen, socket);
        socket.send(
          JSON.stringify({
            action: "id's here",
            intValue: thisPeer.id,
            stringValue: thisPeer.token,
          }),
        );
      } else if (message.action === "i already has id") {
        const requestedId = message.id;
        const token = peerTokens.get(requestedId);
        if (token && token === message.token) {
          thisPeer = createPeer(requestedId, socket);
          console.log(peersById);
        } else {
          thisPeer = createPeer(++idgen, socket);
          console.log(peersById);
          socket.send(
            JSON.stringify({
              action: "id's here",
              intValue: thisPeer.id,
              stringValue: thisPeer.token,
            }),
          );
        }
      } else {
        console.log(
          "message with unknown action",
          JSON.stringify(message.action),
        );
      }
    }
  } finally {
    if (thisPeer) {
      peersById.delete(thisPeer.id);
      console.log(peersById);
    }
    console.log("Socket finished");
  }
}

function createPeer(...params: ConstructorParameters<typeof Peer>) {
  const peer = new Peer(...params);
  peersById.set(peer.id, peer);
  console.log(peersById);
  return peer;
}
