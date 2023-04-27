/** @jsxImportSource @emotion/react */
import { useState } from 'react'
import type {
  RemoteConfigurationSnapshot,
  ConfigurationSnapshotSet,
} from '../protocol/data'
import { Button, Pane, Select } from '../components/design'
import { useLocalStorage } from '../utils'
import * as sentMessages from '../protocol/sent-messages'

function isTrueOrFalseString(v: string): v is 'true' | 'false' {
  return v === 'true' || v === 'false'
}

export function CalibrationPane({
  sendMessage,
  serverState,
}: {
  sendMessage: sentMessages.SendMessage
  serverState: ConfigurationSnapshotSet
}) {
  const client1 = useDeviceSelect({ serverState })
  const client2 = useDeviceSelect({
    serverState,
    exceptClient: client1.clientId,
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
            leaderDevice: string
            followerDevice: string
          } = Object.fromEntries(
            Array.from(formData.entries(), ([k, v]) => [k, +v]),
          ) as any
          const str = (v: unknown) => {
            if (typeof v !== 'string') throw new Error('Missing value')
            return v
          }
          const num = (v: unknown) => +str(v)
          sendMessage({
            type: 'StartCalibration',
            leaderId: num(formData.get('leaderId')),
            leaderSubactionPath: str(formData.get('leaderSubactionPath')),
            followerId: num(formData.get('followerId')),
            followerSubactionPath: str(formData.get('followerSubactionPath')),
            sampleCount: 500,
          })

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
  client: RemoteConfigurationSnapshot | null,
  exceptClient: number = 0,
) {
  return !!(
    client &&
    +clientId !== exceptClient &&
    client.user_paths &&
    client.user_paths.length > 0
  )
}

function useDeviceSelect({
  exceptClient,
  serverState,
}: {
  exceptClient?: number
  serverState: ConfigurationSnapshotSet
}) {
  const [clientIdState, setClientId] = useState(0)
  const [subactionPathState, setSubactionPath] = useState('')

  const [clientId, client] = getClient()
  const subactionPaths = client?.user_paths

  return {
    clientId,
    client,
    setClientId,
    subactionPath: getSubactionPath(),
    setSubactionPath,
    exceptClient,
    subactionPaths,
  }

  function getClient(): readonly [number, RemoteConfigurationSnapshot | null] {
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

  function getSubactionPath() {
    if (!subactionPaths) return subactionPathState
    let subactionPath = subactionPaths.find((d) => d === subactionPathState)
    if (subactionPath) return subactionPath
    subactionPath = subactionPaths[0]
    if (subactionPath) return subactionPath
    return subactionPathState
  }
}

function DeviceSelect({
  serverState,
  type,
  sendMessage,
  data,
}: {
  serverState: ConfigurationSnapshotSet
  type: 'leader' | 'follower'
  sendMessage: sentMessages.SendMessage
  data: ReturnType<typeof useDeviceSelect>
}) {
  const clients = Object.entries(serverState.clients).filter(
    ([id]) => +id !== data.exceptClient,
  )
  const deviceSelectorDisabled =
    !data.client || !data.subactionPaths || data.subactionPaths.length < 1
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
          name={type + 'Id'}
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
              {id}
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
            name={type + 'SubactionPath'}
            autoComplete="off"
            value={data.subactionPath}
            disabled={deviceSelectorDisabled}
            onChange={(evt) => {
              data.setSubactionPath(evt.currentTarget.value)
            }}
          >
            {deviceSelectorDisabled ? (
              <option value="">Select client first</option>
            ) : null}
            {data.subactionPaths?.map((device) => (
              <option key={device} value={device}>
                {device}
              </option>
            ))}
          </Select>
        </label>
        <Button
          type="button"
          onClick={() => {
            sendMessage({
              type: 'TriggerHapticImpulse',
              clientId: data.clientId,
              subactionPath: data.subactionPath,
            })
          }}
        >
          Identify
        </Button>
      </div>
    </div>
  )
}
