/** @jsxImportSource @emotion/react */
import { memo, useEffect, useMemo, useReducer, useRef, useState } from 'react'
import { useLog } from './message-log'
import {
  ListenToSocket,
  SocketProvider,
  useSocket,
} from '../components/listen-to-socket'
import { ErrorBoundary } from '../components/error-boundary'
import {
  ClientBinaryData,
  ClientConfiguration,
  DeviceBinaryData,
  DeviceConfiguration,
  mapData,
  parseBinaryMessage,
  ServerState,
} from '../protocol/data'
import { SyncDevicesButton } from './use-sync-clients-by-headset'
import { TriggerCalibration } from './trigger-calibration'
import { useImmer } from 'use-immer'
import { applyPatches, enableMapSet, enablePatches } from 'immer'

import { ThemeSelector } from '../components/theme'
import { JSONPane, JSONView } from '../components/json-view'
import { Button, Pane } from '../components/design'
import { getName, useLocalStorage } from '../utils'
import * as sentMessages from '../protocol/sent-messages'
import {
  MessageTransmitLogs,
  RecievedMessage,
} from '../protocol/recieved-messages'
import { LogsModalDialog } from './logs-modal-dialog'
import { css } from '@emotion/react'

enableMapSet()
enablePatches()

function useSendKeepAlive(socket: WebSocket) {
  useEffect(() => {
    const interval = setInterval(() => {
      socket.send(JSON.stringify({ action: 'keep alive' }))
    }, 1000)
    return () => {
      clearInterval(interval)
    }
  }, [socket])
}

function deviceReducer(
  state: ClientBinaryData[],
  action:
    | { type: 'text'; message: any }
    | { type: 'binary'; message: ReturnType<typeof parseBinaryMessage> },
) {
  if (action.type !== 'binary') return state
  const binaryMessage = action.message
  if (binaryMessage.clients) {
    const nonExistingClients = binaryMessage.clients.filter(
      (c) => !state.some((s) => s.clientId === c.clientId),
    )
    const messageClients = new Map<number, typeof binaryMessage.clients[0]>()
    for (const c of binaryMessage.clients) messageClients.set(c.clientId, c)

    return state.concat(nonExistingClients).map((stateClient) => {
      const dataFromMessage = messageClients.get(stateClient.clientId)
      if (!dataFromMessage) return stateClient
      return dataFromMessage
    })
  }
  return state
}

export function Dashboard({ socketUrl }: { socketUrl: string }) {
  const [key, setKey] = useState(0)
  return (
    <div
      css={{
        flexGrow: 1,
        color: 'var(--base-5)',
      }}
    >
      <SocketProvider
        url={socketUrl}
        key={key}
        onDisconnected={() => {
          setTimeout(() => {
            setKey((v) => v + 1)
          }, 200)
        }}
      >
        <DashboardInner />
      </SocketProvider>
    </div>
  )
}

function clientLogsReducer(
  state: { [key: number]: MessageTransmitLogs['logs'] },
  action: [number, MessageTransmitLogs['logs']],
): { [key: number]: MessageTransmitLogs['logs'] } {
  return {
    ...state,
    [action[0]]: action[1],
  }
}

