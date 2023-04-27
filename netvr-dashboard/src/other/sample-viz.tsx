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

type SavedCalibration = {
  fileName: string
  followerDevice: number
  leaderDevice: number
  follower: number
  leader: number
  resultRotate: { x: number; y: number; z: number }
  resultTranslate: { x: number; y: number; z: number }
  leaderSamples: readonly Sample[]
  followerSamples: readonly Sample[]
}

type Sample = {
  position: { x: number; y: number; z: number }
  rotation: { x: number; y: number; z: number }
  timestamp: number
}

type NewSample = {
  pose: {
    position: { x: number; y: number; z: number }
    orientation: { x: number; y: number; z: number; w: number }
  }
}

function convert(
  data:
    | SavedCalibration
    | { fileName: string; target: NewSample[]; reference: NewSample[] },
): SavedCalibration {
  if (!('reference' in data)) return data
  return {
    fileName: data.fileName,
    follower: 0,
    followerDevice: 0,
    followerSamples: data.target.map(convertSample),
    leader: 0,
    leaderDevice: 0,
    leaderSamples: data.reference.map(convertSample),
    resultRotate: { x: 0, y: 0, z: 0 },
    resultTranslate: { x: 0, y: 0, z: 0 },
  }
}

function convertSample(v: NewSample) {
  const rotation = new THREE.Euler()
    .setFromQuaternion(
      new THREE.Quaternion().set(
        v.pose.orientation.x,
        v.pose.orientation.y,
        v.pose.orientation.z,
        v.pose.orientation.w,
      ),
    )
    .toArray()
  return {
    position: v.pose.position,
    rotation: {
      x: rotation[0],
      y: rotation[1],
      z: rotation[2],
    },
    timestamp: 0,
  }
}

