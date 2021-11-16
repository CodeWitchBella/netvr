import type { PWebSocket } from "./utils.ts";

const dataLength = 265;
const sendToSelfAsDebug = false;
export class Peer {
  data: Uint8Array = new Uint8Array();
  constructor(
    public readonly id: number,
    public readonly socket: PWebSocket,
  ) {}

  onJson(message: any, peers: Iterable<Peer>) {
    console.log(
      "message with unknown action",
      JSON.stringify(message.action),
    );
  }

  onBinary(data: ArrayBuffer, peers: Iterable<Peer>) {
    this.data = new Uint8Array(data);
  }

  // deno-lint-ignore require-await
  static async tick(clients: Iterable<Peer>) {
    const workingPeers: Peer[] = [];
    const otherPeers: Peer[] = [];
    for (const client of clients) {
      if (client.data.byteLength !== dataLength) otherPeers.push(client);
      else workingPeers.push(client);
    }
    if (workingPeers.length === 0) return;

    const sendPeerCount = (workingPeers.length - (sendToSelfAsDebug ? 0 : 1));
    const sendBuffer = completeBuffer(workingPeers, sendPeerCount);

    const sizeView = new DataView(sendBuffer.buffer, 0, 4);
    sizeView.setInt32(0, sendPeerCount, true);
    let offset = 4;
    for (const client of workingPeers) {
      if (
        sendToSelfAsDebug ||
        client !== workingPeers[workingPeers.length - 1]
      ) {
        sendBuffer.set(client.data, offset);
        offset += dataLength;
      }
    }

    if (sendToSelfAsDebug) {
      for (const { socket } of workingPeers) {
        if (socket.bufferedAmount > 0) continue;
        socket.send(sendBuffer);
      }
    } else if (workingPeers.length === 1) {
      // only one client, nothing to send
      const { socket } = workingPeers[0];
      if (socket.bufferedAmount === 0) {
        socket.send(sendBuffer);
      }
    } else {
      for (let i = 0; i < workingPeers.length; i++) {
        const prevI = i === 0 ? workingPeers.length - 1 : i - 1;
        const previous = workingPeers[prevI];
        // replace self with previous
        sendBuffer.set(
          previous.data,
          4 + (i === workingPeers.length - 1 ? 0 : i) * dataLength,
        );

        const { socket } = workingPeers[i];
        if (socket.bufferedAmount === 0) {
          socket.send(sendBuffer);
        }
      }
    }

    if (otherPeers.length > 0) {
      const sendBuffer = completeBuffer(workingPeers, workingPeers.length);
      for (const { socket } of otherPeers) {
        if (socket.bufferedAmount > 0) continue;
        socket.send(sendBuffer);
      }
    }
  }
}

function completeBuffer(workingPeers: readonly Peer[], sendPeerCount: number) {
  const sendBuffer = new Uint8Array(dataLength * sendPeerCount + 4);
  const sizeView = new DataView(sendBuffer.buffer, 0, 4);
  sizeView.setInt32(0, sendPeerCount, true);
  let offset = 4;
  for (let i = 0; i < sendPeerCount; ++i) {
    const client = workingPeers[i];
    sendBuffer.set(client.data, offset);
    offset += dataLength;
  }
  return sendBuffer;
}
