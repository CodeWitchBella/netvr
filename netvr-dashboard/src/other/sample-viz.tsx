/** @jsxImportSource @emotion/react */
import { OrbitControls } from '@react-three/drei'
import { Canvas, useFrame } from '@react-three/fiber'
import { useControls } from 'leva'
import { Suspense, useEffect, useMemo, useRef, useState } from 'react'
import { useDropzone } from 'react-dropzone'
import * as THREE from 'three'
import { InstancedMesh } from 'three'
import {
  ReprovideTheme,
  useReprovideTheme,
  useTheme,
} from '../components/theme'
import { useWasmSuspending, type WrappedWasm } from '../wasm/wasm-wrapper'
import { ErrorBoundary } from '../components/error-boundary'
import { compute } from '../wasm2/netvr_calibrate'

type SavedCalibration = {
  fileName: string
  target: Sample[]
  reference: Sample[]
  target_name?: string
  reference_name?: string
}

type Pose = {
  position: { x: number; y: number; z: number }
  orientation: { x: number; y: number; z: number; w: number }
}
type Sample = {
  pose: Pose
  prev_pose: Pose
  flags: number
  prev_flags: number
  nanos: number
  now_nanos: number
}

export default function SampleVizRoute() {
  const [savedCalibration, setSavedCalibration] =
    useState<SavedCalibration | null>(null)
  const dropzone = useDropzone({
    noClick: true,
    multiple: false,
    onDrop: ([file]) => {
      file.text().then((v) => {
        setSavedCalibration({ fileName: file.name, ...JSON.parse(v) })
      })
    },
  })
  return (
    <div
      css={{
        flexGrow: 1,
        display: 'flex',
        alignItems: 'stretch',
        justifyContent: 'stretch',
        maxHeight: '100vh',
      }}
      {...dropzone.getRootProps()}
    >
      <input {...dropzone.getInputProps()} css={{ display: 'none' }} />
      <Canvas style={{ height: 'auto' }}>
        <ReprovideTheme value={useReprovideTheme()}>
          <Suspense fallback={<SpinningCube />}>
            <Scene data={savedCalibration} />
          </Suspense>
        </ReprovideTheme>
      </Canvas>
    </div>
  )
}

function SpinningCube() {
  const theme = useTheme()
  const boxRef = useRef<THREE.Mesh>(null)
  const startTime = useRef(Date.now())

  useFrame(() => {
    const now = Date.now()
    if (now - startTime.current < 100) return
    boxRef.current!.rotation.y = ((now / 1000) * Math.PI) % (Math.PI * 2)
    boxRef.current!.position.y = Math.sin((now / 500) * Math.PI) / 2
  })
  return (
    <>
      <pointLight position={[0, 0, 10]} />
      <mesh
        ref={boxRef}
        position-y={-1000}
        rotation-x={Math.PI * 0.125}
        rotation-y={Math.PI * 0.25}
      >
        <boxBufferGeometry args={[2, 2, 2]} />
        <meshStandardMaterial color={theme.base08} />
      </mesh>
    </>
  )
}

const defaultFileName = 'Drag and drop calibration.json to explore it'

const preprocessors: { [key: string]: typeof preprocessFixedTimeStep } = {
  none: (v) => v,
  'Fixed TimeStep': preprocessFixedTimeStep,
  'Use Target Time': preprocessUseTargetTime,
  'Use Reference Time': preprocessUseReferenceTime,
}

