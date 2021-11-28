import type { PWebSocket } from './utils.js'

const sendToSelfAsDebug = true
type Calibration = {
  id: number
  translate: { x: 0; y: 0; z: 0 }
  rotate: { x: 0; y: 0; z: 0 }
  scale: { x: 1; y: 1; z: 1 }
}

export class Peer {
  data: Uint8Array = new Uint8Array()
  deviceInfo?: any
  calibration: Calibration = {
    id: 0,
    translate: { x: 0, y: 0, z: 0 },
    rotate: { x: 0, y: 0, z: 0 },
    scale: { x: 1, y: 1, z: 1 },
  }
  constructor(public readonly id: number, public readonly socket: PWebSocket) {
    this.calibration.id = id
  }

  // called right after constructor
  onConnect(peers: readonly Peer[]) {
    // managed to reconnect before disconnect event was propagated
    for (const peer of peers) {
      peer.disconnectUnsent.delete(this.id)
    }
  }

  onJson(message: any, peers: readonly Peer[]) {
    if (message.action !== 'keep alive') {
      console.log('event.data:', message)
    }

    if (message.action === 'keep alive') {
      // noop
    } else if (message.action === 'device info') {
      this.deviceInfo = message.info
      for (const peer of peers) peer.deviceInfoSent.delete(this.id)
      if (sendToSelfAsDebug) this.deviceInfoSent.delete(this.id)
    } else if (message.action === 'set calibration') {
      for (const peer of peers.concat(this)) {
        for (const cal of message.calibrations) {
          peer.calibrationSent.delete(cal.id) // schedule send
          if (cal.id == peer.id) {
            // set calibration
            peer.calibration = cal
          }
        }
      }
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

  deviceInfoSent = new Set<number>()
  calibrationSent = new Set<number>()
  disconnectUnsent = new Set<number>()

  private sendJsonUnsent(peers: readonly Peer[]) {
    const deviceInfos = []
    const calibrations: (Calibration & { id: number })[] = []
    for (const peer of sendToSelfAsDebug ? peers.concat([this]) : peers) {
      if (!this.deviceInfoSent.has(peer.id)) {
        this.deviceInfoSent.add(peer.id)
        deviceInfos.push({ id: peer.id, info: peer.deviceInfo })
      }
      if (!this.calibrationSent.has(peer.id)) {
        this.calibrationSent.add(peer.id)
        calibrations.push(peer.calibration)
      }
    }
    if (deviceInfos.length !== 0) {
      this.socket.send(
        JSON.stringify({ action: 'device info', info: deviceInfos }),
      )
    }

    if (calibrations.length !== 0) {
      this.socket.send(
        JSON.stringify({ action: 'set calibration', calibrations }),
      )
    }

    if (this.disconnectUnsent.size > 0) {
      this.socket.send(
        JSON.stringify({
          action: 'disconnect',
          ids: Array.from(this.disconnectUnsent),
        }),
      )
      this.disconnectUnsent.clear()
    }
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

  onDisconnect(peers: readonly Peer[]) {
    console.log('onDisconnect', this.id)
    for (const peer of peers) {
      peer.disconnectUnsent.add(this.id)
    }
  }
}