function DashboardInner() {
  const socket = useSocket()
  const [selfId, setSelfId] = useState(-1)
  useEffect(() => {
    let sent = false
    const deviceName = localStorage.getItem('deviceName')
    try {
      const reconnection = localStorage.getItem('reconnection')
      const data = JSON.parse(reconnection || 'invalid')
      socket.send(sentMessages.restoreConnectionFromBrowser(deviceName, data))
      console.log(data)
      setSelfId(data.id)
      sent = true
    } catch (e) {
      console.log(e)
    }

    if (!sent) {
      socket.send(sentMessages.establishNewConnectionFromBrowser(deviceName))
    }
  }, [socket])

  const [stopped, setStopped] = useState(false)
  const [showBinaryRaw, setShowBinary] = useLocalStorage(
    'show-binary',
    'true',
    (v): v is 'true' | 'false' => v === 'true' || v === 'false',
  )
  const showBinary = showBinaryRaw === 'true'
  const [clients, dispatchDevices] = useReducer(deviceReducer, [])
  const [serverState, setServerState] = useImmer<ServerState>({ clients: {} })

  const [log, dispatchLog] = useLog({ showBinary })
  const [clientLogs, dispatchClientLogs] = useReducer(
    clientLogsReducer,
    Object.create(null),
  )
  useSendKeepAlive(socket)

  function sendMessage(data: any) {
    if (data instanceof ArrayBuffer) {
      socket.send(data)
      return
    }
    const message = typeof data === 'string' ? data : JSON.stringify(data)
    dispatchLog({ direction: 'up', message, type: 'text', parsed: data })
    socket.send(message)
  }
  const lastAcceptedBinaryTimestamps = useRef(new Map<number, number>()).current

  const [logsModal, setLogsModal] = useState<{
    key: number
    logs: MessageTransmitLogs['logs']
  } | null>(null)

  const fullscreen = useFullscreen()
  return (
    <ErrorBoundary>
      {logsModal ? (
        <LogsModalDialog
          key={logsModal.key}
          logs={logsModal.logs}
          onClose={() => {
            setLogsModal((v) => (v?.key === logsModal.key ? null : v))
            document.documentElement.style.overflow = ''
          }}
        />
      ) : null}
      <ListenToSocket
        socket={socket}
        onMessage={(message) => {
          if (stopped) return
          if (typeof message !== 'string') {
            const parsed = parseBinaryMessage(message)
            const now = Date.now()
            const skippable = (client: { clientId: number }) => {
              const last = lastAcceptedBinaryTimestamps.get(client.clientId)
              return last && last + 100 > now
            }
            if (parsed.clients?.every(skippable)) {
              return
            }
            for (const client of parsed.clients || []) {
              lastAcceptedBinaryTimestamps.set(client.clientId, now)
            }
            dispatchLog({
              direction: 'down',
              type: 'binary',
              message,
              parsed,
            })
            dispatchDevices({ type: 'binary', message: parsed })
          } else {
            const msg: RecievedMessage = JSON.parse(message)
            if (msg.action === 'transmit logs') {
              setLogsModal((v) => ({ key: (v?.key ?? 0) + 1, logs: msg.logs }))
              dispatchClientLogs([msg.client, msg.logs])
              document.documentElement.scrollTo(0, 0)
              document.documentElement.style.overflow = 'hidden'
              return
            }
            dispatchLog({
              direction: 'down',
              type: 'text',
              message,
              parsed: msg,
            })
            dispatchDevices({ type: 'text', message: msg })
            if (msg.action === "id's here") {
              localStorage.setItem(
                'reconnection',
                JSON.stringify({
                  id: msg.intValue,
                  token: msg.stringValue,
                }),
              )
              setSelfId(msg.intValue)
            } else if (msg.action === 'full state reset') {
              setServerState(msg.state)
            } else if (msg.action === 'patch') {
              setServerState((draft) => {
                applyPatches(
                  draft,
                  msg.patches.map((p) => ({
                    ...p,
                    path: p.path.substring(1).split('/'),
                  })),
                )
              })
            }
          }
        }}
      />

      <div
        css={{
          display: 'flex',
          flexDirection: 'row',
          flexWrap: 'wrap',
          justifyContent: 'space-between',
        }}
      >
        <div css={{ flexBasis: 512, flexGrow: 1 }}>
          <ThemeSelector />
          <Pane title="Quick actions" id="quick-actions">
            <div css={{ display: 'flex', gap: 6 }}>
              <Button
                type="button"
                onClick={() => {
                  sendMessage(sentMessages.resetRoom())
                }}
              >
                Reset room
              </Button>
              <SyncDevicesButton
                sendMessage={sendMessage}
                clients={clients}
                serverState={serverState}
              />
            </div>
            <form
              onSubmit={(evt) => {
                evt.preventDefault()
                const name: string = new FormData(evt.currentTarget).get(
                  'name',
                ) as any
                if (name) localStorage.setItem('deviceName', name)
                else localStorage.removeItem('deviceName')
                socket.close()
              }}
            >
              <label>
                deviceName:{' '}
                <input
                  defaultValue={localStorage.getItem('deviceName') ?? ''}
                  name="name"
                />
              </label>
              <button>set</button>
            </form>
          </Pane>

          <ErrorBoundary>
            <TriggerCalibration
              sendMessage={sendMessage}
              serverState={serverState}
            />
          </ErrorBoundary>
          <StatePane data={serverState} />

          {Object.entries(serverState.clients).map(
            ([key, clientConfiguration]) => {
              const clientId = Number.parseInt(key, 10)
              const binaryClient = clients.find((c) => c.clientId === clientId)
              if (!clientConfiguration.connected) return null
              return (
                <Client
                  key={clientId}
                  client={clientConfiguration}
                  binaryClient={binaryClient ?? { clientId }}
                  socket={socket}
                  selfId={selfId}
                  logs={clientLogs[clientId]}
                />
              )
            },
          )}
        </div>
        <div css={{ flexBasis: 512, flexGrow: 1 }}>
          <Pane>
            <div css={{ flexDirection: 'row', gap: 8, display: 'flex' }}>
              <Button
                type="button"
                onClick={() => setShowBinary(showBinary ? 'false' : 'true')}
              >
                {showBinary ? 'Hide binary' : 'Show binary'}
              </Button>
              <Button type="button" onClick={() => setStopped((v) => !v)}>
                {stopped ? 'Resume' : 'Pause'}
              </Button>
              <div css={{ flexGrow: 1 }} />
              <Button
                type="button"
                onClick={() => {
                  if (fullscreen.enabled) fullscreen.exit()
                  else fullscreen.request()
                }}
              >
                {fullscreen.enabled ? 'Exit fullscreen' : 'Fullscreen'}
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
  )
}

