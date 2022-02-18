import * as immer from 'immer'
import { batchUsingMicrotasks } from './batch-messages.js'

export type SerializedKeyValueState<Key, Value> = readonly (readonly [
  Key,
  Value,
])[]

export function netvrKeyValueStore<Key, Value>(initialValue: Value) {
  immer.enableMapSet()
  immer.enablePatches()

  immer.freeze(initialValue, true)

  let state = new Map<Key, Value>()
  class ChangeEvent extends Event {
    constructor(readonly patches: readonly immer.Patch[]) {
      super('change')
    }
  }
  const target = new EventTarget()

  let patchQueue: immer.Patch[] = []
  const changeEmitter = batchUsingMicrotasks(() => {
    target.dispatchEvent(new ChangeEvent(patchQueue))
    patchQueue = []
  })

  function setState(recipe: (draft: immer.Draft<Map<Key, Value>>) => void) {
    const [nextState, patches] = immer.produceWithPatches(state, (draft) => {
      recipe(draft)
    })
    state = nextState
    for (const patch of patches) patchQueue.push(patch)
    changeEmitter.trigger()
  }

  return {
    restore(savedState: SerializedKeyValueState<Key, Value>) {
      setState((draft) => {
        draft.clear()
        for (const [k, v] of savedState) {
          draft.set(immer.castDraft(k), immer.castDraft(v))
        }
      })
    },
    save(): SerializedKeyValueState<Key, Value> {
      return Array.from(state.entries())
    },
    update(idIn: Key, recipe: (state: immer.Draft<Value>) => void) {
      const id = immer.castDraft(idIn)
      setState((draft) => {
        let value = draft.get(id)
        if (!value) {
          value = immer.createDraft(initialValue)
          draft.set(id, value)
        }
        recipe(value)
      })
    },
    has(id: Key) {
      return state.has(id)
    },
    addEventListener: target.addEventListener.bind(target),
    removeEventListener: target.removeEventListener.bind(target),
    subscribe(type: 'change', callback: (event: ChangeEvent) => void) {
      target.addEventListener(type, callback as any)
      return () => void target.removeEventListener(type, callback as any)
    },
    drainMicrotasks: changeEmitter.drain,
    snapshot: () => state,
    initialValue: immer.castImmutable(initialValue),
  }
}
