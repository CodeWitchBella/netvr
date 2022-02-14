import { useState } from 'react'

export function Calibration({
  sendMessage,
}: {
  sendMessage: (message: any) => void
}) {
  const [message, setMessage] = useState('')
  return (
    <div
      style={{
        border: '1px solid gray',
        borderRadius: 4,
        margin: 8,
        padding: 8,
      }}
    >
      <button
        type="button"
        onClick={() => {
          setMessage('Calibrating')

          sendMessage({ type: 'trigger calibration begin' })
        }}
      >
        Trigger Calibration begin
      </button>
      <button
        type="button"
        onClick={() => {
          setMessage('Calibration ended')

          sendMessage({ type: 'trigger calibration end' })
        }}
      >
        Trigger Calibration end
      </button>
      {message}
    </div>
  )
}
