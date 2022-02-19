import { useEffect, useMemo, useReducer, useState } from 'react'
import { useLog } from './log'
import { ListenToSocket, SocketProvider, useSocket } from './listen-to-socket'
import type { PWebSocket } from './utils'
import { ErrorBoundary } from './error-boundary'
import {
  ClientData,
  DeviceData,
  mapData,
  parseBinaryMessage,
  protocolVersion,
} from './data'
import { SyncDevicesButton } from './sync-devices'
import { Calibration } from './calibration'
import { useImmer } from 'use-immer'
import { applyPatches, enableMapSet, enablePatches } from 'immer'

import { ThemeSelector, useTheme } from './use-theme'
import { JSONPane, JSONView } from './json-view'
import { Button, Pane } from './design'

enableMapSet()
enablePatches()

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
    if (binaryMessage.clients) {
      state = state.map((stateClient) => {
        const client = binaryMessage.clients.find(
          (c) => c.clientId === stateClient.id,
        )
        if (!client) return stateClient
        return {
          ...stateClient,
          info: stateClient.info?.map((stateInfo: any): DeviceData => {
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
    }
    return state
  }

  return state
}

export function Dashboard({ socketUrl }: { socketUrl: string }) {
  return (
    <SocketProvider url={socketUrl}>
      <DashboardInner />
    </SocketProvider>
  )
}

function DashboardInner() {
  const socket = useSocket()
  useEffect(() => {
    let sent = false
    try {
      const reconnection = localStorage.getItem('reconnection')
      const data = JSON.parse(reconnection || 'invalid')
      socket.send(
        JSON.stringify({
          action: 'i already has id',
          protocolVersion,
          ...data,
        }),
      )
      sent = true
    } catch {}

    if (!sent) {
      socket.send(JSON.stringify({ action: 'gimme id', protocolVersion }))
    }
  }, [socket])

  const [stopped, setStopped] = useState(false)
  const [showBinary, toggleShowBinary] = useReducer(
    (state: boolean) => !state,
    true,
  )
  const [clients, dispatchDevices] = useReducer(deviceReducer, [])
  const [serverState, setServerState] = useImmer({})

  const [log, dispatchLog] = useLog({ showBinary })
  useSendKeepAlive(socket)

  function sendMessage(data: any) {
    const message = JSON.stringify(data)
    dispatchLog({ direction: 'up', message })
    socket.send(message)
  }

  const theme = useTheme()
  return (
    <div style={{ flexGrow: 1, background: theme.resolved.base01 }}>
      <ErrorBoundary>
        <ListenToSocket
          socket={socket}
          onMessage={(message) => {
            if (stopped) return
            dispatchLog({ direction: 'down', message })
            dispatchDevices(message)
            if (typeof message === 'string') {
              const msg = JSON.parse(message)
              if (msg.action === "id's here") {
                localStorage.setItem(
                  'reconnection',
                  JSON.stringify({
                    id: msg.intValue,
                    token: msg.stringValue,
                  }),
                )
              } else if (msg.action === 'full state reset') {
                setServerState(
                  Object.fromEntries(
                    msg.clients.map((c: any) => [c.id, c.data]),
                  ),
                )
              } else if (msg.action === 'patch') {
                setServerState((draft) => {
                  applyPatches(draft, msg.patches)
                })
              }
            }
          }}
        />

        <div
          style={{
            display: 'flex',
            flexDirection: 'row',
            flexWrap: 'wrap',
            justifyContent: 'space-between',
          }}
        >
          <div className="clients" style={{ width: 'auto', flexGrow: 1 }}>
            <ThemeSelector />
            <Pane>
              <Button
                type="button"
                onClick={() => {
                  sendMessage({ action: 'reset room' })
                  setTimeout(() => {
                    window.location.reload()
                  }, 100)
                }}
              >
                Reset room
              </Button>
            </Pane>
            <ErrorBoundary>
              <SyncDevicesButton sendMessage={sendMessage} clients={clients} />
            </ErrorBoundary>
            <ErrorBoundary>
              <Calibration sendMessage={sendMessage} />
            </ErrorBoundary>
            <JSONPane
              name="state"
              data={serverState}
              shouldExpandNode={(keyPath, data, level) =>
                data.connected || level === 0
              }
            />
            {clients.map((client) => (
              <Client key={client.id} client={client} socket={socket} />
            ))}
          </div>
          <div style={{ flexGrow: 1 }}>
            <Pane>
              <div style={{ flexDirection: 'row', gap: 8, display: 'flex' }}>
                <Button type="button" onClick={toggleShowBinary}>
                  {showBinary ? 'Hide binary' : 'Show binary'}
                </Button>
                <Button type="button" onClick={() => setStopped((v) => !v)}>
                  {stopped ? 'Resume' : 'Pause'}
                </Button>
              </div>
            </Pane>
            {log.map((event) => (
              <Message
                message={event.message}
                key={event.key}
                timestamp={event.timestamp}
                type={event.type}
                direction={event.direction}
              />
            ))}
          </div>
        </div>
      </ErrorBoundary>
    </div>
  )
}

function Client({
  client,
  socket,
}: {
  client: ClientData
  socket: PWebSocket
}) {
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
      <Button type="button" onClick={toggleShowJson}>
        {showJson ? 'Hide JSON' : 'Show JSON'}
      </Button>
      {client.info?.map((data) => (
        <Device
          device={data}
          key={data.localId}
          clientId={client.id}
          socket={socket}
        />
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

function Device({
  device,
  socket,
  clientId,
}: {
  device: DeviceData
  socket: PWebSocket
  clientId: number
}) {
  const { data, unknown } = mapData(device)
  const [showDetails, toggleShowDetails] = useReducer(
    (prev: boolean) => !prev,
    false,
  )

  function identify() {
    const buffer = new ArrayBuffer(21)
    const view = new DataView(buffer)
    view.setUint8(0, 2) // message type
    view.setUint32(1, clientId, true)
    view.setUint32(5, device.localId, true)
    view.setUint32(9, 0, true) // channel
    view.setFloat32(13, 0.25, true) // amplitude
    view.setFloat32(17, 0.1, true) // time (oculus-only)
    socket.send(buffer)
  }
  return (
    <div
      style={{
        border: '1px solid gray',
        margin: 8,
        padding: 8,
        borderRadius: 4,
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between' }}>
        <div>Device: {device.localId}</div>
        <Button
          type="button"
          onClick={identify}
          disabled={!device.haptics?.supportsImpulse}
        >
          Identify
        </Button>
      </div>
      <div>Name: {device.name}</div>
      <div>Characteristics: {device.characteristics.join(', ')}</div>
      <Button type="button" onClick={toggleShowDetails}>
        {showDetails ? 'Hide details' : 'Show details'}
      </Button>
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
  direction,
}: {
  message: any
  timestamp: string
  type: 'binary' | 'json'
  direction: 'up' | 'down'
}) {
  return (
    <Pane>
      <div>
        {direction === 'down' ? '🔽' : '🔼'} {timestamp}
      </div>
      {type === 'binary' ? (
        <pre>
          <BinaryMessage data={message} />
        </pre>
      ) : (
        <JSONView name="message" data={message} shouldExpandNode={() => true} />
      )}
    </Pane>
  )
}

function BinaryMessage({ data }: { data: ArrayBuffer }) {
  const parsed = useMemo(() => parseBinaryMessage(data), [data])
  return (
    <>
      <div style={{ whiteSpace: 'pre-wrap', width: 500 }}>
        {'type  ___count_____   __client_id__  #d _#bytes'}
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
