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

type DeviceData = { id: number; [key: string]: any }
function deviceReducer(state: DeviceData[], action: any) {
  if (typeof action === 'string') {
    const message = JSON.parse(action)
    if (message.action === 'device info') {
      const incomingIds = new Set<number>(
        message.info.map((info: any) => info.id),
      )
      return state
        .filter((dev) => !incomingIds.has(dev.id))
        .concat(message.info)
    }
  }
  return state
}

export function Dashboard({ socket }: { socket: PWebSocket }) {
  const [devices, dispatchDevices] = useReducer(deviceReducer, [])

  const [log, dispatchLog] = useLog()
  useListenToSocket(socket, (message) => {
    dispatchLog(message)
    dispatchDevices(message)
  })
  useSendKeepAlive(socket)

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'space-between',
      }}
    >
      <div className="devices" style={{ width: 'auto' }}>
        {devices.map((device) => (
          <Device key={device.id} device={device} />
        ))}
      </div>
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
    </div>
  )
}

function Device({ device }: { device: DeviceData }) {
  return (
    <code>
      <pre>{JSON.stringify(device, null, 2)}</pre>
    </code>
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
