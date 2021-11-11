/**
 * Server used for relaying messages between multiple users.
 *
 * Example invocation:
 * $ deno run --unstable --allow-net --watch server.ts
 */

const port = 10_000;
const udp = Deno.listenDatagram({ port, transport: "udp" });
const tcp = Deno.listen({ port, transport: "tcp" });
const textEncoder = new TextEncoder();

/**
 * Global list of recently connected peers to which we want to send updates
 */
class Peer {
  udp?: Deno.NetAddr;
  constructor(public uniqueId: number, public tcp: Deno.Conn) {}
}
const peers = new Map</* uniqueId */ number, Peer>();
let idgen = 1;

Promise.all([
  listenTcp(handleTcpConnection),
  listenUdp(handleUdpRequest),
])
  .catch((e) => {
    // something failed spectacularly bad, do not attempt to recover and quit instead
    console.error(e);
    Deno.exit(1);
  });
console.log("Started");

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
 * Function to handle incoming message from client. Mostly updates info to be
 * broadcast to other peers.
 *
 * @param data binary incoming data
 * @param peer information about peer which sent the data
 */
async function handleUdpRequest(data: Uint8Array, peer: Deno.NetAddr) {
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
  let remainingData = new Uint8Array(0);
  const buffer = new Uint8Array(2048);
  const textDecoder = new TextDecoder("utf-8");

  while (true) {
    const length = await conn.read(buffer);
    if (length === null) return;

    remainingData = concat(remainingData, buffer.subarray(0, length));
    while (remainingData.length > 4) {
      const view = new DataView(
        remainingData.buffer,
        remainingData.byteOffset,
        remainingData.byteLength,
      );
      const stringLength = view.getUint32(0, true);
      if (remainingData.byteLength < stringLength + 4) break;

      const message = textDecoder.decode(
        remainingData.subarray(4, stringLength + 4),
      );
      await handleTcpMessage(conn, JSON.parse(message));
      remainingData = remainingData.subarray(stringLength + 4);
    }
  }
}

/**
 * Concatenates two TypedArrays into a third, newly allocated one
 */
function concat(a: ArrayLike<number>, b: ArrayLike<number>) {
  const res = new Uint8Array(a.length + b.length);
  res.set(a);
  res.set(b, a.length);
  return res;
}

// deno-lint-ignore no-explicit-any
async function handleTcpMessage(conn: Deno.Conn, message: any): Promise<void> {
  if (message.action === "gimme id") {
    if (conn.remoteAddr.transport !== "tcp") {
      throw new Error("TCP message should come from TCP");
    }
    const uniqueId = ++idgen;
    const bytes = textEncoder.encode(
      "1234" + JSON.stringify({ action: "id's here", intValue: uniqueId }) +
        "\n",
    );
    new DataView(bytes.buffer, 0, bytes.length).setUint32(
      0,
      bytes.length - 4,
      true,
    );

    peers.set(uniqueId, new Peer(uniqueId, conn));
    conn.write(bytes);
  }
  console.log("message", message);
  await Promise.resolve();
}
