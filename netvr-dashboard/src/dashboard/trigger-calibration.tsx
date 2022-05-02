/** @jsxImportSource @emotion/react */
import { useState } from 'react'
import type { ServerState } from '../protocol/data'
import { Button, Pane, Select } from '../components/design'
import { getName } from '../utils'
import * as sentMessages from '../protocol/sent-messages'

export function TriggerCalibration({
  sendMessage,
  serverState,
}: {
  sendMessage: (message: any) => void
  serverState: ServerState
}) {
  const [message, setMessage] = useState('')
  const [client1Id, setClient1Id] = useState(0)
  return (
    <Pane title="Calibration" id="calibration">
      <form
        onSubmit={(evt) => {
          evt.preventDefault()

          const formData = new FormData(evt.currentTarget)
          const data: {
            leader: number
            follower: number
            leaderDevice: number
            followerDevice: number
          } = Object.fromEntries(
            Array.from(formData.entries(), ([k, v]) => [k, +v]),
          ) as any
          sendMessage(sentMessages.beginCalibration(data))

          const msg = 'Calibration triggered ' + JSON.stringify(data)
          setMessage(msg)
          setTimeout(() => {
            setMessage((prev) => (prev === msg ? '' : prev))
          }, 750)
        }}
        css={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'flex-start',
          gap: 8,
        }}
      >
        <DeviceSelect
          serverState={serverState}
          onClientSelect={setClient1Id}
          type="leader"
          sendMessage={sendMessage}
        />
        {true ? (
          <DeviceSelect
            serverState={serverState}
            exceptClient={client1Id}
            type="follower"
            sendMessage={sendMessage}
          />
        ) : null}
        <Button type="submit">Trigger Calibration</Button>
        {message}
      </form>
    </Pane>
  )
}

function DeviceSelect({
  serverState,
  onClientSelect,
  exceptClient,
  type,
  sendMessage,
}: {
  serverState: ServerState
  onClientSelect?: (id: number) => void
  exceptClient?: number
  type: 'leader' | 'follower'
  sendMessage: (data: ArrayBuffer) => void
}) {
  const [clientId, setClientId] = useState(0)
  const [deviceId, setDeviceId] = useState(0)
  return (
    <div
      css={{
        display: 'flex',
        flexDirection: 'column',
        paddingInlineStart: 8,
        gap: 4,
      }}
    >
      <div css={{ marginInlineStart: -4, fontWeight: 'bold' }}>{type}</div>
      <label>
        Client
        <Select
          css={{ marginInlineStart: 4 }}
          required
          name={type}
          autoComplete="off"
          onChange={(evt) => {
            const value = +evt.currentTarget.value ?? 0
            setClientId(value)
            onClientSelect?.(value)
          }}
        >
          <option value="">Select client</option>
          {Object.entries(serverState.clients)
            .filter(
              ([id, client]) =>
                client.connected && client.devices && +id !== exceptClient,
            )
            .map(([id, client]) => (
              <option key={id} value={id}>
                {getName({ clientId: id }, client.connectionInfo)}
              </option>
            ))}
        </Select>
      </label>
      <div>
        <label>
          Device
          <Select
            css={{ marginInline: 4 }}
            required
            name={type + 'Device'}
            autoComplete="off"
            disabled={
              !(
                clientId > 0 &&
                clientId !== exceptClient &&
                serverState.clients[clientId]
              )
            }
            onChange={(evt) => {
              setDeviceId(+evt.currentTarget.value ?? 0)
            }}
          >
            <option value="">Select device</option>
            {serverState.clients[clientId]?.devices?.map((device) => (
              <option key={device.localId} value={device.localId}>
                #{device.localId}: {device.name}{' '}
                {device.characteristics.join(',')}
              </option>
            ))}
          </Select>
        </label>
        <Button
          type="button"
          onClick={() =>
            void sendMessage(sentMessages.hapticImpulse({ clientId, deviceId }))
          }
          disabled={
            !serverState.clients[clientId]?.devices?.find(
              (d) => d.localId === deviceId,
            )?.haptics?.supportsImpulse
          }
        >
          Identify
        </Button>
      </div>
    </div>
  )
}