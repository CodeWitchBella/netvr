export function batchUsingMicrotasks(callback: () => void) {
  let promise: null | Promise<void>
  return {
    trigger() {
      if (!promise) {
        promise = Promise.resolve().then(() => {
          promise = null
          callback()
        })
      }
    },
    drain() {
      promise = null
      callback()
    },
  }
}
