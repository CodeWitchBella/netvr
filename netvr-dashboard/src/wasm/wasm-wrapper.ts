import WasmModule, { Module } from './cpp_netvr'
type Value = Awaited<ReturnType<typeof createModule>>

let cachePromise: Promise<Value | null> | null = null
let cacheResult: Value | null | undefined = undefined
function loadWasm():
  | {
      status: 'done'
      value: Value | null
    }
  | {
      status: 'loading'
      value: Promise<Value | null>
    } {
  if (cacheResult !== undefined) return { status: 'done', value: cacheResult }
  if (!cachePromise) {
    cachePromise = createModule().then(
      (v) => {
        cacheResult = v
        return v
      },
      (error) => {
        console.error(error)
        cacheResult = undefined
        return null
      },
    )
  }
  return { status: 'loading', value: cachePromise }
}

export function useWasmSuspending() {
  const wasm = loadWasm()
  if (wasm.status === 'loading') throw wasm.value
  return wasm.value
}

type NotNull<T> = T extends null ? never : T
export type WrappedWasm = NotNull<Awaited<ReturnType<typeof useWasmSuspending>>>

async function createModule() {
  const module = await WasmModule()
  initModule(module)

  let calibrationMethods: ReturnType<typeof wrapCalibrationMethods> | undefined
  return {
    calibration: () => {
      calibrationMethods ??= wrapCalibrationMethods(module)
      const methods = calibrationMethods

      const handle = calibrationMethods.create()
      let onCall: (() => void) | null = null
      return {
        addPair: (
          p1: CalibrationSample['position'],
          q1: CalibrationSample['rotation'],
          p2: CalibrationSample['position'],
          q2: CalibrationSample['rotation'],
        ) => {
          onCall?.()
          methods.addPair(handle, ...p1, ...q1, ...p2, ...q2)
        },
        compute: () => {
          onCall?.()
          methods.compute(handle, methods.buffer)
          let i = 0
          return {
            tx: module.getValue(methods.buffer + i++ * 8, 'double'),
            ty: module.getValue(methods.buffer + i++ * 8, 'double'),
            tz: module.getValue(methods.buffer + i++ * 8, 'double'),
            rex: module.getValue(methods.buffer + i++ * 8, 'double'),
            rey: module.getValue(methods.buffer + i++ * 8, 'double'),
            rez: module.getValue(methods.buffer + i++ * 8, 'double'),
            rqx: module.getValue(methods.buffer + i++ * 8, 'double'),
            rqy: module.getValue(methods.buffer + i++ * 8, 'double'),
            rqz: module.getValue(methods.buffer + i++ * 8, 'double'),
            rqw: module.getValue(methods.buffer + i++ * 8, 'double'),
          }
        },
        destroy: () => {
          onCall?.()
          methods.destroy(handle)
          onCall = () => {
            throw new Error('Calibration was already destroyed')
          }
        },
      }
    },
  }
}

function initModule(module: Module) {
  const logger = module.addFunction(
    (textPtr: number) => void console.log(module.UTF8ToString(textPtr)),
    'vi',
  )
  const set_logger = module.cwrap('isbl_netvr_set_logger', null, ['number'])
  set_logger(logger)
}

function wrapCalibrationMethods(module: Module) {
  const create = module.cwrap('isbl_netvr_calibration_create', 'number', [])
  const destroy = module.cwrap('isbl_netvr_calibration_destroy', null, [
    'number',
  ])
  // prettier-ignore
  const addPair = module.cwrap('isbl_netvr_calibration_add_pair', null, [
    'number',
    'number', 'number', 'number', 'number', 'number', 'number', 'number',
    'number', 'number', 'number', 'number', 'number', 'number', 'number',
  ])
  const compute = module.cwrap('isbl_netvr_calibration_compute', null, [
    'number',
    'number',
  ])

  const buffer = module._malloc(
    8 /* sizeof(double) */ * 10 /* number of fields in CalibrationResult */,
  )
  return {
    create,
    destroy,
    addPair,
    compute,
    buffer,
  }
}

type CalibrationSample = {
  position: readonly [x: number, y: number, z: number]
  rotation: readonly [x: number, y: number, z: number, w: number]
}
