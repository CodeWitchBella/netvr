import { useEffect, useMemo, useReducer, useRef, useState } from 'react'
import { useLog } from './log'
import { ListenToSocket } from './listen-to-socket'
import type { PWebSocket } from './utils'

function useSendKeepAlive(socket: PWebSocket) {
  useEffect(() => {
    const interval = setInterval(() => {
      socket.send(JSON.stringify({ action: 'keep alive' }))
    }, 250)
    return () => {
      clearInterval(interval)
    }
  }, [socket])
}

type DeviceData = { id: number; [key: string]: any }
function deviceReducer(state: DeviceData[], action: any) {
  if (typeof action === 'string') {
    const message = JSON.parse(action)
    if (message.action === 'device info') {
      const incomingIds = new Set<number>(
        message.info.map((info: any) => info.id),
      )
      return state
        .filter((dev) => !incomingIds.has(dev.id))
        .concat(message.info)
    } else if (message.action === 'disconnect') {
      const incomingIds = new Set(message.ids)
      return state.filter((dev) => !incomingIds.has(dev.id))
    }
  } else {
    const binaryMessage = parseBinaryMessage(action)
    state = state.map((stateClient) => {
      const client = binaryMessage.clients.find(
        (c) => c.clientId === stateClient.id,
      )
      if (!client) return stateClient
      return {
        ...stateClient,
        info: stateClient.info?.map((stateInfo: any) => {
          const clientData = client.devices.find(
            (d) => d.deviceId === stateInfo.localId,
          )
          if (!clientData) return stateInfo
          return {
            ...stateInfo,
            data: clientData,
          }
        }),
      }
    })
    return state
  }

  return state
}

export function Dashboard({ socket }: { socket: PWebSocket }) {
  const [stopped, setStopped] = useState(false)
  const [devices, dispatchDevices] = useReducer(deviceReducer, [])

  const [log, dispatchLog] = useLog()
  useSendKeepAlive(socket)

  return (
    <>
      <ListenToSocket
        socket={socket}
        onMessage={(message) => {
          if (stopped) return
          dispatchLog(message)
          dispatchDevices(message)
        }}
      />
      <button type="button" onClick={() => setStopped((v) => !v)}>
        {stopped ? 'Resume' : 'Pause'}
      </button>
      <div
        style={{
          display: 'flex',
          flexDirection: 'row',
          flexWrap: 'wrap',
          justifyContent: 'space-between',
        }}
      >
        <div className="devices" style={{ width: 'auto' }}>
          {devices.map((device) => (
            <Device key={device.id} device={device} />
          ))}
        </div>
        <div className="events">
          {log.map((event) => (
            <Message
              message={event.message}
              key={event.key}
              timestamp={event.timestamp}
              type={event.type}
            />
          ))}
        </div>
      </div>
    </>
  )
}

function Device({ device }: { device: DeviceData }) {
  return (
    <code>
      <pre style={{ whiteSpace: 'pre-wrap', width: 500 }}>
        {JSON.stringify(
          device,
          (key, value) => {
            if (
              Array.isArray(value) &&
              value.length >= 2 &&
              value.length <= 3 &&
              typeof value[0] === 'number' &&
              !Number.isInteger(value[0])
            ) {
              return `(${value.map((v) => v.toFixed(2)).join(', ')})`
            }
            return value
          },
          2,
        )}
      </pre>
    </code>
  )
}

function Message({
  message,
  timestamp,
  type,
}: {
  message: any
  timestamp: string
  type: 'binary' | 'json'
}) {
  return (
    <div className="event">
      <div>🔽 {timestamp}</div>
      <pre>
        {type === 'binary' ? (
          <BinaryMessage data={message} />
        ) : (
          JSON.stringify(message, null, 2)
        )}
      </pre>
    </div>
  )
}

function BinaryMessage({ data }: { data: ArrayBuffer }) {
  const parsed = useMemo(() => parseBinaryMessage(data), [data])
  return (
    <>
      <div style={{ whiteSpace: 'pre-wrap', width: 500 }}>
        {'  ___count_____   __client_id__  #d _#bytes'}
      </div>
      <div style={{ whiteSpace: 'pre-wrap', width: 500 }}>
        {Array.from(new Uint8Array(data).values())
          .map(
            (v, i) => (v + '').padStart(3, ' ') + (i % 16 === 15 ? '\n' : ' '),
          )
          .join('')}{' '}
        ({data.byteLength})
      </div>
      <div style={{ whiteSpace: 'pre-wrap', width: 500 }}>
        {JSON.stringify(parsed, null, 2)}
      </div>
    </>
  )
}

function parseBinaryMessage(data: ArrayBuffer) {
  const view = new DataView(data, 0, data.byteLength)
  var offset = { current: 0 }
  const clientCount = readUint32(view, offset)
  let clients = []
  for (let clientIndex = 0; clientIndex < clientCount; ++clientIndex) {
    const clientId = readUint32(view, offset)
    const numberOfDevices = read7BitEncodedInt(view, offset)
    let devices = []
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
        quat: readArray(view, offsetCopy, readVec3),
        vec3: readArray(view, offsetCopy, readVec3),
        vec2: readArray(view, offsetCopy, readVec2),
        float: readArray(view, offsetCopy, readFloat),
        bool: readArray(view, offsetCopy, readBool),
        uint32: readArray(view, offsetCopy, readUint32),
      })
    }
    clients.push({ devices, clientId })
  }
  return { clientCount, clients }
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
