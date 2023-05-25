/**
 * Definition of a specification of OpenXR action as passed to the dashboard.
 */
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

/**
 * Snapshot of configuration of single remote client.
 */
export type RemoteConfigurationSnapshot = {
  version: number
  user_paths: readonly string[]
  interaction_profiles: readonly {
    path: string
    bindings: readonly RemoteAction[]
  }[]
  name: string
}

/**
 * Snapshot of configuration of all remote clients.
 */
export type ConfigurationSnapshotSet = {
  clients: { [key: number]: RemoteConfigurationSnapshot }
}

/**
 * 3d vector.
 */
export type Vec3 = { x: number; y: number; z: number }
/**
 * Pose. Position and orientation.
 */
export type Pose = {
  position: Vec3
  orientation: { x: number; y: number; z: number; w: number }
}

/**
 * Snapshot of state of single remote client. As oposed to configuration, this
 * data changes often and as such is transfered via UDP.
 */
export type StateSnapshot = {
  controllers: readonly {
    interaction_profile: number
    user_path: number
    pose: Pose
  }[]
  view: Pose
  required_configuration: number
}
