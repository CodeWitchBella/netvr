import type { ClientId } from './recieved-messages'

export type SendMessage = (message: DashboardMessageUp) => void

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
