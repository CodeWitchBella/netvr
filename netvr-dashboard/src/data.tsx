import { notNull } from '@isbl/ts-utils'
import { locationMap } from './location-map'

export const protocolVersion = 1

export type DeviceBinaryData = {
  readonly deviceId: number
  readonly deviceByteCount: number
  readonly bytes: string
  readonly quaternion: readonly (readonly [number, number, number])[]
  readonly vector3: readonly (readonly [number, number, number])[]
  readonly vector2: readonly (readonly [number, number])[]
  readonly float: readonly number[]
  readonly bool: readonly (boolean | 'fail')[]
  readonly uint32: readonly number[]
}
export type DeviceData = {
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
export type ClientData = {
  id: number
  info?: readonly DeviceData[]
}

export function mapData(device: DeviceData): {
  unknown: any
  data: {
    [key in keyof typeof locationMap]?: typeof locationMap[key] extends 'bool'
      ? { type: typeof locationMap[key]; value: boolean | 'fail' }
      : typeof locationMap[key] extends 'float' | 'uint32'
      ? { type: typeof locationMap[key]; value: number }
      : typeof locationMap[key] extends 'quaternion' | 'vector3'
      ? {
          type: typeof locationMap[key]
          value: readonly [number, number, number]
        }
      : typeof locationMap[key] extends 'vector2'
      ? { type: typeof locationMap[key]; value: readonly [number, number] }
      : never
  }
} {
  if (!device.data) return { unknown: [], data: {} }
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
        const loc = device.locations[key]
        const value = device.data?.[type][loc]
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
      if (!device.data || !(key in device.data)) return null
      const items = (device.data[key] as any[])
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
      let clients = []
      for (let clientIndex = 0; clientIndex < clientCount; ++clientIndex) {
        const clientId = readUint32(view, offset)
        const numberOfDevices = read7BitEncodedInt(view, offset)
        let devices: DeviceBinaryData[] = []
        for (let deviceId = 0; deviceId < numberOfDevices; ++deviceId) {
          const deviceByteCount = read7BitEncodedInt(view, offset)
          const deviceBytes = view.buffer.slice(
            offset.current,
            offset.current + deviceByteCount,
          )
          const offsetCopy = { ...offset }
          offset.current += deviceByteCount
          const deviceId = read7BitEncodedInt(view, offsetCopy)

          devices.push({
            deviceId,
            deviceByteCount,
            bytes: new Uint8Array(deviceBytes).join(' '),
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
  return [
    readFloat(view, offset),
    readFloat(view, offset),
    readFloat(view, offset),
  ] as const
}

function readVec2(view: DataView, offset: { current: number }) {
  return [readFloat(view, offset), readFloat(view, offset)] as const
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
