// @ts-nocheck
import { notNull } from '@isbl/ts-utils'
import { useState } from 'react'
import * as sentMessages from '../protocol/sent-messages'
import { DatagramState } from './merge-data'

type Props = {
  state: DatagramState
  sendMessage: sentMessages.SendMessage
}

/**
 * Triggers the simple calibration algorithm which uses the headset position.
 * @param param0
 * @returns
 */
export function useSyncClientsByHeadset({ state, sendMessage }: Props) {
  const [message, setMessage] = useState('')
  return { onClick: syncClientsByHeadset, message }

  function syncClientsByHeadset() {
    setMessage('')
    const headsets = Object.entries(state)
      .map(([clientId, c]) => {
        console.log(c)

        const data = c.snapshot
        const posRotMask = 3 /* position 1 | rotation 2 */
        if (
          data.view.position &&
          data.view.orientation &&
          (data.view.position.x !== 0 ||
            data.view.position.z !== 0 ||
            data.view.orientation.y !== 0)
        ) {
          return {
            clientId: clientId,
            position: data.view.position,
            rotation: data.view.orientation,
          }
        }
        return null
      })
      .filter(notNull)

    if (headsets.length < 1) {
      setMessage('Not enough tracked headsets')
      return
    }
    sendMessage({ type: 'CalibrateByHeadsetPosition' })
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
