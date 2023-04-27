import { useCallback, useEffect, useState } from 'react'

export function useLocalStorage<Value extends string = string>(
  key: string,
  defaultValue: Value,
  validate: (v: string) => v is Value,
) {
  const [state, setState] = useState<{ key: string; value: string | null }>(
    () => ({
      key,
      value: localStorage.getItem(key),
    }),
  )
  useEffect(() => {
    setState((prev) =>
      prev.key === key ? prev : { key, value: localStorage.getItem(key) },
    )
    window.addEventListener('storage', listener)
    return () => window.removeEventListener('storage', listener)
    function listener(event: StorageEvent) {
      if (event.key === key) {
        console.log(event)
        const newValue = event.newValue
        setState((prev) =>
          prev.value === newValue ? prev : { key, value: newValue },
        )
      }
    }
  }, [key])

  return [
    state.value === null
      ? defaultValue
      : validate(state.value)
      ? state.value
      : defaultValue,
    useCallback(
      (value: string) => {
        if (validate(value)) {
          localStorage.setItem(key, value)
          setState({ key, value })
        }
      },
      [key, validate],
    ),
  ] as const
}
