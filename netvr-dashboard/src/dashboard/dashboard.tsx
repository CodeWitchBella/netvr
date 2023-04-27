/** @jsxImportSource @emotion/react */
import { memo, useEffect, useState } from 'react'
import { Message, useLog } from './message-log'
import {
  ListenToSocket,
  SocketProvider,
  useSocket,
} from '../components/listen-to-socket'
import { ErrorBoundary } from '../components/error-boundary'
import { ConfigurationSnapshotSet } from '../protocol/data'
import { CalibrationPane } from './calibration-pane'
import { enableMapSet, enablePatches } from 'immer'

import { ThemeSelector } from '../components/theme'
import { JSONPane } from '../components/json-view'
import { Button, Pane } from '../components/design'
import { useLocalStorage } from '../utils'

import { QuickActionsPane } from './quick-actions-pane'
import { ClientPane } from './client-pane'
import { SendMessage } from '../protocol/sent-messages'
import { DashboardMessageDown } from '../protocol/recieved-messages'

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

  const [log, dispatchLog] = useLog({ showDatagrams: showDatagrams })
  useSendKeepAlive(socket)

  const sendMessage: SendMessage = function sendMessage(data: any) {
    if (data instanceof ArrayBuffer) {
      socket.send(data)
      return
    }
    const message = typeof data === 'string' ? data : JSON.stringify(data)
    dispatchLog({ direction: 'up', message, type: 'text', parsed: data })
    socket.send(message)
  }

  const [configurationSnapshot, setConfigurationSnapshot] =
    useState<ConfigurationSnapshotSet | null>(null)

  return (
    <ErrorBoundary>
      <ListenToSocket
        socket={socket}
        onMessage={(message) => {
          if (stopped || typeof message !== 'string') return

          const msg: DashboardMessageDown = JSON.parse(message)

          dispatchLog({
            direction: 'down',
            type: 'text',
            message,
            parsed: msg,
          })
          if (msg.type === 'ConfigurationSnapshotChanged') {
            setConfigurationSnapshot(msg.value)
          }
        }}
      />
      {configurationSnapshot === null ? null : (
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
              closeSocket={() => void socket.close()}
            />
            <ErrorBoundary>
              <CalibrationPane
                sendMessage={sendMessage}
                serverState={configurationSnapshot}
              />
            </ErrorBoundary>
            <StatePane data={configurationSnapshot} />

            {Object.entries(configurationSnapshot.clients).map(
              ([key, clientConfiguration]) => {
                const clientId = Number.parseInt(key, 10)

                return (
                  <ClientPane
                    key={clientId}
                    client={clientConfiguration}
                    clientId={clientId}
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
                  direction={event.direction}
                />
              ))}
            </Pane>
          </div>
        </div>
      )}
    </ErrorBoundary>
  )
}

const StatePane = memo<{ data: ConfigurationSnapshotSet }>(function StatePane({
  data,
}) {
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
