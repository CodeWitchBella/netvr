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
      leaderId: ClientId
      leaderSubactionPath: string
      followerId: ClientId
      followerSubactionPath: string
      sampleCount: number
    }
  | {
      type: 'TriggerHapticImpulse'
      clientId: ClientId
      subactionPath: string
    }