function Scene({ data: dataIn }: { data: SavedCalibration | null }) {
  const theme = useTheme()
  const [{ translate, rotate, preprocess, use: useAlg }, set] = useControls(
    () => ({
      translate: [0, 0, 0],
      rotate: [0, 0, 0],
      file: { editable: false, value: defaultFileName },
      preprocess: {
        value: 'none',
        options: Object.keys(preprocessors),
      },
      use: {
        value: 'netvr',
        options: ['sc', 'netvr'],
      },
      meanDistance: { editable: false, value: '' },
      stdDev: { editable: false, value: '' },
    }),
  )
  const target = useControls('Target', () => ({
    name: { editable: false, value: dataIn?.target_name + '' },
    color: { editable: false, value: theme.base08 },
  }))
  const reference = useControls('Reference', () => ({
    name: { editable: false, value: dataIn?.reference_name + '' },
    color: { editable: false, value: theme.base0B },
  }))
  useEffect(() => {
    target[1]({ name: dataIn?.target_name + '', color: theme.base08 })
    reference[1]({ name: dataIn?.reference_name + '', color: theme.base0B })
  }, [dataIn, reference, target, theme.base08, theme.base0B])

  const data = useMemo(
    () => (!dataIn ? null : (preprocessors[preprocess] ?? ((v) => v))(dataIn)),
    [dataIn, preprocess],
  )

  const wasm = useWasmSuspending()
  const recomputed = useMemo(() => {
    if (!wasm || !data) return
    try {
      const string = JSON.stringify(data)
      console.time('computeCalibration')
      const string2 = compute(string)
      console.timeEnd('computeCalibration')
      const res2: ReturnType<typeof computeCalibration> | { error: any } =
        JSON.parse(string2)

      if ('error' in res2) throw new Error(res2.error)
      if (useAlg === 'netvr') {
        const q = (res2 as any).rotateq
        const rotate = new THREE.Euler()
          .setFromQuaternion(new THREE.Quaternion(q.x, q.y, q.z, q.w))
          .toArray()
          .slice(0, 3) as any
        console.log(rotate)
        return {
          translate: res2.translate,
          rotate,
        }
      }
    } catch (e) {
      console.error(e)
    }
    console.log('Using old')
    console.time('computeCalibration')
    const res = computeCalibration(wasm, data)
    console.log('res', res)
    console.timeEnd('computeCalibration')
    return res
  }, [data, useAlg, wasm])

  useEffect(() => {
    set({ file: data?.fileName ?? defaultFileName })
  }, [data?.fileName, set])

  useEffect(() => {
    if (recomputed) set(recomputed)
  }, [recomputed, set])

  const transformedSamples = useMemo(() => {
    if (!data) return null
    const rotateThree = new THREE.Quaternion()
    rotateThree.setFromEuler(new THREE.Euler(...multiply(rotate, 1), 'XYZ'))

    return {
      reference: data.reference
        //.slice(0, data.target.length)
        .map((v) => vecToArray(v.pose.position)),
      target: data.target
        //.slice(0, data.reference.length)
        .map((v) => {
          let position = vecToArray(v.pose.position)
          // apply offset+angle
          position = vecToArray(
            new THREE.Vector3(
              position[0],
              position[1],
              position[2],
            ).applyQuaternion(rotateThree),
          )
          position = plus(
            [position[0], position[1], position[2]],
            [translate[0], translate[1], translate[2]],
          )

          return position
        }),
    }
  }, [data, rotate, translate])

  useEffect(() => {
    if (!transformedSamples) return
    //const timeout = setTimeout(() => {
    const dists = transformedSamples.reference
      .slice(0, transformedSamples.target.length)
      .map((reference, i) => dist(reference, transformedSamples.target[i]))
    const mean = dists.reduce((a, b) => a + b, 0) / dists.length
    const variance =
      dists.reduce((acc, dist) => acc + (mean - dist) * (mean - dist), 0) /
      dists.length
    const stdDev = Math.sqrt(variance)
    set({
      stdDev:
        (stdDev * 1000).toFixed(1) +
        'mm = ' +
        ((stdDev / mean) * 100).toFixed(0) +
        '%',
      meanDistance: (mean * 1000).toFixed(1) + 'mm',
    })
    //}, 15)
    //return () => void clearTimeout(timeout)
  }, [set, transformedSamples])

  const { timePointsTarget, timePointsReference } = useMemo(
    () => ({
      timePointsTarget: data?.target.map(
        (s, i, list) =>
          [
            0,
            (i === 0 ? 0 : s.nanos - list[i - 1].nanos) / 1000_000_000 + 0.05,
            s.nanos / 1000_000_000,
          ] as const,
      ),
      timePointsReference: data?.reference.map(
        (s, i, list) =>
          [
            0,
            (i === 0 ? 0 : s.nanos - list[i - 1].nanos) / 1000_000_000,
            s.nanos / 1000_000_000,
          ] as const,
      ),
    }),
    [data],
  )
  useEffect(() => {
    if (!timePointsTarget || !timePointsReference) return
    const averageTarget =
      timePointsTarget?.map((v) => v[1]).reduce((a, b) => a + b, 0) /
      timePointsTarget?.length
    const averageReference =
      timePointsReference?.map((v) => v[1]).reduce((a, b) => a + b, 0) /
      timePointsReference?.length
    console.log({ averageTarget, averageReference })
  }, [timePointsTarget, timePointsReference])

  const mov = transformedSamples?.reference[0]
  return (
    <group position={mov ? [-mov[0], -mov[1], mov[2]] : undefined}>
      <pointLight position={[10, 10, 10]} />
      <ambientLight />
      <OrbitControls />
      {transformedSamples ? (
        <>
          <PolyLine
            points={transformedSamples.reference}
            color={theme.base08}
            thickness={0.002}
          />
          <PolyLine
            points={transformedSamples.target}
            color={theme.base0B}
            thickness={0.002}
          />

          <Connections
            points1={transformedSamples.reference}
            points2={transformedSamples.target}
            color={theme.base03}
          />
        </>
      ) : null}
      {timePointsReference && timePointsTarget ? (
        <>
          <PolyLine
            points={timePointsReference}
            color={theme.base08}
            thickness={0.002}
          />
          <PolyLine
            points={timePointsTarget}
            color={theme.base0B}
            thickness={0.002}
          />
          <Connections
            points1={timePointsReference}
            points2={timePointsTarget}
            color={theme.base03}
          />
        </>
      ) : null}
      <Segment from={[0, 0, 0]} to={[1, 0, 0]} color="red" thickness={0.004} />
      <Segment
        from={[0, 0, 0]}
        to={[0, 1, 0]}
        color="green"
        thickness={0.004}
      />
      <Segment from={[0, 0, 0]} to={[0, 0, 1]} color="blue" thickness={0.004} />
    </group>
  )
}

