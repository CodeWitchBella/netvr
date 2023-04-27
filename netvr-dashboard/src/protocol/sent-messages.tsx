import type { ClientId } from './recieved-messages'

export type SendMessage = (message: DashboardMessageUp) => void

export type DashboardMessageUp =
  | {
      type:
        | 'MoveSomeClients'
        | 'KeepAlive'
        | 'Init'
        | 'CalibrateByHeadsetPosition'
    }
  | { type: 'ResetCalibration'; clientId: ClientId }
  | {
      type: 'StartCalibration'
      targetId: ClientId
      targetSubactionPath: string
      referenceId: ClientId
      referenceSubactionPath: string
      sampleCount: number
    }
  | {
      type: 'TriggerHapticImpulse'
      clientId: ClientId
      subactionPath: string
    }
  | { type: 'SetName'; name: string; clientId: number }
