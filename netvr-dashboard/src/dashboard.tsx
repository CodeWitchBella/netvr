import { useEffect, useReducer, useRef, useState } from 'react'
import { useLog } from './log'
import { useListenToSocket } from './use-listen-to-socket'
import type { PWebSocket } from './utils'

function useSendKeepAlive(socket: PWebSocket) {
  useEffect(() => {
    const interval = setInterval(() => {
      socket.send(JSON.stringify({ action: 'keep alive' }))
    }, 250)
    return () => {
      clearInterval(interval)
    }
  }, [socket])
}

export function Dashboard({ socket }: { socket: PWebSocket }) {
  const [devices, dispatchDevices] = useReducer(
    (state: { id: number; [key: string]: any }[], action: any) => {
      if (typeof action === 'string') {
        const message = JSON.parse(action)
        if (message.action === 'device info') {
        }
      }
      return state
    },
    [],
  )

  const [log, dispatchLog] = useLog()
  useListenToSocket(socket, (message) => {
    dispatchLog(message)
    dispatchDevices(message)
  })
  useSendKeepAlive(socket)

  return (
    <div className="events">
      {log.map((event) => (
        <Message
          message={event.message}
          key={event.key}
          timestamp={event.timestamp}
          type={event.type}
        />
      ))}
    </div>
  )
}

function Message({
  message,
  timestamp,
  type,
}: {
  message: any
  timestamp: string
  type: 'binary' | 'json'
}) {
  return (
    <div className="event">
      <div>🔽 {timestamp}</div>
      <pre>
        {type === 'binary' ? message + '' : JSON.stringify(message, null, 2)}
      </pre>
    </div>
  )
}
