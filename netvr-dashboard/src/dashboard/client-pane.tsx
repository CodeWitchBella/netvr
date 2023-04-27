/** @jsxImportSource @emotion/react */
import { ReactNode, useState } from 'react'
import { Pane, Button, Input } from '../components/design'
import { RemoteConfigurationSnapshot, StateSnapshot } from '../protocol/data'
import * as sentMessages from '../protocol/sent-messages'
import { ClientId } from '../protocol/recieved-messages'

export function ClientPane({
  client,
  clientId,
  sendMessage,
  stateSnapshot,
}: {
  client: RemoteConfigurationSnapshot
  clientId: ClientId
  sendMessage: sentMessages.SendMessage
  stateSnapshot: StateSnapshot | null
}) {
  function resetCalibration() {
    sendMessage({ type: 'ResetCalibration', clientId })
  }

  return (
    <Pane id={'client-' + clientId} title={`Client ${clientId}`}>
      <div css={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        <div
          css={{
            padding: 8,
            display: 'flex',
            flexDirection: 'row',
            flexWrap: 'wrap',
            justifyContent: 'space-between',
          }}
        >
          <NameInput
            remoteValue={client.name}
            submit={(name) => {
              sendMessage({ type: 'SetName', name, clientId })
            }}
          />
          <Button type="button" onClick={resetCalibration}>
            Reset
          </Button>
        </div>
        {client.user_paths?.map((userPath) => {
          return (
            <Device
              userPath={userPath}
              key={userPath}
              clientId={clientId}
              sendMessage={sendMessage}
            />
          )
        }) ?? null}
      </div>
    </Pane>
  )
}

function NameInput({
  remoteValue,
  submit,
}: {
  remoteValue: string
  submit: (value: string) => void
}) {
  const [value, setValue] = useState<null | string>(null)
  return (
    <form
      onSubmit={(evt) => {
        evt.preventDefault()
        const data = new FormData(evt.currentTarget)
        submit(data.get('name') as any)
        setTimeout(() => {
          setValue(null)
        }, 250)
      }}
    >
      <label css={{ display: 'flex', gap: 4, alignItems: 'center' }}>
        Name:
        <Input
          name="name"
          value={value ?? remoteValue}
          onChange={(evt) => void setValue(evt.currentTarget.value)}
        />
        <Button>Set</Button>
      </label>
    </form>
  )
}

function ClientContent({
  children,
  className,
}: {
  children: ReactNode
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

function Device({
  userPath,
  sendMessage,
  clientId,
}: {
  userPath: string
  sendMessage: sentMessages.SendMessage
  clientId: number
}) {
  return (
    <ClientContent
      css={{
        border: '1px solid var(--base-2)',
        paddingBlockStart: 8,
        paddingBlockEnd: 8,
      }}
    >
      <div css={{ display: 'flex', justifyContent: 'space-between' }}>
        <div>Device: {userPath}</div>

        <Button
          type="button"
          onClick={() => {
            sendMessage({
              type: 'TriggerHapticImpulse',
              clientId,
              subactionPath: userPath,
            })
          }}
        >
          Identify
        </Button>
      </div>
    </ClientContent>
  )
}
