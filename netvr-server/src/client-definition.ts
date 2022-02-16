export type RestoreData = { id: number }
export type State = { id: number }

export function restoreState(data: RestoreData): State {
  return data
}

export function serializeState(state: State): RestoreData {
  return state
}

export function initializeState(id: number): State {
  return { id }
}
