/** @jsxImportSource @emotion/react */
import { Pane, Button, Input } from '../components/design'
import * as sentMessages from '../protocol/sent-messages'

/**
 * A pane that contains quick actions that can be performed by pushing a single
 * button without having to select anything.
 * @param props
 * @returns
 */
export function QuickActionsPane(props: {
  sendMessage: sentMessages.SendMessage
  closeSocket: () => void
}) {
  const { sendMessage, closeSocket } = props

  return (
    <Pane title="Quick actions" id="quick-actions">
      <div
        css={{ display: 'flex', gap: 6, marginBlockEnd: 8, flexWrap: 'wrap' }}
      >
        <Button
          type="button"
          onClick={() => {
            sendMessage({ type: 'CalibrateByHeadsetPosition' })
          }}
        >
          Sync Devices by headset position
        </Button>
        <Button
          type="button"
          onClick={() => void sendMessage({ type: 'MoveSomeClients' })}
        >
          Move some clients
        </Button>
        <Button
          type="button"
          onClick={() => void sendMessage({ type: 'ResetAllCalibrations' })}
        >
          Reset all calibrations
        </Button>
        <Button
          type="button"
          onClick={() => void sendMessage({ type: 'ForceDisconnectAll' })}
        >
          Disconnect all clients
        </Button>
      </div>
    </Pane>
  )
}
