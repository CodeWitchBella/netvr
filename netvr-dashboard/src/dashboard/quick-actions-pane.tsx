/** @jsxImportSource @emotion/react */
import { Pane, Button, Input } from '../components/design'
import * as sentMessages from '../protocol/sent-messages'

export function QuickActionsPane(props: {
  sendMessage: sentMessages.SendMessage
  closeSocket: () => void
}) {
  const { sendMessage, closeSocket } = props

  return (
    <Pane title="Quick actions" id="quick-actions">
      <div css={{ display: 'flex', gap: 6, marginBlockEnd: 8 }}>
        <Button
          type="button"
          onClick={() => {
            sendMessage({ type: 'CalibrateByHeadsetPosition' })
          }}
        >
          Sync Devices by headset position
        </Button>
      </div>
      <div>
        <Button
          type="button"
          onClick={() => void sendMessage({ type: 'MoveSomeClients' })}
        >
          Move some clients
        </Button>
      </div>
      <form
        css={{ display: 'flex', gap: 4 }}
        onSubmit={(evt) => {
          evt.preventDefault()
          const name: string = new FormData(evt.currentTarget).get(
            'name',
          ) as any
          if (name) localStorage.setItem('deviceName', name)
          else localStorage.removeItem('deviceName')
          closeSocket()
        }}
      >
        <label>
          deviceName:{' '}
          <Input
            defaultValue={localStorage.getItem('deviceName') ?? ''}
            name="name"
          />
        </label>
        <Button>Change</Button>
      </form>
    </Pane>
  )
}