function Segment({
  from,
  to,
  color,
  thickness,
}: {
  from: [number, number, number]
  to: [number, number, number]
  color: string
  thickness?: number
}) {
  return <PolyLine points={[from, to]} color={color} thickness={thickness} />
}

function PolyLine(props: {
  points: readonly (readonly [number, number, number])[]
  color: any
  thickness?: number
}) {
  return (
    <ErrorBoundary fallback={null}>
      <PolyLineInner {...props} />
    </ErrorBoundary>
  )
}

function PolyLineInner({
  points,
  color,
  thickness = 0.001,
}: {
  points: readonly (readonly [number, number, number])[]
  color: any
  thickness?: number
}) {
  const ref = useRef<InstancedMesh | undefined>()
  useEffect(() => {
    const mesh = ref.current
    if (!mesh) return
    const temp = new THREE.Object3D()
    for (let id = 0; id < points.length - 1; ++id) {
      temp.position.set(...points[id])
      const d = dist(points[id], points[id + 1])
      temp.scale.set(thickness, thickness, d)
      temp.lookAt(...points[id + 1])
      temp.translateZ(d / 2)
      temp.updateMatrix()
      mesh.setMatrixAt(id, temp.matrix)
    }
    mesh.instanceMatrix.needsUpdate = true
  }, [points, thickness])
  return (
    <instancedMesh
      ref={ref as any}
      args={[undefined, undefined, points.length - 1]}
    >
      <boxGeometry />
      <meshStandardMaterial color={color} />
    </instancedMesh>
  )
}

function Connections({
  points1,
  points2,
  color,
}: {
  points1: readonly (readonly [number, number, number])[]
  points2: readonly (readonly [number, number, number])[]
  color: any
}) {
  const ref = useRef<InstancedMesh | undefined>()
  const count = Math.min(points1.length, points2.length)
  useEffect(() => {
    const mesh = ref.current
    if (!mesh) return
    const temp = new THREE.Object3D()
    for (let id = 0; id < count; ++id) {
      temp.position.set(...points1[id])
      const d = dist(points1[id], points2[id])
      temp.scale.set(0.001, 0.001, d)
      temp.lookAt(...points2[id])
      temp.translateZ(d / 2)
      temp.updateMatrix()
      mesh.setMatrixAt(id, temp.matrix)
    }
    mesh.instanceMatrix.needsUpdate = true
  }, [count, points1, points2])
  return (
    <instancedMesh ref={ref as any} args={[undefined, undefined, count]}>
      <boxGeometry />
      <meshStandardMaterial color={color} />
    </instancedMesh>
  )
}

function dist(
  a: readonly [number, number, number],
  b: readonly [number, number, number],
) {
  const x = a[0] - b[0]
  const y = a[1] - b[1]
  const z = a[2] - b[2]

  return Math.sqrt(x * x + y * y + z * z)
}

