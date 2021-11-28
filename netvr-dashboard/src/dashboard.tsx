import { useEffect, useMemo, useReducer, useRef, useState } from 'react'
import { useLog } from './log'
import { ListenToSocket } from './listen-to-socket'
import type { PWebSocket } from './utils'
import { ErrorBoundary } from './error-boundary'
import { ClientData, DeviceData, mapData, parseBinaryMessage } from './data'
import { SyncDevicesButton } from './sync-devices'

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

function deviceReducer(state: ClientData[], action: any) {
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
  const [showBinary, toggleShowBinary] = useReducer(
    (state: boolean) => !state,
    true,
  )
  const [clients, dispatchDevices] = useReducer(deviceReducer, [])

  const [log, dispatchLog] = useLog({ showBinary })
  useSendKeepAlive(socket)

  function sendMessage(data: any) {
    const message = JSON.stringify(data)
    dispatchLog({ direction: 'up', message })
    socket.send(message)
  }

  return (
    <ErrorBoundary>
      <ListenToSocket
        socket={socket}
        onMessage={(message) => {
          if (stopped) return
          dispatchLog({ direction: 'down', message })
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
        <div className="clients" style={{ width: 'auto' }}>
          <ErrorBoundary>
            <SyncDevicesButton sendMessage={sendMessage} clients={clients} />
          </ErrorBoundary>
          {clients.map((client) => (
            <Client key={client.id} client={client} />
          ))}
        </div>
        <div className="events">
          <button type="button" onClick={toggleShowBinary}>
            {showBinary ? 'Hide binary' : 'Show binary'}
          </button>
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
    </ErrorBoundary>
  )
}

function Client({ client }: { client: ClientData }) {
  const [showJson, toggleShowJson] = useReducer((prev: boolean) => !prev, false)
  return (
    <div
      className="device"
      style={{
        border: '1px solid gray',
        margin: 8,
        padding: 8,
        borderRadius: 4,
      }}
    >
      <div>id: {client.id}</div>
      <button type="button" onClick={toggleShowJson}>
        {showJson ? 'Hide JSON' : 'Show JSON'}
      </button>
      {client.info?.map((data) => (
        <Device device={data} key={data.localId} />
      )) ?? null}
      {showJson ? (
        <code>
          <pre style={{ whiteSpace: 'pre-wrap', width: 500 }}>
            {JSON.stringify(
              client,
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
      ) : null}
    </div>
  )
}

function Device({ device }: { device: DeviceData }) {
  const { data, unknown } = mapData(device)
  const [showDetails, toggleShowDetails] = useReducer(
    (prev: boolean) => !prev,
    false,
  )
  return (
    <div
      style={{
        border: '1px solid gray',
        margin: 8,
        padding: 8,
        borderRadius: 4,
      }}
    >
      <div>Device: {device.localId} </div>
      <div>Name: {device.name}</div>
      <div>Characteristics: {device.characteristics.join(', ')}</div>
      <button type="button" onClick={toggleShowDetails}>
        {showDetails ? 'Hide details' : 'Show details'}
      </button>
      {showDetails ? (
        <>
          {Object.entries(data).map(([key, o]) => (
            <div key={key}>
              {key}:{' '}
              {o.type === 'quaternion' ||
              o.type === 'vector3' ||
              o.type === 'vector2'
                ? o.value.map((v) => v.toFixed(2)).join(', ')
                : o.type === 'float'
                ? o.value.toFixed(2)
                : o.type === 'uint32'
                ? o.value.toFixed(0)
                : o.type === 'bool'
                ? o.value === 'fail'
                  ? 'fail'
                  : o.value
                  ? '✅'
                  : '❌'
                : null}
            </div>
          ))}
          <div>
            <b>Unkown:</b>
            {unknown ? <pre>{JSON.stringify(unknown, null, 2)}</pre> : ' none'}
          </div>
        </>
      ) : null}
    </div>
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
