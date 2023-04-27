import type { ConfigurationSnapshotSet, StateSnapshot } from './data'

export type SocketAddr = string
export type ClientId = number

export type ConnectionEstablished = {
  type: 'ConnectionEstablished'
  id: ClientId
  addr: SocketAddr
}
export type FullyConnected = {
  type: 'FullyConnected'
  id: ClientId
}
export type ConnectionClosed = {
  type: 'ConnectionClosed'
  id: ClientId
}

export type DatagramUp = {
  type: 'DatagramUp'
  id: ClientId
  message: StateSnapshot
}

export type ConfigurationSnapshotChanged = {
  type: 'ConfigurationSnapshotChanged'
  value: ConfigurationSnapshotSet
}

export type DashboardMessageDown =
  | ConfigurationSnapshotChanged
  | DatagramUp
  | ConnectionClosed
  | FullyConnected
  | ConnectionEstablished
