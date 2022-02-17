import { produce } from 'immer'
import { createFeature } from './create-feature'

type Calibration = {
  translate: { x: number; y: number; z: number }
  rotate: { x: number; y: number; z: number }
  scale: { x: number; y: number; z: number }
}

const identity = {
  translate: { x: 0, y: 0, z: 0 },
  rotate: { x: 0, y: 0, z: 0 },
  scale: { x: 1, y: 1, z: 1 },
}

export const calibrationFeature = createFeature(identity, {
  connectedClients: new Set<number>(),
})
  .setActions({
    distribute: (utils, state, list: readonly number[]) => {
      const lastCalibrations = new Map<number, Calibration>()
      for (const cal of list) {
        if (state.global.connectedClients.has(cal.id)) {
          lastCalibrations.set(cal.id, cal)
        }
      }

      utils.broadcast({
        action: 'set calibration',
        calibrations: Array.from(lastCalibrations.values()),
      })
    },
    sendAll: (utils, state, targets: readonly number[]) => {
      for (const target of targets) {
        utils.send(target, {
          action: 'set calibration',
          calibrations: Array.from(state.calibrations.values()),
        })
      }
    },
  })
  .onMessage('set calibration', (source, message, state, act) => {
    for (const cal of message.calibrations) {
      act('distribute', cal)
    }
    return produce(state, (draft) => {
      for (const cal of message.calibrations) {
        draft.calibrations.set(cal.id, cal)
      }
    })
  })
  .onConnect((clientId, state, act) => {
    act('sendAll', clientId)
    return produce(state, (draft) => {
      draft.calibrations.set(clientId, createIdentity(clientId))
      draft.connectedClients.add(clientId)
    })
  })
  .onDisconnect((clientId, state) => {
    const current = state.connectedClients.has(clientId)
    if (!current) throw new Error('Not connected client disconnected. How!?')
    return {
      save: null,
      state: produce(state, (draft) => {
        draft.connectedClients.delete(clientId)
      }),
    }
  })
  .onReconnect((clientId, state, save, act) => {
    act('sendAll', clientId)
    act('distribute', clientId)
    return produce(state, (draft) => {
      draft.connectedClients.add(clientId)
    })
  })
  .finalize()
