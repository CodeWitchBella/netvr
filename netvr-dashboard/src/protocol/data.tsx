export type RemoteAction = {
  type:
    | 'Boolean'
    | 'Float'
    | 'Vector2f'
    | 'Pose'
    | 'VibrationOutput'
    | 'Unknown'
  name: string
  localized_name: string
  binding: string
}

export type RemoteConfigurationSnapshot = {
  version: number
  user_paths: readonly string[]
  interaction_profiles: readonly {
    path: string
    bindings: readonly RemoteAction[]
  }[]
}

export type ConfigurationSnapshotSet = {
  clients: { [key: number]: RemoteConfigurationSnapshot }
}

export type Vec3 = { x: number; y: number; z: number }
export type Pose = {
  position: Vec3
  orientation: { x: number; y: number; z: number; w: number }
}

export type StateSnapshot = {
  controllers: readonly {
    interaction_profile: number
    user_path: number
    pose: Pose
  }[]
  view: Pose
  required_configuration: number
}
