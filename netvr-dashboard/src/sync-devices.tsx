import { notNull } from '@isbl/ts-utils'
import { useState } from 'react'
import { ClientData, mapData } from './data'

export function SyncDevicesButton({
  clients,
  sendMessage,
}: {
  clients: ClientData[]
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
          setMessage('')
          const headsets = clients
            .map((c) => {
              const head = c.info?.find(
                (d) =>
                  d.characteristics.includes('HeadMounted') &&
                  d.characteristics.includes('TrackedDevice') &&
                  d.data,
              )
              if (!head) return null
              const data = mapData(head).data
              const posRotMask = 3 /* position 1 | rotation 2 */
              if (
                data.trackingState &&
                (data.trackingState.value & posRotMask) !== posRotMask
              ) {
                // tracking state is available but either position or rotation
                // is not being tracked
                return null
              }
              if (data.centerEyeRotation && data.centerEyePosition) {
                return {
                  clientId: c.id,
                  position: data.centerEyePosition.value,
                  rotation: data.centerEyeRotation.value,
                }
              }
              if (data.deviceRotation && data.devicePosition) {
                return {
                  clientId: c.id,
                  position: data.devicePosition.value,
                  rotation: data.deviceRotation.value,
                }
              }
              return null
            })
            .filter(notNull)

          if (headsets.length < 2) {
            setMessage('Not enough tracked headsets')
            return
          }
          sendMessage({
            action: 'set calibration',
            calibrations: headsets.map((headset) => ({
              id: headset.clientId,
              translate: {
                x: headset.position[0],
                y: 0,
                z: headset.position[2],
              },
              rotate: {
                x: 0,
                y: headset.rotation[1],
                z: 0,
              },
              scale: { x: 1, y: 1, z: 1 },
            })),
          })
          console.log(headsets)
        }}
      >
        Sync Devices by headset position
      </button>
      {message}
    </div>
  )
}
