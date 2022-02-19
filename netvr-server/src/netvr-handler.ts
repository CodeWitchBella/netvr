import produce, { applyPatches, Patch } from 'immer'
import { batchUsingMicrotasks } from './batch-messages.js'
import type {
  NetvrHandler,
  NetvrRoomOptions,
  Utils,
} from './netvr-id-handler-layer.js'
import { immerStore } from './immer-store.js'
import * as immer from 'immer'

type Calibration = {
  translate: { x: number; y: number; z: number }
  rotate: { x: number; y: number; z: number }
  scale: { x: number; y: number; z: number }
}

type ClientState = {
  connected: boolean
  calibration: Calibration
}

const version = 1
type SerializedState = {
  version: number
  clients: (Omit<ClientState, 'connected'> & { id: number })[]
}

const emptyClient = immer.freeze(
  {
    connected: false,
    calibration: {
      translate: { x: 0, y: 0, z: 0 },
      rotate: { x: 0, y: 0, z: 0 },
      scale: { x: 1, y: 1, z: 1 },
    },
  },
  true,
)

const sendToSelfAsDebug = false
export function netvrRoomOptions(
  utils: Utils,
): NetvrRoomOptions<SerializedState> {
  const store = immerStore(new Map<number, ClientState>())

  const cancelSaveListener = store.subscribe(
    'change',
    batchUsingMicrotasks(utils.triggerSave).trigger,
  )

  return {
    newConnection: (id) => onConnection(id),
    protocolVersion: 2,
    restoreConnection: (id) => onConnection(id),
    save: () => {
      return {
        version,
        clients: Array.from(store.snapshot().entries()).map(
          ([k, { connected, ...v }]) => ({ ...v, id: k }),
        ),
      }
    },
    restore: (data) => {
      if (version !== data.version) return
      store.reset(
        new Map(
          data.clients.map(({ id, ...client }) => [
            id,
            { connected: false, ...client },
          ]),
        ),
      )
    },
    destroy: () => {
      cancelSaveListener()
    },
  }

  function updateClient(
    id: number,
    recipe: (draft: immer.Draft<ClientState>) => void,
  ) {
    store.update((draft) => {
      let value = draft.get(id)
      if (!value) {
        value = immer.createDraft(emptyClient)
        draft.set(id, value)
      }
      recipe(value)
    })
  }

  function onConnection(id: number): NetvrHandler {
    updateClient(id, (draft) => {
      draft.connected = true
    })
    store.drainMicrotasks()

    let patches: Patch[] = []
    const unsubscribe = store.subscribe('change', (event) => {
      for (const patch of event.patches) patches.push(patch)
    })

    utils.send(id, {
      action: 'full state reset',
      clients: Object.fromEntries(store.snapshot().entries()),
    })

    return {
      destroy() {
        unsubscribe()
        updateClient(id, (draft) => {
          draft.connected = false
        })
        utils.triggerSave()
      },
      onJson(message) {
        if (message.action === 'set') {
          if (
            typeof message.client === 'number' &&
            typeof message.field === 'string' &&
            message.field in store.initialValue &&
            message.value &&
            store.snapshot().has(message.client)
          ) {
            updateClient(message.client, (draft) => {
              ;(draft as any)[message.field] = message.value
            })
          } else {
            throw new Error('Invalid action:set message')
          }
        } else if (message.action === 'keep alive') {
          if (patches.length > 0) {
            utils.send(id, { action: 'patch', patches })
          }

          patches = []
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