function plus(
  a: readonly [number, number, number],
  b: readonly [number, number, number],
): [number, number, number] {
  return [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

function vecToArray(v: {
  x: number
  y: number
  z: number
}): [number, number, number] {
  return [v.x, v.y, v.z]
}

function quatToArray(v: {
  x: number
  y: number
  z: number
  w: number
}): [number, number, number, number] {
  return [v.x, v.y, v.z, v.w]
}

function multiply(
  v: readonly [number, number, number],
  scalar: number,
): [number, number, number] {
  return [v[0] * scalar, v[1] * scalar, v[2] * scalar]
}

function preprocessUseTargetTime(data: SavedCalibration): SavedCalibration {
  const res = {
    ...data,
    reference: [] as Sample[],
    target: [] as Sample[],
  }

  let referenceI = 0,
    targetI = 0
  while (referenceI < data.target.length && targetI < data.reference.length) {
    const targetSample = data.target[referenceI]
    const referenceSample = data.reference[targetI]
    const referenceSample2 = data.reference[targetI + 1]
    if (!referenceSample2) break
    if (targetSample.nanos < referenceSample2.nanos) {
      res.target.push(targetSample)

      res.reference.push(
        mixSamples(targetSample.nanos, referenceSample, referenceSample2),
      )

      referenceI++
    } else {
      targetI++
    }
  }

  return res
}

function preprocessUseReferenceTime(data: SavedCalibration): SavedCalibration {
  const res = preprocessUseTargetTime({
    ...data,
    target: data.reference,
    reference: data.target,
  })

  return {
    ...res,
    reference: res.target,
    target: res.reference,
  }
}

function preprocessFixedTimeStep(data: SavedCalibration): SavedCalibration {
  const timestepAverageTarget =
    (data.target[data.target.length - 1].nanos - data.target[0].nanos) /
    (data.target.length - 1)
  const timestepAverageReference =
    (data.reference[data.reference.length - 1].nanos -
      data.reference[0].nanos) /
    (data.reference.length - 1)
  const timestep = (timestepAverageTarget + timestepAverageReference) / 2
  const res = {
    ...data,
    reference: [] as Sample[],
    target: [] as Sample[],
  }

  let time = Math.max(data.target[0].nanos, data.reference[0].nanos)
  let endTime = Math.min(
    data.target[data.target.length - 1].nanos,
    data.reference[data.reference.length - 1].nanos,
  )
  let referenceI = 0
  let targetI = 0
  while (time < endTime) {
    const referenceSample1 = data.reference[referenceI]
    const referenceSample2 = data.reference[referenceI + 1]
    const targetSample1 = data.target[targetI]
    const targetSample2 = data.target[targetI + 1]
    if (
      !referenceSample1 ||
      !referenceSample2 ||
      !targetSample1 ||
      !targetSample2
    ) {
      break
    }

    if (referenceSample2.nanos < time) {
      referenceI++
      continue
    }
    if (targetSample2.nanos < time) {
      targetI++
      continue
    }

    res.reference.push(mixSamples(time, referenceSample1, referenceSample2))
    res.target.push(mixSamples(time, targetSample1, targetSample2))
    time += timestep
  }

  return res
}

function mixSamples(nanos: number, sample1: Sample, sample2: Sample): Sample {
  const period = sample2.nanos - sample1.nanos
  const timeSinceSample1 = nanos - sample1.nanos
  const portion = timeSinceSample1 / period
  return {
    ...sample1,
    nanos,
    pose: {
      position: lerp(portion, sample1.pose.position, sample2.pose.position),
      orientation: slerp(
        portion,
        sample1.pose.orientation,
        sample2.pose.orientation,
      ),
    },
  }
}

function lerp(
  ratio: number,
  a: { x: number; y: number; z: number },
  b: { x: number; y: number; z: number },
) {
  return {
    x: b.x * ratio + (1 - ratio) * a.x,
    y: b.y * ratio + (1 - ratio) * a.y,
    z: b.z * ratio + (1 - ratio) * a.z,
  }
}

function slerp(
  ratio: number,
  a: { x: number; y: number; z: number; w: number },
  b: { x: number; y: number; z: number; w: number },
): { x: number; y: number; z: number; w: number } {
  const qa = new THREE.Quaternion().set(a.x, a.y, a.z, a.w)
  const qb = new THREE.Quaternion().set(b.x, b.y, b.z, b.w)
  const mixed = qa.slerp(qb, ratio)
  const res = new THREE.Euler().setFromQuaternion(mixed)
  return { x: res.x, y: res.y, z: res.z, w: res.w }
}

function computeCalibration(wasm: WrappedWasm, data: SavedCalibration) {
  const calibration = wasm.calibration()
  try {
    const count = Math.min(data.reference.length, data.target.length)
    for (let i = 0; i < count; ++i) {
      calibration.addPair(
        vecToArray(data.reference[i].pose.position),
        quatToArray(data.reference[i].pose.orientation),
        vecToArray(data.target[i].pose.position),
        quatToArray(data.target[i].pose.orientation),
      )
    }
    const result = calibration.compute()
    return {
      translate: [result.tx, result.ty, result.tz] as const,
      rotate: [result.rex, result.rey, result.rez] as const,
      //vecToArray(
      //  new THREE.Euler().setFromQuaternion(
      //    new THREE.Quaternion(result.rqx, result.rqy, result.rqz, result.rqw),
      //  ),
      //),
    }
  } finally {
    calibration.destroy()
  }
}
