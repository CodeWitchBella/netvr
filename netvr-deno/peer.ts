import type { PWebSocket } from "./utils.ts";

const dataLength = 79;
export class Peer {
  data?: ArrayBuffer;
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
    this.data = data;
  }

  // deno-lint-ignore require-await
  static async tick(clients: Iterable<Peer>) {
    const workingPeers: Peer[] = [];
    for (const client of clients) {
      workingPeers.push(client);
    }
    if (workingPeers.length === 0) return;

    const sendBuffer = new Uint8Array(dataLength * workingPeers.length + 4);
    const sizeView = new DataView(sendBuffer.buffer, 0, 4);
    sizeView.setInt32(0, workingPeers.length, true);
    let offset = 4;
    for (const client of workingPeers) {
      if (!client.data || client.data.byteLength < dataLength) continue;
      sendBuffer.set(
        new Uint8Array(client.data, 0, dataLength),
        offset,
      );
      offset += dataLength;
    }

    // TODO: do not self own data
    for (const client of workingPeers) {
      if (client.socket.bufferedAmount > 0) continue;
      client.socket.send(sendBuffer);
    }
  }
}
