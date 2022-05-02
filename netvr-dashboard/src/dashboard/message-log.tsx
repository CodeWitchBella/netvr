/** @jsxImportSource @emotion/react */
import { memo, useMemo, useReducer } from 'react'
import { Pane } from '../components/design'
import { JSONView } from '../components/json-view'

type MessageData<T extends 'binary' | 'json'> = {
  type: T
  key: number
  message: T extends 'binary' ? { raw: ArrayBuffer; parsed: any } : any
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
  action: { direction: 'down' | 'up' } & (
    | { type: 'text'; message: string; parsed: any }
    | { type: 'binary'; message: ArrayBuffer; parsed: any }
  ),
) {
  const timestamp = new Date().toISOString()
  if (action.type === 'text') {
    return {
      ...state,
      events: state.events.concat({
        type: 'json',
        message: action.parsed,
        key: state.keyGen,
        timestamp,
        direction: action.direction,
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
        message: {
          raw: action.message,
          parsed: action.parsed,
        },
        key: state.keyGen,
        timestamp,
        direction: action.direction,
      })
      .slice(-10),
  }
}
const defaultState: State = {
  events: [],
  binaryEvents: [],
  keyGen: 1,
}

export function useLog({ showBinary }: { showBinary: boolean }) {
  const [log, dispatch] = useReducer(logReducer, defaultState)
  return [
    ([] as readonly (MessageData<'binary'> | MessageData<'json'>)[])
      .concat(log.events)
      .concat(showBinary ? log.binaryEvents : [])
      .sort((a, b) => -a.timestamp.localeCompare(b.timestamp)),
    dispatch,
  ] as const
}

export const Message = memo(function Message({
  message,
  timestamp,
  type,
  direction,
}: {
  message: any
  timestamp: string
  type: 'binary' | 'json'
  direction: 'up' | 'down'
}) {
  return (
    <Pane>
      <div>
        {direction === 'down' ? '🔽' : '🔼'} {timestamp}
      </div>
      {type === 'binary' ? (
        <pre>
          <BinaryMessage raw={message.raw} parsed={message.parsed} />
        </pre>
      ) : (
        <JSONView name="message" data={message} shouldExpandNode={() => true} />
      )}
    </Pane>
  )
})

function BinaryMessage({ raw, parsed }: { raw: ArrayBuffer; parsed: any }) {
  const rawText = useMemo(
    () =>
      Array.from(new Uint8Array(raw).values())
        .map((v, i) => (v + '').padStart(3, ' ') + (i % 16 === 15 ? '\n' : ' '))
        .join(''),
    [raw],
  )
  return (
    <>
      <div css={{ whiteSpace: 'pre-wrap', width: 500 }}>
        {'type  ___count_____   __client_id__  #d _#bytes'}
      </div>
      <div css={{ whiteSpace: 'pre-wrap', width: 500 }}>
        {rawText} ({raw.byteLength})
      </div>
      <div css={{ whiteSpace: 'pre-wrap', width: 500 }}>
        <JSONView data={parsed} shouldExpandNode={() => false} />
      </div>
    </>
  )
}
