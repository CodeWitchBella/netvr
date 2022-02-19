import * as immer from 'immer'
import { batchUsingMicrotasks } from './batch-messages.js'

export function immerStore<T>(initialState: T) {
  immer.enableMapSet()
  immer.enablePatches()

  let state = initialState
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

  function setState(recipe: (draft: immer.Draft<T>) => void | T) {
    const [nextState, patches] = immer.produceWithPatches(state, recipe)
    state = nextState
    for (const patch of patches) patchQueue.push(patch)
    changeEmitter.trigger()
  }

  return {
    reset(value: T) {
      setState(() => value)
    },
    update(recipe: (state: immer.Draft<T>) => void) {
      setState(recipe)
    },
    addEventListener: target.addEventListener.bind(target),
    removeEventListener: target.removeEventListener.bind(target),
    subscribe(type: 'change', callback: (event: ChangeEvent) => void) {
      target.addEventListener(type, callback as any)
      return () => void target.removeEventListener(type, callback as any)
    },
    drainMicrotasks: changeEmitter.drain,
    snapshot: () => state,
    initialValue: immer.castImmutable(initialState),
  }
}
