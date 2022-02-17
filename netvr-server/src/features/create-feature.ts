import type { Utils } from '../netvr-id-handler-layer'

type ActionHandler<Payload, State> = (
  utils: Utils,
  state: State,
  payloads: readonly Payload[],
) => void

type FullState<GlobalState, PerClientState> = {
  global: GlobalState
  client: Map<number, PerClientState>
}

export const createFeature = <
  PerClientState,
  GlobalState,
  Types extends string,
  Payloads extends { [type in Types]: any },
>(
  perClientState: PerClientState,
  globalState: GlobalState,
) => ({
  setActions: (actions: {
    [type in keyof Payloads]: ActionHandler<
      Payloads[type],
      FullState<GlobalState, PerClientState>
    >
  }) => {
    type State = FullState<GlobalState, PerClientState>
    type Act = <Action extends keyof Payloads>(
      action: Action,
      payload: Payloads[Action],
    ) => void
    type MessageHandler = (
      source: number,
      message: { [key: string]: any },
      state: State,
      act: Act,
    ) => State

    Object.freeze(perClientState)
    const events: {
      onReconnect?: (
        clientId: number,
        state: State,
        save: any,
        act: Act,
      ) => State
      onConnect?: (clientId: number, state: State, act: Act) => State
      onDisconnect?: (
        clientId: number,
        state: State,
        act: Act,
      ) => {
        state: State
        save: any
      }
      onBinary?: (
        source: number,
        message: ArrayBuffer,
        state: State,
        act: Act,
      ) => State
    } = {}
    const messageHandlers = new Map<string, MessageHandler>()

    return {
      onMessage(action: string, handler: MessageHandler) {
        if (messageHandlers.has(action))
          throw new Error("Can't set multiple handlers for the same message")
        messageHandlers.set(action, handler)
        return this
      },
      onDisconnect<Save>(
        disconnect: (
          clientId: number,
          state: State,
          act: Act,
        ) => { state: State; save: Save },
      ) {
        if (events.onDisconnect)
          throw new Error('There can only be one onDisconnect per feature')
        events.onDisconnect = disconnect

        return {
          onReconnect: (
            reconnect: (
              clientId: number,
              state: State,
              save: Save,
              act: Act,
            ) => State,
          ) => {
            if (events.onReconnect)
              throw new Error('There can only be one onReconnect per feature')
            events.onReconnect = reconnect
            return this
          },
        }
      },
      onConnect(handler: (clientId: number, state: State, act: Act) => State) {
        if (events.onConnect)
          throw new Error('There can only be one onConnect per feature')
        events.onConnect = handler
        return this
      },
      onBinary(
        handler: (
          source: number,
          message: ArrayBuffer,
          state: State,
          act: <Action extends keyof Payloads>(
            action: Action,
            payload: Payloads[Action],
          ) => void,
        ) => State,
      ) {
        if (events.onBinary)
          throw new Error('There can only be one onBinary per feature')
        events.onBinary = handler
        return this
      },
      finalize() {
        return Object.freeze({
          initialState: Object.freeze({
            global: globalState,
            client: new Map(),
          }),
          onJson: (
            source: number,
            message: { [key: string]: any },
            state: State,
            utils: Utils,
          ) => {
            const handler = messageHandlers.get(message.action)
            if (handler) {
              return actWrap(utils, state, (act) =>
                handler(source, message, state, act),
              )
            }
            return state
          },
          onBinary: (
            source: number,
            message: ArrayBuffer,
            state: State,
            utils: Utils,
          ) => {
            const handler = events.onBinary
            if (handler) {
              return actWrap(utils, state, (act) =>
                handler(source, message, state, act),
              )
            }
            return state
          },
          onConnect: (who: number, state: State, utils: Utils) => {
            const handler = events.onConnect
            if (handler) {
              return actWrap(utils, state, (act) => handler(who, state, act))
            }
            return state
          },
        })

        function actWrap<T>(
          utils: Utils,
          state: State,
          handler: (act: Act) => T,
        ) {
          const payloadsMap = new Map<keyof Payloads, any[]>()
          const act: Act = (action, payload) => {
            let list = payloadsMap.get(action)
            if (!list) {
              list = []
              payloadsMap.set(action, list)
            }
            list.push(payload)
          }

          const res = handler(act)

          for (const [action, payloads] of payloadsMap.entries()) {
            actions[action](utils, state, payloads)
          }

          return res
        }
      },
    }
  },
})
