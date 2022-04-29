import { protocolVersion } from './data'

export function restoreConnectionFromBrowser(
  deviceName: string | null,
  data: { id: number; token: string },
) {
  return JSON.stringify({
    action: 'i already has id',
    protocolVersion,
    info: {
      deviceName: deviceName ?? undefined,
      isBrowser: true,
    },
    ...data,
  })
}

export function establishNewConnectionFromBrowser(deviceName: string | null) {
  return JSON.stringify({
    action: 'gimme id',
    protocolVersion,
    info: {
      deviceName: deviceName ?? undefined,
      isBrowser: true,
    },
  })
}

export function requestLogs(client: number) {
  return JSON.stringify({
    action: 'request logs',
    client,
  })
}

export function quit(client: number) {
  return JSON.stringify({
    action: 'quit',
    client,
  })
}

export function hapticImpulse({
  clientId,
  deviceId,
}: {
  clientId: number
  deviceId: number
}) {
  const buffer = new ArrayBuffer(21)
  const view = new DataView(buffer)
  view.setUint8(0, 2) // message type
  view.setUint32(1, clientId, true)
  view.setUint32(5, deviceId, true)
  view.setUint32(9, 0, true) // channel
  view.setFloat32(13, 0.25, true) // amplitude
  view.setFloat32(17, 0.1, true) // time (oculus-only)
  return buffer
}

export function resetRoom() {
  return JSON.stringify({ action: 'reset room' })
}

export function beginCalibration(data: {
  leader: number
  follower: number
  leaderDevice: number
  followerDevice: number
}) {
  return JSON.stringify({ feature: 'calibration', action: 'begin', ...data })
}

export function setCalibration(
  data: readonly {
    client: number
    value: {
      translate: { x: number; y: number; z: number }
      rotate: { x: number; y: number; z: number }
      scale: { x: number; y: number; z: number }
    }
  }[],
) {
  return JSON.stringify({
    action: 'multiset',
    data: data.map((v) => ({ field: 'calibration', ...v })),
  })
}
