/** @jsxImportSource @emotion/react */
import React, { useReducer } from 'react'
import { Pane, Button } from '../components/design'
import { RemoteConfigurationSnapshot } from '../protocol/data'
import * as sentMessages from '../protocol/sent-messages'
import { ClientId } from '../protocol/recieved-messages'

export function ClientPane({
  client,
  clientId,
  sendMessage,
}: {
  client: RemoteConfigurationSnapshot
  clientId: ClientId
  sendMessage: sentMessages.SendMessage
}) {
  function resetCalibration() {
    sendMessage({ type: 'ResetCalibration', clientId })
  }

  return (
    <Pane id={'client-' + clientId} title={`Client ${clientId}`}>
      <div
        css={{
          display: 'flex',
          flexDirection: 'row-reverse',
          flexWrap: 'wrap',
          gap: 8,
        }}
      >
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
