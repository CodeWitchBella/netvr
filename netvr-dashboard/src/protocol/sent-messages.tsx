import type { ClientId } from './recieved-messages'

/**
 * Signature of function that sends messages to server.
 */
export type SendMessage = (message: DashboardMessageUp) => void

/**
 * Messages that dashboard can send to server.
 */
export type DashboardMessageUp =
  | {
      type:
        | 'MoveSomeClients'
        | 'KeepAlive'
        | 'Init'
        | 'CalibrateByHeadsetPosition'
        | 'ResetAllCalibrations'
        | 'ForceDisconnectAll'
    }
  | { type: 'ResetCalibration'; clientId: ClientId }
  | {
      type: 'StartCalibration'
      targetId: ClientId
      targetSubactionPath: string
      referenceId: ClientId
      referenceSubactionPath: string

      conf: {
        sample_count: number
        sample_interval_nanos: number
      }
    }
  | {
      type: 'StartHijack'
      targetId: ClientId
      targetSubactionPath: string
      referenceId: ClientId
      referenceSubactionPath: string
    }
  | { type: 'FinishCalibration' }
  | {
      type: 'ReapplyCalibration'
      targetId: ClientId
      targetSubactionPath: string
      referenceId: ClientId
      referenceSubactionPath: string
      data: any
    }
  | {
      type: 'TriggerHapticImpulse'
      clientId: ClientId
      subactionPath: string
    }
  | { type: 'SetName'; name: string; clientId: number }