export default function SampleVizRoute() {
  const [savedCalibration, setSavedCalibration] =
    useState<SavedCalibration | null>(null)
  const dropzone = useDropzone({
    noClick: true,
    multiple: false,
    onDrop: ([file]) => {
      file.text().then((v) => {
        setSavedCalibration(convert({ fileName: file.name, ...JSON.parse(v) }))
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
  'Use Follower Time': preprocessUseFollowerTime,
  'Use Leader Time': preprocessUseLeaderTime,
}

function Scene({ data: dataIn }: { data: SavedCalibration | null }) {
  const theme = useTheme()
  const [{ translate, rotate, preprocess }, set] = useControls(() => ({
    translate: [0, 0, 0],
    rotate: [0, 0, 0],
    file: { editable: false, value: defaultFileName },
    preprocess: {
      value: 'none',
      options: Object.keys(preprocessors),
    },
    leader: { editable: false, value: theme.base08 },
    follower: { editable: false, value: theme.base0B },
    meanDistance: { editable: false, value: '' },
    stdDev: { editable: false, value: '' },
  }))

  const data = useMemo(
    () => (!dataIn ? null : (preprocessors[preprocess] ?? ((v) => v))(dataIn)),
    [dataIn, preprocess],
  )

  const wasm = useWasmSuspending()
  const recomputed = useMemo(() => {
    if (!wasm || !data) return
    return computeCalibration(wasm, data)
  }, [data, wasm])

  useEffect(() => {
    set({ file: data?.fileName ?? defaultFileName })
  }, [data?.fileName, set])

  useEffect(() => {
    if (recomputed) set(recomputed)
  }, [recomputed, set])

  useEffect(() => {
    set({ follower: theme.base0B, leader: theme.base08 })
  }, [set, theme])

  const transformedSamples = useMemo(() => {
    if (!data) return null
    const rotateThree = new THREE.Quaternion()
    rotateThree.setFromEuler(new THREE.Euler(...multiply(rotate, 1), 'XYZ'))

    return {
      leader: data.leaderSamples
        .slice(0, data.followerSamples.length)
        .map((v) => objectToArray(v.position)),
      follower: data.followerSamples
        .slice(0, data.leaderSamples.length)
        .map((v) => {
          let position = objectToArray(v.position)
          // apply offset+angle
          position = objectToArray(
            new THREE.Vector3(
              position[0],
              position[1],
              position[2],
            ).applyQuaternion(rotateThree),
          )
          position = plus(position, translate)

          return position
        }),
    }
  }, [data, rotate, translate])

  useEffect(() => {
    if (!transformedSamples) return
    //const timeout = setTimeout(() => {
    const dists = transformedSamples.leader.map((leader, i) =>
      dist(leader, transformedSamples.follower[i]),
    )
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

  const { timePointsFollower, timePointsLeader } = useMemo(
    () => ({
      timePointsFollower: data?.followerSamples.map(
        (s, i, list) =>
          [
            0,
            i === 0 ? 0 : s.timestamp - list[i - 1].timestamp + 0.05,
            s.timestamp,
          ] as const,
      ),
      timePointsLeader: data?.leaderSamples.map(
        (s, i, list) =>
          [
            0,
            i === 0 ? 0 : s.timestamp - list[i - 1].timestamp,
            s.timestamp,
          ] as const,
      ),
    }),
    [data],
  )

  const mov = transformedSamples?.leader[0]
  return (
    <group
      scale={[1, 1, -1]}
      position={mov ? [-mov[0], -mov[1], mov[2]] : undefined}
    >
      <pointLight position={[10, 10, 10]} />
      <ambientLight />
      <OrbitControls />
      {transformedSamples ? (
        <>
          <PolyLine
            points={transformedSamples.leader}
            color={theme.base08}
            thickness={0.002}
          />
          <PolyLine
            points={transformedSamples.follower}
            color={theme.base0B}
            thickness={0.002}
          />

          <Connections
            points1={transformedSamples.leader}
            points2={transformedSamples.follower}
            color={theme.base03}
          />
        </>
      ) : null}
      {timePointsLeader && timePointsFollower ? (
        <>
          <PolyLine
            points={timePointsLeader}
            color={theme.base08}
            thickness={0.002}
          />
          <PolyLine
            points={timePointsFollower}
            color={theme.base0B}
            thickness={0.002}
          />
          <Connections
            points1={timePointsLeader}
            points2={timePointsFollower}
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

function PolyLine({
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

function objectToArray(v: {
  x: number
  y: number
  z: number
}): [number, number, number] {
  return [v.x, v.y, v.z]
}

function multiply(
  v: readonly [number, number, number],
  scalar: number,
): [number, number, number] {
  return [v[0] * scalar, v[1] * scalar, v[2] * scalar]
}

/**
 * Converts Vector3 from Unity's left-handed coordinate system to OVR's
 * right-handed coordinates.
 */
function createOVRVector3FromUnity({
  x,
  y,
  z,
}: {
  x: number
  y: number
  z: number
}): [x: number, y: number, z: number] {
  return [x, y, -z]
}

function createOVRQuaternionFromUnityEuler(rotation: {
  x: number
  y: number
  z: number
}): readonly [x: number, y: number, z: number, w: number] {
  const quaternion = new THREE.Quaternion().setFromEuler(
    new THREE.Euler(rotation.y, rotation.x, -rotation.z, 'XYZ'),
  )

  return [-quaternion.y, -quaternion.x, -quaternion.z, quaternion.w]
}

function preprocessUseFollowerTime(data: SavedCalibration): SavedCalibration {
  const res = {
    ...data,
    leaderSamples: [] as Sample[],
    followerSamples: [] as Sample[],
  }

  let leaderI = 0,
    followerI = 0
  while (
    leaderI < data.followerSamples.length &&
    followerI < data.leaderSamples.length
  ) {
    const followerSample = data.followerSamples[leaderI]
    const leaderSample = data.leaderSamples[followerI]
    const leaderSample2 = data.leaderSamples[followerI + 1]
    if (!leaderSample2) break
    if (followerSample.timestamp < leaderSample2.timestamp) {
      res.followerSamples.push(followerSample)

      res.leaderSamples.push(
        mixSamples(followerSample.timestamp, leaderSample, leaderSample2),
      )

      leaderI++
    } else {
      followerI++
    }
  }

  return res
}

function preprocessUseLeaderTime(data: SavedCalibration): SavedCalibration {
  const res = preprocessUseFollowerTime({
    ...data,
    leaderSamples: data.followerSamples,
    followerSamples: data.leaderSamples,
  })

  return {
    ...res,
    leaderSamples: res.followerSamples,
    followerSamples: res.leaderSamples,
  }
}

function preprocessFixedTimeStep(data: SavedCalibration): SavedCalibration {
  const timestepAverageFollower =
    (data.followerSamples[data.followerSamples.length - 1].timestamp -
      data.followerSamples[0].timestamp) /
    (data.followerSamples.length - 1)
  const timestepAverageLeader =
    (data.leaderSamples[data.leaderSamples.length - 1].timestamp -
      data.leaderSamples[0].timestamp) /
    (data.leaderSamples.length - 1)
  const timestep = (timestepAverageFollower + timestepAverageLeader) / 2
  const res = {
    ...data,
    leaderSamples: [] as Sample[],
    followerSamples: [] as Sample[],
  }

  let time = Math.max(
    data.followerSamples[0].timestamp,
    data.leaderSamples[0].timestamp,
  )
  let endTime = Math.min(
    data.followerSamples[data.followerSamples.length - 1].timestamp,
    data.leaderSamples[data.leaderSamples.length - 1].timestamp,
  )
  let leaderI = 0
  let followerI = 0
  while (time < endTime) {
    const leaderSample1 = data.leaderSamples[leaderI]
    const leaderSample2 = data.leaderSamples[leaderI + 1]
    const followerSample1 = data.followerSamples[followerI]
    const followerSample2 = data.followerSamples[followerI + 1]
    if (
      !leaderSample1 ||
      !leaderSample2 ||
      !followerSample1 ||
      !followerSample2
    ) {
      break
    }

    if (leaderSample2.timestamp < time) {
      leaderI++
      continue
    }
    if (followerSample2.timestamp < time) {
      followerI++
      continue
    }

    res.leaderSamples.push(mixSamples(time, leaderSample1, leaderSample2))
    res.followerSamples.push(mixSamples(time, followerSample1, followerSample2))
    time += timestep
  }

  return res
}

function mixSamples(time: number, sample1: Sample, sample2: Sample): Sample {
  const period = sample2.timestamp - sample1.timestamp
  const timeSinceSample1 = time - sample1.timestamp
  const portion = timeSinceSample1 / period
  return {
    timestamp: time,
    position: lerp(portion, sample1.position, sample2.position),
    rotation: slerp(portion, sample1.rotation, sample2.rotation),
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
  a: { x: number; y: number; z: number },
  b: { x: number; y: number; z: number },
) {
  const qa = new THREE.Quaternion().setFromEuler(
    new THREE.Euler(a.y, a.x, -a.z, 'XYZ'),
  )
  const qb = new THREE.Quaternion().setFromEuler(
    new THREE.Euler(b.y, b.x, -b.z, 'XYZ'),
  )
  const mixed = qa.slerp(qb, ratio)
  const res = new THREE.Euler().setFromQuaternion(mixed)
  return { x: res.y, y: res.x, z: -res.z }
}

function computeCalibration(wasm: WrappedWasm, data: SavedCalibration) {
  const calibration = wasm.calibration()
  try {
    const count = Math.min(
      data.leaderSamples.length,
      data.followerSamples.length,
    )
    for (let i = 0; i < count; ++i) {
      calibration.addPair(
        createOVRVector3FromUnity(data.leaderSamples[i].position),
        createOVRQuaternionFromUnityEuler(data.leaderSamples[i].rotation),
        createOVRVector3FromUnity(data.followerSamples[i].position),
        createOVRQuaternionFromUnityEuler(data.followerSamples[i].rotation),
      )
    }
    const result = calibration.compute()
    return {
      translate: [result.tx, result.ty, -result.tz] as const,
      rotate: objectToArray(
        new THREE.Euler().setFromQuaternion(
          new THREE.Quaternion(
            -result.rqx,
            -result.rqy,
            result.rqz,
            result.rqw,
          ),
        ),
      ),
    }
  } finally {
    calibration.destroy()
  }
}
