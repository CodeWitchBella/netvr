/** @jsxImportSource @emotion/react */
import React, { useReducer } from 'react'
import { Pane, Button } from '../components/design'
import {
  ClientBinaryData,
  ClientConfiguration,
  DeviceBinaryData,
  DeviceConfiguration,
  mapData,
} from '../protocol/data'
import * as sentMessages from '../protocol/sent-messages'
import { getName } from '../utils'

export function ClientPane({
  binaryClient,
  client,
  sendMessage,
}: {
  binaryClient: ClientBinaryData | { clientId: number; devices?: undefined }
  client: ClientConfiguration
  sendMessage: sentMessages.SendMessage
}) {
  function resetCalibration() {
    sendMessage(
      sentMessages.setCalibration([
        {
          client: binaryClient.clientId,
          value: {
            rotate: { x: 0, y: 0, z: 0 },
            translate: { x: 0, y: 0, z: 0 },
            scale: { x: 1, y: 1, z: 1 },
          },
        },
      ]),
    )
  }

  return (
    <Pane
      id={'client-' + binaryClient.clientId}
      title={`Client ${getName(binaryClient, client.connectionInfo)}`}
    >
      <div
        css={{
          display: 'flex',
          flexDirection: 'row-reverse',
          flexWrap: 'wrap',
          gap: 8,
        }}
      >
        <div>
          {client.connectionInfo.isBrowser ? null : (
            <div css={{ display: 'flex', gap: 6 }}>
              <Button
                type="button"
                onClick={() => {
                  sendMessage(sentMessages.requestLogs(binaryClient.clientId))
                }}
              >
                Request logs
              </Button>
              <Button
                type="button"
                onClick={() => {
                  sendMessage(sentMessages.quit(binaryClient.clientId))
                }}
              >
                Quit
              </Button>
            </div>
          )}
        </div>
        <div
          css={{
            flexGrow: 1,
            display: 'flex',
            flexDirection: 'column',
            gap: 4,
          }}
        >
          <ClientContent>ip: {client.connectionInfo.ip}</ClientContent>

          {isDefaultCalibration(client.calibration) ? null : (
            <ClientContent
              css={{
                flexDirection: 'row',
                alignItems: 'center',
              }}
            >
              <div>
                <div>
                  Translate: {vectorToString(client.calibration.translate, 'm')}
                </div>
                <div>
                  Rotate:{' '}
                  {vectorToString(
                    client.calibration.rotate,
                    '°',
                    180 / Math.PI,
                  )}
                </div>
              </div>
              <div css={{ display: 'flex', alignItems: 'flex-end' }}>
                <Button type="button" onClick={resetCalibration}>
                  Reset
                </Button>
              </div>
            </ClientContent>
          )}
        </div>
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
            sendMessage={sendMessage}
          />
        )
      }) ?? null}
    </Pane>
  )
}

function ClientContent({
  children,
  className,
}: {
  children: React.ReactNode
  className?: string
}) {
  return (
    <div
      css={{
        display: 'flex',
        flexDirection: 'column',
        gap: 4,
        paddingBlock: 4,
        paddingInline: 8,
        borderRadius: 4,
      }}
      className={className}
    >
      {children}
    </div>
  )
}

function isDefaultCalibration(calibration: ClientConfiguration['calibration']) {
  return (
    calibration.translate.x === 0 &&
    calibration.translate.y === 0 &&
    calibration.translate.z === 0 &&
    calibration.rotate.x === 0 &&
    calibration.rotate.y === 0 &&
    calibration.rotate.z === 0 &&
    calibration.scale.x === 1 &&
    calibration.scale.y === 1 &&
    calibration.scale.z === 1
  )
}

function vectorToString(
  vector: { x: number; y: number; z: number },
  unit: string = '',
  multiply: number = 1,
) {
  return `(${(vector.x * multiply).toFixed(2)}${unit}, ${(
    vector.y * multiply
  ).toFixed(2)}${unit}, ${(vector.z * multiply).toFixed(2)}${unit})`
}

function Device({
  device,
  configuration,
  sendMessage,
  clientId,
}: {
  device: DeviceBinaryData | null
  configuration: DeviceConfiguration
  sendMessage: sentMessages.SendMessage
  clientId: number
}) {
  const [showDetails, toggleShowDetails] = useReducer(
    (prev: boolean) => !prev,
    false,
  )

  return (
    <ClientContent
      css={{
        border: '1px solid var(--base-2)',
        paddingBlockStart: 8,
        paddingBlockEnd: 8,
      }}
    >
      <div css={{ display: 'flex', justifyContent: 'space-between' }}>
        <div>Device: {configuration.localId}</div>
        {device ? (
          <Button
            type="button"
            onClick={() => {
              sendMessage(
                sentMessages.hapticImpulse({
                  clientId,
                  deviceId: device.deviceId,
                }),
              )
            }}
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
    </ClientContent>
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
