import type { NetvrHandler, NetvrRoomOptions, Utils } from './netvr-room'
import * as client from './client-definition'

type Client = {
  state: client.State
}

export function netvrRoomOptions(
  utils: Utils,
): NetvrRoomOptions<client.RestoreData> {
  const clients: Client[] = []

  return {
    newConnection: (id) => onConnection(client.initializeState(id)),
    protocolVersion: 1,
    restoreConnection: (data) => onConnection(client.restoreState(data)),
  }

  function onConnection(state: client.State): NetvrHandler<client.RestoreData> {
    const thisClient: Client = { state }
    clients.push(thisClient)
    return {
      save: () => client.serializeState(state),
      destroy() {
        const index = clients.indexOf(thisClient)
        if (index >= 0) {
          clients.splice(index, 1)
        } else {
          console.warn('Client not found. It should deleted only from destroy')
        }
      },
      onBinary() {},
      onJson() {},
    }
  }
}
