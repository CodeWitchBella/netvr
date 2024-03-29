/** @jsxImportSource @emotion/react */
import { useEffect, useRef, useState } from 'react'
import type {
  RemoteConfigurationSnapshot,
  ConfigurationSnapshotSet,
} from '../protocol/data'
import { Button, Input, Pane, Select } from '../components/design'
import * as sentMessages from '../protocol/sent-messages'
import { MergedData } from './merge-data'
import { JSONView } from '../components/json-view'
import { useLocalStorage } from '../utils'
import { useDropzone } from 'react-dropzone'

/**
 * Pane for triggering calibration and choosing target and reference devices.
 *
 * @param param0
 * @returns
 */
export function CalibrationPane({
  sendMessage,
  serverState,
  mergedData,
}: {
  sendMessage: sentMessages.SendMessage
  serverState: ConfigurationSnapshotSet
  mergedData: MergedData
}) {
  const target = useDeviceSelect({ serverState })
  const reference = useDeviceSelect({
    serverState,
    exceptClient: target.clientId,
  })
  const [shortcutsIn, setShortcuts] = useLocalStorage(
    'enable-keyboard-shortcuts',
    'true' as 'true' | 'false',
    isBooleanString,
  )
  const shortcuts = shortcutsIn === 'true'
  const form = useRef<HTMLFormElement>(null)

  useEffect(() => {
    document.addEventListener('keydown', listener)
    return () => document.removeEventListener('keydown', listener)
    function listener(evt: KeyboardEvent) {
      if (!shortcuts) return
      if (evt.key === 't') {
        console.log('Triggering')
        form.current?.dispatchEvent(
          new Event('submit', { cancelable: true, bubbles: true }),
        )
      }
      if (evt.key === 'a' && target.subactionPaths?.length) {
        target.setSubactionPath(target.subactionPaths[0])
      }
      if (
        evt.key === 's' &&
        target.subactionPaths &&
        target.subactionPaths.length > 1
      ) {
        target.setSubactionPath(target.subactionPaths[1])
      }
      if (evt.key === 'k' && reference.subactionPaths?.length) {
        reference.setSubactionPath(reference.subactionPaths[0])
      }
      if (
        evt.key === 'l' &&
        reference.subactionPaths &&
        reference.subactionPaths.length > 1
      ) {
        reference.setSubactionPath(reference.subactionPaths[1])
      }
    }
  })

  const [message, setMessage] = useState('')
  const dropzone = useDropzone({
    noClick: true,
    multiple: false,
    onDrop: ([file]) => {
      file.text().then((v) => {
        console.log(v)
        sendMessage({
          type: 'ReapplyCalibration',
          targetId: target.clientId,
          targetSubactionPath: target.subactionPath,
          referenceId: reference.clientId,
          referenceSubactionPath: reference.subactionPath,
          data: JSON.parse(v),
        })
        //setSavedCalibration({ fileName: file.name, ...JSON.parse(v) })
      })
    },
  })
  const hijack = useRef(false)
  return (
    <div {...dropzone.getRootProps()}>
      <Pane title="Calibration" id="calibration">
        <input {...dropzone.getInputProps()} css={{ display: 'none' }} />
        <form
          ref={form}
          onSubmit={(evt) => {
            evt.preventDefault()

            const formData = new FormData(evt.currentTarget)
            const data: {
              target: number
              targetDevice: string
              reference: number
              referenceDevice: string
            } = Object.fromEntries(
              Array.from(formData.entries(), ([k, v]) => [k, +v]),
            ) as any
            const str = (v: unknown) => {
              if (typeof v !== 'string') throw new Error('Missing value')
              return v
            }
            const num = (v: unknown) => +str(v)

            if (hijack.current) {
              sendMessage({
                type: 'StartHijack',
                targetId: num(formData.get('targetId')),
                targetSubactionPath: str(formData.get('targetSubactionPath')),
                referenceId: num(formData.get('referenceId')),
                referenceSubactionPath: str(
                  formData.get('referenceSubactionPath'),
                ),
              })
            } else {
              sendMessage({
                type: 'StartCalibration',
                targetId: num(formData.get('targetId')),
                targetSubactionPath: str(formData.get('targetSubactionPath')),
                referenceId: num(formData.get('referenceId')),
                referenceSubactionPath: str(
                  formData.get('referenceSubactionPath'),
                ),
                conf: {
                  sample_count: 500,
                  sample_interval_nanos: 1000 * 1000 * 20,
                },
              })
            }
            hijack.current = false

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
            '.highlight': {
              display: shortcuts ? 'inline' : 'none',
            },
          }}
        >
          <label css={{ userSelect: 'none' }}>
            <Input
              type="checkbox"
              checked={shortcuts}
              onChange={(evt) => setShortcuts(evt.currentTarget.checked + '')}
            />{' '}
            Enable keyboard shortcuts
          </label>
          <DeviceSelect
            serverState={serverState}
            data={target}
            type="target"
            sendMessage={sendMessage}
            shortcut="[AS]"
          />
          <div css={{ marginTop: -16 }}>
            <JSONView
              name="data"
              shouldExpandNode={() => true}
              data={mergedData[target.clientId]?.controllers?.find(
                (c) => c.user_path === target.subactionPath,
              )}
            />
          </div>

          <DeviceSelect
            serverState={serverState}
            data={reference}
            type="reference"
            sendMessage={sendMessage}
            shortcut="[KL]"
          />
          <div css={{ marginTop: -16 }}>
            <JSONView
              name="data"
              shouldExpandNode={() => true}
              data={mergedData[reference.clientId]?.controllers?.find(
                (c) => c.user_path === reference.subactionPath,
              )}
            />
          </div>
          <Button type="submit">
            <span className="highlight">[</span>T
            <span className="highlight">]</span>rigger Calibration
          </Button>
          <Button
            onClick={() => {
              hijack.current = true
            }}
            type="submit"
          >
            Data collection
          </Button>
          <Button
            onClick={() => sendMessage({ type: 'FinishCalibration' })}
            type="button"
          >
            Data collection finish
          </Button>
          {message}
        </form>
      </Pane>
    </div>
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
  shortcut,
}: {
  serverState: ConfigurationSnapshotSet
  type: 'target' | 'reference'
  sendMessage: sentMessages.SendMessage
  data: ReturnType<typeof useDeviceSelect>
  shortcut: string
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
              {client.name} #{id}
            </option>
          ))}
        </Select>
      </label>
      <div css={{ display: 'flex', alignItems: 'center' }}>
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
        <div className="highlight">{shortcut}</div>
      </div>
    </div>
  )
}
function isBooleanString(v: unknown): v is 'true' | 'false' {
  return v === 'true' || v === 'false'
}
