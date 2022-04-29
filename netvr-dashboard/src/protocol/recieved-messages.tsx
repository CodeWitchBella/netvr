import { Patch } from 'immer'
import { ServerState } from './data'

export type MessageTransmitLogs = {
  action: 'transmit logs'
  client: number
  logs: readonly {
    text: string
    type: 'error' | 'assert' | 'warning' | 'log' | 'exception'
  }[]
}

export type MessageAssignId = {
  action: "id's here"
  intValue: number
  stringValue: string
}

export type MessageFullStateReset = {
  action: 'full state reset'
  state: ServerState
}

export type MessageStatePatch = {
  action: 'patch'
  patches: readonly (Omit<Patch, 'path'> & { path: string })[]
}

export type RecievedMessage =
  | MessageTransmitLogs
  | MessageAssignId
  | MessageFullStateReset
  | MessageStatePatch
