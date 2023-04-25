/** @jsxImportSource @emotion/react */
import { memo, useEffect, useReducer, useRef, useState } from 'react'
import { Message, useLog } from './message-log'
import {
  ListenToSocket,
  SocketProvider,
  useSocket,
} from '../components/listen-to-socket'
import { ErrorBoundary } from '../components/error-boundary'
import {
  ClientBinaryData,
  parseBinaryMessage,
  ServerState,
} from '../protocol/data'
import { CalibrationPane } from './calibration-pane'
import { useImmer } from 'use-immer'
import { applyPatches, enableMapSet, enablePatches } from 'immer'

import { ThemeSelector } from '../components/theme'
import { JSONPane } from '../components/json-view'
import { Button, Pane } from '../components/design'
import { useLocalStorage } from '../utils'
import {
  MessageTransmitLogs,
  RecievedMessage,
} from '../protocol/recieved-messages'
import { LogsModalDialog } from './logs-modal-dialog'
import { QuickActionsPane } from './quick-actions-pane'
import { ClientPane } from './client-pane'

enableMapSet()
enablePatches()

function useSendKeepAlive(socket: WebSocket) {
  useEffect(() => {
    const interval = setInterval(() => {
      socket.send(JSON.stringify({ type: 'KeepAlive' }))
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

function DashboardInner() {
  const socket = useSocket()
  useEffect(() => {
    socket.send(JSON.stringify({ type: 'Init' }))
  }, [socket])

  const [stopped, setStopped] = useState(false)
  const [showBinaryRaw, setShowDatagrams] = useLocalStorage(
    'show-binary',
    'true',
    (v): v is 'true' | 'false' => v === 'true' || v === 'false',
  )
  const showDatagrams = showBinaryRaw === 'true'
  const [clients, dispatchDevices] = useReducer(deviceReducer, [])
  const [serverState, setServerState] = useImmer<ServerState>({ clients: {} })

  const [log, dispatchLog] = useLog({ showDatagrams: showDatagrams })
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
            if (msg.action === 'full state reset') {
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
          <QuickActionsPane
            sendMessage={sendMessage}
            serverState={serverState}
            clients={clients}
            closeSocket={() => void socket.close()}
          />
          <ErrorBoundary>
            <CalibrationPane
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
                <ClientPane
                  key={clientId}
                  client={clientConfiguration}
                  binaryClient={binaryClient ?? { clientId }}
                  sendMessage={sendMessage}
                />
              )
            },
          )}
        </div>
        <div css={{ flexBasis: 512, flexGrow: 1 }}>
          <Pane title="Local message log" id="messages">
            <div css={{ flexDirection: 'row', gap: 8, display: 'flex' }}>
              <Button
                type="button"
                onClick={() =>
                  setShowDatagrams(showDatagrams ? 'false' : 'true')
                }
              >
                {showDatagrams ? 'Hide datagrams' : 'Show datagrams'}
              </Button>
              <Button type="button" onClick={() => setStopped((v) => !v)}>
                {stopped ? 'Resume' : 'Pause'}
              </Button>
            </div>
            {log.map((event) => (
              <Message
                message={event.message}
                key={event.key}
                timestamp={event.timestamp}
                type={event.type}
                direction={event.direction}
              />
            ))}
          </Pane>
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
