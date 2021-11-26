import type { PWebSocket } from './utils.js'

const sendToSelfAsDebug = true
export class Peer {
  data: Uint8Array = new Uint8Array()
  info?: any
  constructor(public readonly id: number, public readonly socket: PWebSocket) {}

  onJson(message: any, peers: readonly Peer[]) {
    if (message.action !== 'keep alive') {
      console.log('event.data:', message)
    }

    if (message.action === 'keep alive') {
      // noop
    } else if (message.action === 'device info') {
      this.info = message.info
      for (const peer of peers) peer.jsonSent.delete(this.id)
      if (sendToSelfAsDebug) this.jsonSent.delete(this.id)
    } else {
      console.log('message with unknown action', JSON.stringify(message.action))
    }

    this.sendBinaryUnsent()
    this.sendJsonUnsent(peers)
  }

  binaryUnsent = new Map<number, Uint8Array>()
  private binaryUnsentBytes() {
    let bytes = 0
    for (const v of this.binaryUnsent.values()) bytes += v.byteLength
    return bytes
  }

  private sendBinaryUnsent() {
    const count = this.binaryUnsent.size
    if (count === 0) return
    try {
      const sendBuffer = new Uint8Array(this.binaryUnsentBytes() + 4)
      const sizeView = new DataView(sendBuffer.buffer, 0, 4)
      sizeView.setInt32(0, count, true)
      let offset = 4
      for (const buff of this.binaryUnsent.values()) {
        sendBuffer.set(buff, offset)
        offset += buff.byteLength
      }

      this.socket.send(sendBuffer)
    } finally {
      this.binaryUnsent.clear()
    }
  }

  jsonSent = new Set<number>()

  private sendJsonUnsent(peers: readonly Peer[]) {
    const info = []
    for (const peer of sendToSelfAsDebug ? peers.concat([this]) : peers) {
      if (!this.jsonSent.has(peer.id)) {
        this.jsonSent.add(peer.id)
        info.push({ id: peer.id, info: peer.info })
      }
    }
    if (info.length === 0) return

    this.socket.send(JSON.stringify({ action: 'device info', info }))
  }

  onBinary(data: ArrayBuffer, peers: readonly Peer[]) {
    this.data = new Uint8Array(data)

    //console.log(Array.from(this.data.values()).join(' '))
    peers = Array.from(peers)
    for (const peer of peers) peer.binaryUnsent.set(this.id, this.data)
    if (sendToSelfAsDebug) this.binaryUnsent.set(this.id, this.data)

    this.sendBinaryUnsent()
    this.sendJsonUnsent(peers)
  }
}
