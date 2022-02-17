export function batchMessages<Argument>(
  ms: number,
  handler: (args: readonly Argument[]) => void,
) {
  let timeout: ReturnType<typeof setTimeout> | null = null
  let list: Argument[] = []
  return {
    add: (argument: Argument) => {
      if (!timeout) timeout = setTimeout(timeoutHandler, ms)
      list.push(argument)
    },
    remove: (argument: Argument) => {
      const index = list.indexOf(argument)
      if (index < 0) return
      list.splice(index, 1)
      if (list.length < 1) {
        clearTimeout(timeout)
        timeout = null
      }
    },
  }

  function timeoutHandler() {
    timeout = null
    handler(list)
    list.splice(0, list.length)
  }
}

export function batchUsingMicrotasks(callback: () => void) {
  let promise: null | Promise<void>
  return () => {
    if (!promise) {
      promise = Promise.resolve().then(() => {
        promise = null
        callback()
      })
    }
  }
}
