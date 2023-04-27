/** @jsxImportSource @emotion/react */
import { memo, useEffect, useMemo, useReducer, useState } from 'react'
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
import { DashboardMessageDown, DatagramUp } from '../protocol/recieved-messages'
import { DatagramState, mergeData } from './merge-data'

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

function datagramReducer(
  state: DatagramState,
  action: { now: number; datagram: DatagramUp },
): DatagramState {
  const {
    now,
    datagram: { id, message },
  } = action
  const res: DatagramState = {
    ...state,
    [id]: {
      id,
      time: now,
      snapshot: message,
    },
  }
  for (const [key, value] of Object.entries(res)) {
    if (value.time < now - 1000) {
      delete res[key as any]
    }
  }
  return res
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
  const [datagramData, dispatchDatagram] = useReducer(datagramReducer, {})
  const mergedData = useMemo(
    () => mergeData(datagramData, configurationSnapshot),
    [datagramData, configurationSnapshot],
  )

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
          if (msg.type === 'DatagramUp') {
            dispatchDatagram({ now: Date.now(), datagram: msg })
          } else if (msg.type === 'ConfigurationSnapshotChanged') {
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
                mergedData={mergedData}
              />
            </ErrorBoundary>
            <StatePane data={mergedData} />

            {Object.entries(configurationSnapshot.clients).map(
              ([key, clientConfiguration]) => {
                const clientId = Number.parseInt(key, 10)

                return (
                  <ClientPane
                    key={clientId}
                    client={clientConfiguration}
                    clientId={clientId}
                    sendMessage={sendMessage}
                    stateSnapshot={datagramData[clientId]?.snapshot ?? null}
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

const StatePane = memo<{ data: any }>(function StatePane({ data }) {
  return (
    <JSONPane
      title="State"
      id="state"
      name="state"
      data={data}
      shouldExpandNode={(keyPath, data, level) => {
        if (keyPath[0] === 'pose')
          return (
            data.position.x !== 0 &&
            data.position.y !== 0 &&
            data.position.z !== 0
          )
        return level <= 5 && keyPath[0] !== 'configuration'
      }}
    />
  )
})
