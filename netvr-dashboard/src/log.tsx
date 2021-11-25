import { useReducer } from 'react'

type MessageData<T extends 'binary' | 'json'> = {
  type: T
  key: number
  message: T extends 'binary' ? ArrayBuffer : any
  timestamp: string
}

type State = {
  events: MessageData<'json'>[]
  binaryEvents: MessageData<'binary'>[]
  keyGen: number
}

function logReducer(state: State, action: any) {
  const timestamp = new Date().toISOString()
  if (typeof action === 'string') {
    return {
      ...state,
      events: state.events.concat({
        type: 'json',
        message: JSON.parse(action),
        key: state.keyGen,
        timestamp,
      }),
      keyGen: state.keyGen + 1,
    }
  }
  return {
    ...state,
    keyGen: state.keyGen + 1,
    binaryEvents: state.binaryEvents
      .concat({
        type: 'binary',
        message: action,
        key: state.keyGen,
        timestamp,
      })
      .slice(-10),
  }
}
const defaultState: State = {
  events: [],
  binaryEvents: [],
  keyGen: 1,
}

export function useLog() {
  const [log, dispatch] = useReducer(logReducer, defaultState)
  return [
    ([] as readonly (MessageData<'binary'> | MessageData<'json'>)[])
      .concat(log.events)
      .concat(log.binaryEvents)
      .sort((a, b) => -a.timestamp.localeCompare(b.timestamp)),
    dispatch,
  ] as const
}
