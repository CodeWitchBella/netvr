/**
 * This file has dual purpose. First is to provide (basic) TS types for the real
 * wasm-compiled c++ code. Second is to act as a fallback in case the real code
 * is not present. This (ab)uses the fact that vite resolves .js before .ts[1].
 * It is primarily usefull for local development of features that do not require
 * wasm - you do not need the wasm file to get up and running.
 *
 * [1]: https://vitejs.dev/config/#resolve-extensions
 */

/**
 * Loads wasm asynchronously. Types are setup to match what I have enabled in
 * CMakeLists.txt, so that everything declared here _should_ be available.
 *
 * @returns Promise of loaded Module
 */
export default function WasmModule(): Promise<{
  cwrap: typeof cwrap
  _malloc: (bytes: number) => number
  getValue: (offset: number, type: 'double') => number
  addFunction: (fn: Function, type: string) => number
  UTF8ToString: (ptr: number) => string
}> {
  return Promise.reject(new Error('Fallback was used'))
}

export type Module = Awaited<ReturnType<typeof WasmModule>>

// all following code is copied and slightly modified from @types/emscripten
// the reason I don't use that package is that is unneccessarily pollutes global scope
declare function cwrap<
  I extends Array<JSType | null> | [],
  R extends JSType | null,
>(
  ident: string,
  returnType: R,
  argTypes: I,
  opts?: {
    async?: boolean | undefined
  },
): (...arg: ArgsToType<I>) => ReturnToType<R>

type JSType = 'number' | 'string' | 'array' | 'boolean'
type ReturnToType<R extends JSType | null> = R extends null
  ? null
  : StringToType<Exclude<R, null>>
type StringToType<R extends any> = R extends JSType
  ? {
      number: number
      string: string
      array: number[] | string[] | boolean[] | Uint8Array | Int8Array
      boolean: boolean
      null: null
    }[R]
  : never
type ArgsToType<T extends Array<JSType | null>> = Extract<
  {
    [P in keyof T]: StringToType<T[P]>
  },
  any[]
>