const StatePane = memo<{ data: ServerState }>(function StatePane({ data }) {
  return (
    <JSONPane
      title="State"
      id="state"
      name="state"
      data={data}
      shouldExpandNode={(keyPath, data, level) =>
        data.connected || level <= 1 || keyPath[0] === 'connectionInfo'
      }
    />
  )
})

function Client({
  binaryClient,
  client,
  socket,
  selfId,
  logs,
}: {
  binaryClient: ClientBinaryData | { clientId: number; devices?: undefined }
  client: ClientConfiguration
  socket: WebSocket
  selfId: number
  logs?: MessageTransmitLogs['logs']
}) {
  const [show, setShow] = useState<'none' | 'json' | 'logs'>('none')
  return (
    <Pane
      id={'client-' + binaryClient.clientId}
      title={`Client ${getName(binaryClient, client.connectionInfo)}`}
    >
      <div
        css={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'flex-start',
        }}
      >
        <div>
          <div>
            id: {binaryClient.clientId}
            {selfId === binaryClient.clientId ? ' (this browser)' : null}
          </div>
          <div>ip: {client.connectionInfo.ip}</div>
          <div>connected: {client.connected ? '✅' : '❌'}</div>
        </div>
        {selfId === binaryClient.clientId ||
        client.connectionInfo.isBrowser ? null : (
          <div css={{ display: 'flex', gap: 6 }}>
            <Button
              type="button"
              onClick={() => {
                socket.send(sentMessages.requestLogs(binaryClient.clientId))
              }}
            >
              Request logs
            </Button>
            <Button
              type="button"
              onClick={() => {
                socket.send(sentMessages.quit(binaryClient.clientId))
              }}
            >
              Quit
            </Button>
          </div>
        )}
      </div>

      {client.devices?.map((info) => {
        const data = binaryClient.devices?.find(
          (d) => d.deviceId === info.localId,
        )
        return (
          <Device
            device={data ?? null}
            configuration={info}
            key={info.localId}
            clientId={binaryClient.clientId}
            socket={socket}
          />
        )
      }) ?? null}
      <div css={{ display: 'flex', gap: 6 }}>
        <Button
          type="button"
          onClick={() => void setShow(show === 'json' ? 'none' : 'json')}
        >
          {show === 'json' ? 'Hide JSON' : 'Show JSON'}
        </Button>
        {logs ? (
          <Button
            type="button"
            onClick={() => void setShow(show === 'logs' ? 'none' : 'logs')}
          >
            {show === 'logs' ? 'Hide logs' : 'Show logs'}
          </Button>
        ) : null}
      </div>
      {show === 'none' ? null : (
        <code
          css={{
            position: 'relative',
            width: '100%',
            height: 500,
          }}
        >
          <pre
            css={[
              show === 'json' ? css({ whiteSpace: 'pre-wrap' }) : {},
              {
                overflow: 'auto',
                position: 'absolute',
                left: 0,
                right: 0,
                top: 0,
                bottom: 0,
              },
            ]}
          >
            {show === 'logs'
              ? logs?.map((v) => v.text).join('\n') || ''
              : JSON.stringify(
                  binaryClient,
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
      )}
    </Pane>
  )
}

function Device({
  device,
  configuration,
  socket,
  clientId,
}: {
  device: DeviceBinaryData | null
  configuration: DeviceConfiguration
  socket: WebSocket
  clientId: number
}) {
  const [showDetails, toggleShowDetails] = useReducer(
    (prev: boolean) => !prev,
    false,
  )

  return (
    <div
      css={{
        border: '1px solid gray',
        margin: 8,
        padding: 8,
        borderRadius: 4,
      }}
    >
      <div css={{ display: 'flex', justifyContent: 'space-between' }}>
        <div>Device: {configuration.localId}</div>
        {device ? (
          <Button
            type="button"
            onClick={() =>
              void socket.send(
                sentMessages.hapticImpulse({
                  clientId,
                  deviceId: device.deviceId,
                }),
              )
            }
            disabled={!configuration.haptics?.supportsImpulse}
          >
            Identify
          </Button>
        ) : null}
      </div>
      <div>Name: {configuration.name}</div>
      <div>Characteristics: {configuration.characteristics.join(', ')}</div>
      {device ? (
        <>
          <Button type="button" onClick={toggleShowDetails}>
            {showDetails ? 'Hide details' : 'Show details'}
          </Button>
          {showDetails ? (
            <DeviceDetails device={device} configuration={configuration} />
          ) : null}
        </>
      ) : null}
    </div>
  )
}

function DeviceDetails({
  device,
  configuration,
}: {
  device: DeviceBinaryData
  configuration: DeviceConfiguration
}) {
  const { data, unknown } = mapData(device, configuration)
  return (
    <>
      {Object.entries(data).map(([key, o]) => (
        <div key={key}>
          {key}:{' '}
          {o.type === 'quaternion' || o.type === 'vector3'
            ? `${o.value.x.toFixed(2)}, ${o.value.y.toFixed(
                2,
              )}, ${o.value.z.toFixed(2)}`
            : o.type === 'vector2'
            ? `${o.value.x.toFixed(2)}, ${o.value.y.toFixed(2)} `
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
  )
}

const Message = memo(function Message({
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
          <BinaryMessage raw={message.raw} parsed={message.parsed} />
        </pre>
      ) : (
        <JSONView name="message" data={message} shouldExpandNode={() => true} />
      )}
    </Pane>
  )
})

function BinaryMessage({ raw, parsed }: { raw: ArrayBuffer; parsed: any }) {
  const rawText = useMemo(
    () =>
      Array.from(new Uint8Array(raw).values())
        .map((v, i) => (v + '').padStart(3, ' ') + (i % 16 === 15 ? '\n' : ' '))
        .join(''),
    [raw],
  )
  return (
    <>
      <div css={{ whiteSpace: 'pre-wrap', width: 500 }}>
        {'type  ___count_____   __client_id__  #d _#bytes'}
      </div>
      <div css={{ whiteSpace: 'pre-wrap', width: 500 }}>
        {rawText} ({raw.byteLength})
      </div>
      <div css={{ whiteSpace: 'pre-wrap', width: 500 }}>
        <JSONView data={parsed} shouldExpandNode={() => false} />
      </div>
    </>
  )
}

function useFullscreen() {
  const [enabled, setEnabled] = useState(!!document.fullscreenElement)
  useEffect(() => {
    document.addEventListener('fullscreenchange', handler)
    return () => void document.removeEventListener('fullscreenchange', handler)
    function handler() {
      setEnabled(!!document.fullscreenElement)
    }
  })

  return {
    enabled,
    exit: () => {
      document.exitFullscreen()
    },
    request: () => {
      document.documentElement.requestFullscreen({ navigationUI: 'hide' })
    },
  }
}
