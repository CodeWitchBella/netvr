import { notNull } from '@isbl/ts-utils'
import { useState } from 'react'
import { ClientBinaryData, mapData, ServerState } from '../protocol/data'
import { Button } from '../components/design'
import * as sentMessages from '../protocol/sent-messages'

type Props = {
  clients: readonly ClientBinaryData[]
  serverState: ServerState
  sendMessage: (message: any) => void
}

export function SyncDevicesButton(props: Props) {
  const syncDevicesByHeadset = useSyncClientsByHeadset(props)
  return (
    <>
      <Button type="button" onClick={syncDevicesByHeadset.onClick}>
        Sync Devices by headset position
      </Button>
      {syncDevicesByHeadset.message}
    </>
  )
}

function useSyncClientsByHeadset({ clients, serverState, sendMessage }: Props) {
  const [message, setMessage] = useState('')
  return { onClick: syncClientsByHeadset, message }

  function syncClientsByHeadset() {
    setMessage('')
    const headsets = clients
      .map((c) => {
        const headInfo = serverState.clients[c.clientId]?.devices?.find(
          (d) =>
            d.characteristics.includes('HeadMounted') &&
            d.characteristics.includes('TrackedDevice') &&
            d.data,
        )
        if (!headInfo) return null
        const headData = c.devices.find((d) => d.deviceId === headInfo.localId)
        if (!headData) return null
        const data = mapData(headData, headInfo).data
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
            clientId: c.clientId,
            position: data.centerEyePosition.value,
            rotation: data.centerEyeRotation.value,
          }
        }
        if (data.deviceRotation && data.devicePosition) {
          return {
            clientId: c.clientId,
            position: data.devicePosition.value,
            rotation: data.deviceRotation.value,
          }
        }
        return null
      })
      .filter(notNull)

    if (headsets.length < 1) {
      setMessage('Not enough tracked headsets')
      return
    }
    sendMessage(
      sentMessages.setCalibration(
        headsets.map((headset) => {
          const x = headset.position.x
          const z = headset.position.z
          const angle = headset.rotation.y
          return {
            client: headset.clientId,
            value: {
              translate: {
                x: -Math.cos(angle) * x + Math.sin(angle) * z,
                y: 0,
                z: -Math.sin(angle) * x - Math.cos(angle) * z,
              },
              rotate: {
                x: 0,
                y: -angle,
                z: 0,
              },
              scale: { x: 1, y: 1, z: 1 },
            },
          }
        }),
      ),
    )
  }
}

function invertAndDecompose(
  position: readonly [number, number, number],
  rotation: readonly [number, number, number],
) {
  const matrix = new DOMMatrix()
    .rotate(0, rotation[1], 0)
    .translate(position[0], 0, position[2])
  return extract(matrix)
}
function extract(mat: DOMMatrix) {
  // supports only scale*rotate*translate matrix
  var radians = Math.PI / 180
  // prettier-ignore
  const m = [
    mat.m11, mat.m12, mat.m13, mat.m14,
    mat.m21, mat.m22, mat.m23, mat.m24,
    mat.m31, mat.m32, mat.m33, mat.m34,
    mat.m41, mat.m42, mat.m43, mat.m44
  ]

  var sX = Math.sqrt(m[0] * m[0] + m[1] * m[1] + m[2] * m[2]),
    sY = Math.sqrt(m[4] * m[4] + m[5] * m[5] + m[6] * m[6]),
    sZ = Math.sqrt(m[8] * m[8] + m[9] * m[9] + m[10] * m[10])

  var rX = Math.atan2(-m[9] / sZ, m[10] / sZ) / radians,
    rY = Math.asin(m[8] / sZ) / radians,
    rZ = Math.atan2(-m[4] / sY, m[0] / sX) / radians

  if (m[4] === 1 || m[4] === -1) {
    rX = 0
    rY = (m[4] * -Math.PI) / 2
    rZ = (m[4] * Math.atan2(m[6] / sY, m[5] / sY)) / radians
  }

  var tX = m[12] / sX,
    tY = m[13] / sX,
    tZ = m[14] / sX

  return {
    translate: { x: tX, y: tY, z: tZ },
    rotate: { x: rX, y: rY, z: rZ },
    scale: { x: sX, y: sY, z: sZ },
  }
}
