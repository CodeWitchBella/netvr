/**
 * Server used for relaying messages between multiple users.
 *
 * Example invocation:
 * $ deno run --unstable --allow-net --watch server.ts
 */

const socket = Deno.listenDatagram({ port: 10000, transport: "udp" });
console.log(socket.addr);

/**
 * Global list of recently connected peers to which we want to send updates
 */
let peers = new Set<{
  timestamp: number;
  hostname: string;
  port: number;
}>();

for await (const [data, peer] of socket) {
  if (peer.transport === "udp") {
    handleRequest(data, peer).catch((e) => {
      console.error(e);
    });
  } else {
    console.warn("Recieved message with unexpected transport", peer);
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
// deno-lint-ignore require-await
async function handleRequest(data: Uint8Array, peer: Deno.NetAddr) {
  addPeerAndCleanup(peer, Date.now());
  console.log("new request", data, peer);
  console.log("peers", peers);

  await socket.send(new TextEncoder().encode("abcd"), peer);
}
