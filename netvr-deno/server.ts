/**
 * Server used for relaying messages between multiple users.
 *
 * Example invocation:
 * $ deno run --unstable --allow-net --watch server.ts
 */
import { iterateReader } from "https://deno.land/std@0.114.0/streams/conversion.ts";

const port = 10_000;
const udp = Deno.listenDatagram({ port, transport: "udp" });
const tcp = Deno.listen({ port, transport: "tcp" });
console.log(udp.addr);

/**
 * Global list of recently connected peers to which we want to send updates
 */
let peers = new Set<{
  timestamp: number;
  hostname: string;
  port: number;
}>();

Promise.all([
  listenTcp(handleTcpConnection),
  listenUdp(handleUdpRequest),
])
  .catch((e) => {
    // something failed spectacularly bad, do not attempt to recover and quit instead
    console.error(e);
    Deno.exit(1);
  });

/**
 * Listen to messages on UDP server port
 */
async function listenUdp(
  // deno-lint-ignore no-explicit-any
  handler: (data: Uint8Array, peer: Deno.NetAddr) => any,
) {
  for await (const [data, peer] of udp) {
    if (peer.transport === "udp") {
      Promise.resolve(handler(data, peer)).catch((e) => {
        console.error(e);
      });
    } else {
      console.warn("Recieved message with unexpected transport", peer);
    }
  }
}

/**
 * Listen to messages on UDP server port
 */
async function listenTcp(
  // deno-lint-ignore no-explicit-any
  handler: (conn: Deno.Conn) => Promise<any>,
) {
  for await (const connection of tcp) {
    handler(connection).catch((e) => {
      console.error(e);
    }).then(function _finally() {
      // close the connection upon handleTcpConnection finishing
      connection.close();
    });
  }
}

/**
 * Add peer to list of peers and remove old timeouted peers.
 *
 * @param peer to add to list
 * @param timestamp current time
 */
function addPeerAndCleanup(peer: Deno.NetAddr, timestamp: number) {
  const timeout = 60 * 1000; // 60 seconds
  let exists = false;
  for (const existingPeer of peers) {
    if (existingPeer.timestamp + timeout < timestamp) {
      peers.delete(existingPeer);
    }
    if (
      peer.hostname === existingPeer.hostname && peer.port === existingPeer.port
    ) {
      existingPeer.timestamp = timestamp;
      exists = true;
    }
  }
  if (!exists) {
    peers.add({
      hostname: peer.hostname,
      port: peer.port,
      timestamp,
    });
  }
}

/**
 * Function to handle incoming message from client. Mostly updates info to be
 * broadcast to other peers.
 *
 * @param data binary incoming data
 * @param peer information about peer which sent the data
 */
async function handleUdpRequest(data: Uint8Array, peer: Deno.NetAddr) {
  addPeerAndCleanup(peer, Date.now());
  console.log("new request", data, peer);
  console.log("peers", peers);

  await udp.send(new TextEncoder().encode("abcd"), peer);
}

/**
 * Function to handle incoming connection from client.
 *
 * @param conn object representing open tcp connection
 */
async function handleTcpConnection(conn: Deno.Conn) {
  conn.write(new TextEncoder().encode(JSON.stringify({ type: "hello" })));
  const buffer = new Uint8Array(2048);
  while (true) {
    const length = await conn.read(buffer);
    if (length === null) return;
    console.log("tcp message:", buffer.slice(0, length));
  }
}
