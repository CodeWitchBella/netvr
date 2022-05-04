/** @jsxImportSource @emotion/react */
import { useState } from 'react'
import type { ClientConfiguration, ServerState } from '../protocol/data'
import { Button, Pane, Select } from '../components/design'
import { getName, useLocalStorage } from '../utils'
import * as sentMessages from '../protocol/sent-messages'

function isTrueOrFalseString(v: string): v is 'true' | 'false' {
  return v === 'true' || v === 'false'
}

export function CalibrationPane({
  sendMessage,
  serverState,
}: {
  sendMessage: (message: any) => void
  serverState: ServerState
}) {
  const [selfCalRaw, setSelfCal] = useLocalStorage(
    'self-calibration',
    'false',
    isTrueOrFalseString,
  )
  const selfCal = selfCalRaw === 'true'
  const client1 = useDeviceSelect({ serverState })
  const client2 = useDeviceSelect({
    serverState,
    exceptClient: selfCal ? undefined : client1.clientId,
  })

  const [message, setMessage] = useState('')
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
        <label title="Only useful for debugging. Allows selecting same client twice.">
          <input
            type="checkbox"
            name="self-calibration"
            checked={selfCal}
            onChange={(evt) =>
              void setSelfCal(evt.currentTarget.checked ? 'true' : 'false')
            }
          />{' '}
          Allow self-calibration
        </label>
        <DeviceSelect
          serverState={serverState}
          data={client1}
          type="leader"
          sendMessage={sendMessage}
        />
        {true ? (
          <DeviceSelect
            serverState={serverState}
            data={client2}
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

function isSelectableClient(
  clientId: number | string,
  client: ClientConfiguration | null,
  exceptClient: number = 0,
) {
  return !!(
    client &&
    +clientId !== exceptClient &&
    client.connected &&
    !client.connectionInfo.isBrowser &&
    client.devices &&
    client.devices.length > 0
  )
}

function useDeviceSelect({
  exceptClient,
  serverState,
}: {
  exceptClient?: number
  serverState: ServerState
}) {
  const [clientIdState, setClientId] = useState(0)
  const [deviceIdState, setDeviceId] = useState(0)

  const [clientId, client] = getClient()
  const devices = client?.devices?.filter((v) =>
    v.characteristics.includes('TrackedDevice'),
  )

  return {
    clientId,
    client,
    setClientId,
    deviceId: getDeviceId(),
    setDeviceId,
    exceptClient,
    devices,
  }

  function getClient(): readonly [number, ClientConfiguration | null] {
    if (
      isSelectableClient(
        clientIdState,
        serverState.clients[clientIdState],
        exceptClient,
      )
    ) {
      return [clientIdState, serverState.clients[clientIdState]]
    }

    const found = Object.entries(serverState.clients).find(([k, client]) =>
      isSelectableClient(k, client, exceptClient),
    )
    if (!found) return [0, null]
    return [+found[0], found[1]]
  }

  function getDeviceId() {
    if (!devices) return deviceIdState
    let device = devices.find((d) => d.localId === deviceIdState)
    if (device) return device.localId
    device = devices.find((d) => d.characteristics.includes('HeldInHand'))
    if (device) return device.localId
    return deviceIdState
  }
}

function DeviceSelect({
  serverState,
  type,
  sendMessage,
  data,
}: {
  serverState: ServerState
  type: 'leader' | 'follower'
  sendMessage: (data: ArrayBuffer) => void
  data: ReturnType<typeof useDeviceSelect>
}) {
  const clients = Object.entries(serverState.clients).filter(
    ([id, client]) =>
      client.connected && client.devices && +id !== data.exceptClient,
  )
  const deviceSelectorDisabled =
    !data.client || !data.devices || data.devices.length < 1
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
          disabled={clients.length < 1}
          value={data.clientId}
          onChange={(evt) => {
            const value = +evt.currentTarget.value ?? 0
            data.setClientId(value)
          }}
        >
          {clients.length < 1 ? (
            <option value="">No client available</option>
          ) : null}
          {clients.map(([id, client]) => (
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
            value={data.deviceId}
            disabled={deviceSelectorDisabled}
            onChange={(evt) => {
              data.setDeviceId(+evt.currentTarget.value ?? 0)
            }}
          >
            {deviceSelectorDisabled ? (
              <option value="">Select client first</option>
            ) : null}
            {data.devices?.map((device) => (
              <option key={device.localId} value={device.localId}>
                #{device.localId}: {device.name.replace(' OpenXR', '')}
                {'\n'}
                {device.characteristics.join(',')}
              </option>
            ))}
          </Select>
        </label>
        <Button
          type="button"
          onClick={() => {
            sendMessage(
              sentMessages.hapticImpulse({
                clientId: data.clientId,
                deviceId: data.deviceId,
              }),
            )
          }}
          disabled={
            !data.client?.devices?.find((d) => d.localId === data.deviceId)
              ?.haptics?.supportsImpulse
          }
        >
          Identify
        </Button>
      </div>
    </div>
  )
}
