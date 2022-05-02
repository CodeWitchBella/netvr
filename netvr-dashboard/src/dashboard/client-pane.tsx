/** @jsxImportSource @emotion/react */
import { css } from '@emotion/react'
import { useReducer, useState } from 'react'
import { Pane, Button } from '../components/design'
import {
  ClientBinaryData,
  ClientConfiguration,
  DeviceBinaryData,
  DeviceConfiguration,
  mapData,
} from '../protocol/data'
import { MessageTransmitLogs } from '../protocol/recieved-messages'
import * as sentMessages from '../protocol/sent-messages'
import { getName } from '../utils'

export function ClientPane({
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
