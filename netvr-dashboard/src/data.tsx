import { notNull } from '@isbl/ts-utils'
import { locationMap } from './location-map'

export const protocolVersion = 2

export type DeviceBinaryData = {
  readonly deviceId: number
  readonly quaternion: readonly { x: number; y: number; z: number }[]
  readonly vector3: readonly { x: number; y: number; z: number }[]
  readonly vector2: readonly { x: number; y: number }[]
  readonly float: readonly number[]
  readonly bool: readonly (boolean | 'fail')[]
  readonly uint32: readonly number[]
}

export type ClientBinaryData = {
  clientId: number
  devices: readonly DeviceBinaryData[]
}

export type DeviceConfiguration = {
  locations: { [key: string]: number }
  name: string
  characteristics: string[]
  localId: number
  lengths: { [key: string]: number }
  data?: DeviceBinaryData
  haptics: null | {
    numChannels: 1
    supportsImpulse: true
    supportsBuffer: false
    bufferFrequencyHz: 0
    bufferMaxSize: 0
    bufferOptimalSize: 0
  }
}

export type ClientConfiguration = {
  connected: boolean
  connectionInfo: {
    ip: string
    deviceName?: string
    deviceModel?: string
    deviceUniqueIdentifier?: string
    graphicsDeviceName?: string
    operatingSystem?: string
    userName?: string
    isBrowser?: boolean
  }
  calibration: {
    translate: { x: number; y: number; z: number }
    rotate: { x: number; y: number; z: number }
    scale: { x: number; y: number; z: number }
  }
  devices?: DeviceConfiguration[]
}

export type ServerState = {
  clients: { [key: number]: ClientConfiguration }
}

export function mapData(
  rawData: DeviceBinaryData,
  configuration: DeviceConfiguration,
): {
  unknown: any
  data: {
    [key in keyof typeof locationMap]?: typeof locationMap[key] extends 'bool'
      ? { type: typeof locationMap[key]; value: boolean | 'fail' }
      : typeof locationMap[key] extends 'float' | 'uint32'
      ? { type: typeof locationMap[key]; value: number }
      : typeof locationMap[key] extends 'quaternion' | 'vector3'
      ? {
          type: typeof locationMap[key]
          value: { x: number; y: number; z: number }
        }
      : typeof locationMap[key] extends 'vector2'
      ? { type: typeof locationMap[key]; value: { x: number; y: number } }
      : never
  }
} {
  if (!rawData) return { unknown: [], data: {} }
  const visited = {
    quaternion: new Set<number>(),
    vector3: new Set<number>(),
    vector2: new Set<number>(),
    float: new Set<number>(),
    bool: new Set<number>(),
    uint32: new Set<number>(),
  } as const

  const data = Object.fromEntries(
    Object.entries(locationMap)
      .map(([key, type]) => {
        const loc = configuration.locations[key]
        const value = rawData?.[type][loc]
        if (loc < 0 || value === undefined) return null
        visited[type]?.add(loc)
        return [key, { type, value }]
      })
      .filter(Boolean) as any,
  )
  const unkownEntries = (
    Object.entries(visited) as [keyof typeof visited, Set<number>][]
  )
    .map(([key, set]) => {
      if (!rawData || !(key in rawData)) return null
      const items = (rawData[key] as any[])
        .filter((data, i) => !set.has(i))
        .map((data, index) => ({ data, index }))
      if (items.length < 1) return null
      return [key, items]
    })
    .filter(notNull)
  return {
    data,
    unknown:
      unkownEntries.length > 0 ? Object.fromEntries(unkownEntries) : null,
  }
}

export function parseBinaryMessage(data: ArrayBuffer) {
  try {
    const view = new DataView(data, 0, data.byteLength)
    var offset = { current: 0 }
    const messageType = readByte(view, offset)
    if (messageType === 1) {
      const clientCount = readUint32(view, offset)
      let clients: ClientBinaryData[] = []
      for (let clientIndex = 0; clientIndex < clientCount; ++clientIndex) {
        const clientId = readUint32(view, offset)
        const numberOfDevices = read7BitEncodedInt(view, offset)
        let devices: DeviceBinaryData[] = []
        for (let deviceId = 0; deviceId < numberOfDevices; ++deviceId) {
          const deviceByteCount = read7BitEncodedInt(view, offset)
          const offsetCopy = { ...offset }
          offset.current += deviceByteCount
          const deviceId = read7BitEncodedInt(view, offsetCopy)

          devices.push({
            deviceId,
            quaternion: readArray(view, offsetCopy, readVec3),
            vector3: readArray(view, offsetCopy, readVec3),
            vector2: readArray(view, offsetCopy, readVec2),
            float: readArray(view, offsetCopy, readFloat),
            bool: readArray(view, offsetCopy, readBool),
            uint32: readArray(view, offsetCopy, readUint32),
          })
        }
        clients.push({ devices, clientId })
      }
      return { clientCount, clients }
    }
    return { error: 'Unknown message type' }
  } catch (e: any) {
    console.log(e)
    return {
      error: 'failed to parse binary message',
      detail: {
        message: e?.message,
        error: e,
      },
    }
  }
}

function readUint32(view: DataView, offset: { current: number }) {
  const val = view.getUint32(offset.current, true)
  offset.current += 4
  return val
}

function readByte(view: DataView, offset: { current: number }) {
  const val = view.getUint8(offset.current)
  offset.current += 1
  return val
}

function readBool(view: DataView, offset: { current: number }) {
  const val = view.getUint8(offset.current)
  offset.current += 1
  return val === 0 ? false : val === 1 ? true : 'fail'
}

function readVec3(view: DataView, offset: { current: number }) {
  return {
    x: readFloat(view, offset),
    y: readFloat(view, offset),
    z: readFloat(view, offset),
  } as const
}

function readVec2(view: DataView, offset: { current: number }) {
  return { x: readFloat(view, offset), y: readFloat(view, offset) } as const
}

function readFloat(view: DataView, offset: { current: number }): number {
  const val = view.getFloat32(offset.current, true)
  offset.current += 4
  return val
}

function readArray<T>(
  view: DataView,
  offset: { current: number },
  readItem: (view: DataView, offset: { current: number }) => T,
) {
  const count = read7BitEncodedInt(view, offset)
  const array: T[] = []
  for (let i = 0; i < count; ++i) {
    array.push(readItem(view, offset))
  }
  return array
}

function read7BitEncodedInt(
  view: DataView,
  offset: { current: number },
): number {
  let val = 0
  let mul = 1
  let byte: number
  do {
    byte = readByte(view, offset)
    val += (byte & 0x7f) * mul
    mul *= 128
  } while (byte & 0x80)
  return val
}

export function sendHapticImpulse(
  sendMessage: (buffer: ArrayBuffer) => void,
  clientId: number,
  deviceId: number,
) {
  const buffer = new ArrayBuffer(21)
  const view = new DataView(buffer)
  view.setUint8(0, 2) // message type
  view.setUint32(1, clientId, true)
  view.setUint32(5, deviceId, true)
  view.setUint32(9, 0, true) // channel
  view.setFloat32(13, 0.25, true) // amplitude
  view.setFloat32(17, 0.1, true) // time (oculus-only)
  sendMessage(buffer)
}
