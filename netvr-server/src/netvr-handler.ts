import type { NetvrHandler, NetvrRoomOptions } from './netvr-room'

type RestoreData = {}

class Handler implements NetvrHandler<RestoreData, Handler> {
  constructor(data?: RestoreData) {}

  save() {
    return {}
  }

  onBinary(message: ArrayBuffer): void {}

  onJson(message: { [type: string]: string }): void {}
}

export const netvrRoomOptions: NetvrRoomOptions<RestoreData, Handler> = {
  newConnection: (id) => new Handler(),
  protocolVersion: 1,
  restoreConnection: (data) => new Handler(data),
}
