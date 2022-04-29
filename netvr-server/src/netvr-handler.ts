import produce, { applyPatches, Patch } from 'immer'
import { batchUsingMicrotasks } from './batch-messages.js'
import type {
  ConnectionInfo,
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
  connectionInfo: ConnectionInfo
  calibration: Calibration
}

const version = 1
type SerializedState = {
  version: number
  clients: (Omit<ClientState, 'connected'> & { id: number })[]
}

const emptyClient = immer.freeze<ClientState>(
  {
    connected: false,
    connectionInfo: {},
    calibration: {
      translate: { x: 0, y: 0, z: 0 },
      rotate: { x: 0, y: 0, z: 0 },
      scale: { x: 1, y: 1, z: 1 },
    },
  },
  true,
)

const sampleClient = {
  devices: [],
}
const sendToSelfAsDebug = false
export function netvrRoomOptions(
  utils: Utils,
): NetvrRoomOptions<SerializedState> {
  const store = immerStore({ clients: new Map<number, ClientState>() })

  const cancelSaveListener = store.subscribe(
    'change',
    batchUsingMicrotasks(utils.triggerSave).trigger,
  )

  return {
    newConnection: onConnection,
    protocolVersion: 2,
    restoreConnection: onConnection,
    save: () => {
      return {
        version,
        clients: Array.from(store.snapshot().clients.entries()).map(
          ([k, { connected, ...v }]) => ({ ...v, id: k }),
        ),
      }
    },
    restore: (data) => {
      if (version !== data.version) return
      store.reset({
        clients: new Map(
          data.clients.map(({ id, ...client }) => [
            id,
            { connected: false, ...client },
          ]),
        ),
      })
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
      let value = draft.clients.get(id)
      if (!value) {
        value = immer.createDraft(emptyClient)
        draft.clients.set(id, value)
      }
      recipe(value)
    })
  }

  function onConnection(
    id: number,
    connectionInfo: ConnectionInfo,
  ): NetvrHandler {
    updateClient(id, (draft) => {
      draft.connected = true
      draft.connectionInfo = connectionInfo
    })
    store.drainMicrotasks()

    let patches: Patch[] = []
    const unsubscribe = store.subscribe('change', (event) => {
      for (const patch of event.patches) patches.push(patch)
    })

    {
      const snap = store.snapshot()
      utils.send(id, {
        action: 'full state reset',
        state: {
          ...snap,
          clients: Object.fromEntries(snap.clients.entries()),
        },
      })
    }

    function sendPatches() {
      if (patches.length > 0) {
        utils.send(id, {
          action: 'patch',
          patches: patches.map((p) => ({ ...p, path: '/' + p.path.join('/') })),
        })
      }

      patches = []
    }

    return {
      destroy() {
        unsubscribe()
        updateClient(id, (draft) => {
          draft.connected = false
        })
        utils.triggerSave()
      },
      onJson(message) {
        if (message.action === 'keep alive') {
          sendPatches()
          return
        }
        console.log('message from:', id, message)
        if (message.feature === 'calibration') {
          if (message.action === 'begin') {
            utils.send(message.leader, message)
            utils.send(message.follower, message)
          } else if (message.action === 'samples') {
            utils.send(message.leader, message)
          } else {
            throw new Error('Unknown message for calibration feature')
          }
        } else if (message.action === 'multiset') {
          const data = message.data
          if (Array.isArray(data)) {
            store.update((draft) => {
              for (const item of data) {
                if (
                  typeof item.client === 'number' &&
                  typeof item.field === 'string' &&
                  item.field in sampleClient &&
                  item.value &&
                  draft.clients.has(item.client)
                ) {
                  var client = draft.clients.get(item.client)
                  ;(client as any)[item.field] = item.value
                } else {
                  throw new Error('Invalid multiset item')
                }
              }
            })
          }
        } else if (message.action === 'set') {
          if (
            typeof message.client === 'number' &&
            typeof message.field === 'string' &&
            message.field in sampleClient &&
            message.value &&
            store.snapshot().clients.has(message.client)
          ) {
            updateClient(message.client, (draft) => {
              ;(draft as any)[message.field] = message.value
            })
          } else {
            throw new Error('Invalid action:set message')
          }
        } else if (message.action === 'quit') {
          utils.send(message.client, message)
        } else if (message.action === 'request logs') {
          utils.send(message.client, { ...message, client: id })
        } else if (message.action === 'transmit logs') {
          utils.send(message.client, { ...message, client: id })
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

          let headerLength = 4 + 4 + 1
          const sendBuffer = new Uint8Array(contents.byteLength + headerLength)
          const headerView = new DataView(sendBuffer.buffer, 0, headerLength)
          headerView.setUint8(0, 1) // type
          headerView.setUint32(1, 1, true) // count
          headerView.setUint32(5, id, true) // client id
          sendBuffer.set(contents, headerLength)

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
        sendPatches()
      },
    }
  }
}
