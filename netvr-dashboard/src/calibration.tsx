import { useState } from 'react'
import { Button, Pane } from './design'

export function Calibration({
  sendMessage,
}: {
  sendMessage: (message: any) => void
}) {
  const [message, setMessage] = useState('')
  return (
    <Pane>
      <div style={{ display: 'flex', gap: 8 }}>
        <Button
          type="button"
          onClick={() => {
            setMessage('Calibrating')

            sendMessage({ type: 'trigger calibration begin' })
          }}
        >
          Trigger Calibration begin
        </Button>
        <Button
          type="button"
          onClick={() => {
            setMessage('Calibration ended')

            sendMessage({ type: 'trigger calibration end' })
          }}
        >
          Trigger Calibration end
        </Button>
        {message}
      </div>
    </Pane>
  )
}
