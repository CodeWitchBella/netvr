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
import { SpinningCube } from './shared'

type FileData = {
  fileName: string
  text: string
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

export default function DelayVizRoute() {
  const [fileData, setFileData] = useState<FileData | null>(null)
  const dropzone = useDropzone({
    noClick: true,
    multiple: false,
    onDrop: ([file]) => {
      file.text().then((v) => {
        setFileData({ fileName: file.name, text: v })
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
            <Help fileName={fileData?.fileName} />
            {fileData ? <Scene data={fileData} /> : null}
          </Suspense>
        </ReprovideTheme>
      </Canvas>
    </div>
  )
}

function Help({ fileName }: { fileName?: string }) {
  const [, set] = useControls(() => ({
    file: { editable: false, value: defaultFileName },
  }))

  useEffect(() => {
    set({ file: fileName ?? defaultFileName })
  }, [fileName, set])
  return null
}

function parseV3(line: string) {
  const [x, y, z] = line
    .substring(1, line.length - 1)
    .split(', ')
    .map(parseFloat)
  return [+x, +y, +z] as const
}
function parseV4(line: string) {
  const [x, y, z, w] = line
    .substring(1, line.length - 1)
    .split(', ')
    .map(parseFloat)
  return [+x, +y, +z, +w] as const
}

function parseLine(lineIn: string) {
  const line = lineIn.split('\t').reverse()
  const time = line.pop()!.replace(',', '.')
  const local: {
    id: number
    characteristics: string
    position: readonly [number, number, number]
    orientation: readonly [number, number, number, number]
    rotation: readonly [number, number, number]
  }[] = []
  const remote: {
    id: number
    interactionProfile: string
    subactionPath: string
    position: readonly [number, number, number]
    orientation: readonly [number, number, number, number]
    rotation: readonly [number, number, number]
  }[] = []
  while (line.length) {
    const type = line.pop()
    if (type === 'local') {
      local.push({
        id: parseInt(line.pop()!),
        characteristics: line.pop()!,
        position: parseV3(line.pop()!),
        orientation: parseV4(line.pop()!),
        rotation: parseV3(line.pop()!),
      })
    } else if (type === 'remote') {
      remote.push({
        id: parseInt(line.pop()!),
        interactionProfile: line.pop()!,
        subactionPath: line.pop()!,
        position: parseV3(line.pop()!),
        orientation: parseV4(line.pop()!),
        rotation: parseV3(line.pop()!),
      })
    } else {
      console.info(type, line)
      throw new Error('Parser failed')
    }
  }
  return { time, local, remote }
}

function distanceTraveled(
  local: readonly (readonly [number, number, number])[],
) {
  let acc = 0
  let prev = local[0]
  for (let i = 1; i < local.length; ++i) {
    acc += dist(prev, local[i])
    prev = local[i]
  }
  return acc
}

function choosePair(data: readonly ReturnType<typeof parseLine>[]): {
  local: readonly (readonly [number, number, number])[]
  remote: readonly (readonly [number, number, number])[]
} {
  const local: (readonly [number, number, number])[][] = []
  const remote: (readonly [number, number, number])[][] = []
  for (const device of data[0].local) {
    if (!data.every((v) => v.local.some((v) => v.id === device.id))) continue
    local.push(
      data.map((v) => v.local.find((v) => v.id === device.id)!.position),
    )
  }
  for (const device of data[0].remote) {
    if (!data.every((v) => v.remote.some((v) => v.id === device.id))) continue
    remote.push(
      data.map((v) => v.remote.find((v) => v.id === device.id)!.position),
    )
  }

  let localDistanceMax = distanceTraveled(local[0])
  let bestLocal = local[0]
  for (const v of local) {
    const d = distanceTraveled(v)
    if (d > localDistanceMax) {
      localDistanceMax = d
      bestLocal = v
    }
  }
  let remoteDistanceMax = distanceTraveled(remote[0])
  let bestRemote = remote[0]
  for (const v of remote) {
    const d = distanceTraveled(v)
    if (d > remoteDistanceMax) {
      remoteDistanceMax = d
      bestRemote = v
    }
  }
  return { local: bestLocal, remote: bestRemote }
}

const defaultFileName = 'Drag and drop data.txt to explore it'

function Scene({ data: dataIn }: { data: FileData }) {
  const theme = useTheme()

  const data = useMemo(
    () => choosePair(dataIn.text.trim().split('\n').map(parseLine)),
    [dataIn],
  )

  const mov = data.local[0]
  return (
    <group position={mov ? [-mov[0], -mov[1], -mov[2]] : undefined}>
      <pointLight position={[10, 10, 10]} />
      <ambientLight />
      <OrbitControls />

      <PolyLine points={data.local} color={theme.base08} thickness={0.002} />
      <PolyLine points={data.remote} color={theme.base0B} thickness={0.002} />

      <Connections
        points1={data.local}
        points2={data.remote}
        color={theme.base03}
      />

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
