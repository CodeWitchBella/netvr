import { useReducer } from 'react'

type MessageData<T extends 'binary' | 'json'> = {
  type: T
  key: number
  message: T extends 'binary' ? ArrayBuffer : any
  timestamp: string
  direction: 'down' | 'up'
}

type State = {
  events: MessageData<'json'>[]
  binaryEvents: MessageData<'binary'>[]
  keyGen: number
}

function logReducer(
  state: State,
  { direction, message }: { direction: 'down' | 'up'; message: any },
) {
  const timestamp = new Date().toISOString()
  if (typeof message === 'string') {
    return {
      ...state,
      events: state.events.concat({
        type: 'json',
        message: JSON.parse(message),
        key: state.keyGen,
        timestamp,
        direction,
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
        message,
        key: state.keyGen,
        timestamp,
        direction,
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
