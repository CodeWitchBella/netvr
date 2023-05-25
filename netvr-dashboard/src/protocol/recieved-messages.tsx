import type { ConfigurationSnapshotSet, StateSnapshot } from './data'

/**
 * Corresponds with SocketAddr in Rust.
 */
export type SocketAddr = string
/**
 * Corresponds with ClientId in Rust.
 */
export type ClientId = number

/**
 * Message semt from, server to dashboard when a new client starts connecting.
 */
export type ConnectionEstablished = {
  type: 'ConnectionEstablished'
  id: ClientId
  addr: SocketAddr
}
/**
 * Message sent from server to dashboard when a client is fully connected.
 */
export type FullyConnected = {
  type: 'FullyConnected'
  id: ClientId
}
/**
 * Message sent from server to dashboard when a client disconnects.
 */
export type ConnectionClosed = {
  type: 'ConnectionClosed'
  id: ClientId
}

/**
 * Server forwards datagrams from clients to dashboard as this message.
 */
export type DatagramUp = {
  type: 'DatagramUp'
  id: ClientId
  message: StateSnapshot
}

/**
 * Message sent from server to dashboard when configuration changes.
 *
 */
export type ConfigurationSnapshotChanged = {
  type: 'ConfigurationSnapshotChanged'
  value: ConfigurationSnapshotSet
}

/**
 * Message sent from server to dashboard.
 */
export type DashboardMessageDown =
  | ConfigurationSnapshotChanged
  | DatagramUp
  | ConnectionClosed
  | FullyConnected
  | ConnectionEstablished
