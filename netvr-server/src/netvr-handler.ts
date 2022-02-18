import produce, { applyPatches, Patch } from 'immer'
import { batchUsingMicrotasks } from './batch-messages.js'
import type {
  NetvrHandler,
  NetvrRoomOptions,
  Utils,
} from './netvr-id-handler-layer.js'
import {
  netvrKeyValueStore,
  SerializedKeyValueState,
} from './netvr-key-value-store.js'

type Calibration = {
  translate: { x: number; y: number; z: number }
  rotate: { x: number; y: number; z: number }
  scale: { x: number; y: number; z: number }
}

type ClientState = {
  connected: boolean
  calibration: Calibration
}

const sendToSelfAsDebug = false
export function netvrRoomOptions(
  utils: Utils,
): NetvrRoomOptions<SerializedKeyValueState<number, ClientState>> {
  const store = netvrKeyValueStore<number, ClientState>({
    connected: false,
    calibration: {
      translate: { x: 0, y: 0, z: 0 },
      rotate: { x: 0, y: 0, z: 0 },
      scale: { x: 1, y: 1, z: 1 },
    },
  })

  const cancelSaveListener = store.subscribe(
    'change',
    batchUsingMicrotasks(utils.triggerSave).trigger,
  )

  return {
    newConnection: (id) => onConnection(id),
    protocolVersion: 2,
    restoreConnection: (id) => onConnection(id),
    save: store.save,
    restore: store.restore,
    destroy: () => {
      cancelSaveListener()
    },
  }

  function onConnection(id: number): NetvrHandler {
    store.update(id, (draft) => {
      console.log(draft)
      draft.connected = true
    })
    store.drainMicrotasks()

    let stateBeforePatches = store.snapshot()
    let patches: Patch[] = []
    const unsubscribe = store.subscribe('change', (event) => {
      for (const patch of event.patches) patches.push(patch)
    })

    utils.send(id, {
      action: 'full state reset',
      clients: Array.from(stateBeforePatches.entries()).map(([id, data]) => ({
        id,
        data,
      })),
    })

    return {
      destroy() {
        console.log('destroy()')
        unsubscribe()
        store.update(id, (draft) => {
          draft.connected = false
        })
      },
      onJson(message) {
        if (message.action === 'reset room') {
          store.clear()
        } else if (message.action === 'set') {
          if (
            typeof message.client === 'number' &&
            typeof message.field === 'string' &&
            message.field in store.initialValue &&
            message.value &&
            store.has(message.client)
          ) {
            store.update(message.client, (draft) => {
              ;(draft as any)[message.field] = message.value
            })
          } else {
            throw new Error('Invalid action:set message')
          }
        } else if (message.action === 'keep alive') {
          produce(
            stateBeforePatches,
            (draft) => void applyPatches(draft, patches),
            (p) => {
              patches = p
            },
          )

          if (patches.length > 0) {
            utils.send(id, { action: 'patch', patches })
          }

          patches = []
          stateBeforePatches = store.snapshot()
        } else {
          throw new Error(
            'Unknown message action ' + JSON.stringify(message.action),
          )
        }
      },
      onBinary(data) {
        const type = new Uint8Array(data.slice(0, 1))[0]
        if (type === 1 /* tracking info */) {
          const contents = new Uint8Array(data.slice(1))

          const sendBuffer = new Uint8Array(contents.byteLength + 4 + 1)
          const headerView = new DataView(sendBuffer.buffer, 0, 5)
          headerView.setUint8(0, 1) // type
          headerView.setInt32(1, 1, true) // count
          let offset = 5
          sendBuffer.set(contents, offset)

          utils.broadcastBinary(
            sendBuffer,
            sendToSelfAsDebug ? undefined : { omit: id },
          )
        } else if (type === 2 /* haptics */) {
          const target = new DataView(data).getUint32(1, true)
          const request = new Uint8Array(data.slice(4))
          request[0] = 2

          utils.sendBinary(target, request)
        } else {
          console.warn(`Unknown binary message type ${type}`)
        }
      },
    }
  }
}
