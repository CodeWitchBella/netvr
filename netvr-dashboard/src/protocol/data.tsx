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

export type StateSnapshot = {
  controllers: readonly {}[]
  views: readonly {}[]
  required_configuration: number
}
