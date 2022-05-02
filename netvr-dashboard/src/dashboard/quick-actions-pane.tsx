/** @jsxImportSource @emotion/react */
import { Pane, Button } from '../components/design'
import { ClientBinaryData, ServerState } from '../protocol/data'
import * as sentMessages from '../protocol/sent-messages'
import { useSyncClientsByHeadset } from './use-sync-clients-by-headset'

export function QuickActionsPane(props: {
  sendMessage: sentMessages.SendMessage
  clients: readonly ClientBinaryData[]
  serverState: ServerState
  closeSocket: () => void
}) {
  const { sendMessage, closeSocket } = props
  const syncDevicesByHeadset = useSyncClientsByHeadset(props)

  return (
    <Pane title="Quick actions" id="quick-actions">
      <div css={{ display: 'flex', gap: 6 }}>
        <Button
          type="button"
          onClick={() => {
            sendMessage(sentMessages.resetRoom())
          }}
        >
          Reset room
        </Button>
        <Button type="button" onClick={syncDevicesByHeadset.onClick}>
          Sync Devices by headset position
        </Button>
        {syncDevicesByHeadset.message}
      </div>
      <form
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
          <input
            defaultValue={localStorage.getItem('deviceName') ?? ''}
            name="name"
          />
        </label>
        <button>set</button>
      </form>
    </Pane>
  )
}
