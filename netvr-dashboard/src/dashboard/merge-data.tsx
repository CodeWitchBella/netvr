import { ConfigurationSnapshotSet, StateSnapshot } from '../protocol/data'

/**
 * State communicated via UDP to the server (dashboard gets it from the server
 * via websocket in either case)
 */
export type DatagramState = {
  [key: number]: {
    id: number
    time: number
    snapshot: StateSnapshot
  }
}

/**
 * Merges datagram data with configuration snapshot, so that the dashboard can
 * show it all in one place.
 *
 * @param datagramData
 * @param configurationSnapshot
 * @returns
 */
export function mergeData(
  datagramData: DatagramState,
  configurationSnapshot: ConfigurationSnapshotSet | null,
) {
  return Object.fromEntries(
    Object.entries(datagramData)
      .map(([k, v]) => {
        const configuration = configurationSnapshot?.clients[k as any]
        const snapshot = v.snapshot
        if (configuration?.version !== snapshot.required_configuration) {
          return {
            id: k,
            time: new Date(v.time).toISOString().slice(11),
            configuration,
            controllers_raw: snapshot.controllers,
            ...snapshot,
            controllers: undefined,
          }
        }
        return {
          id: k,
          time: new Date(v.time).toISOString().slice(11),
          configuration,
          view: snapshot.view,
          controllers: snapshot.controllers.map((c) => ({
            ...c,
            interaction_profile:
              configuration.interaction_profiles[c.interaction_profile - 1]
                ?.path,
            user_path: configuration.user_paths[c.user_path - 1],
          })),
        }
      })
      .map(({ id, ...rest }) => [id, rest]),
  )
}
/**
 * Type representing the merged data. This describes the full state.
 */
export type MergedData = ReturnType<typeof mergeData>
