/** @jsxImportSource @emotion/react */
import { OrbitControls } from '@react-three/drei'
import { Canvas } from '@react-three/fiber'
import { useControls } from 'leva'
import { Suspense, useEffect, useMemo, useState } from 'react'
import { useDropzone } from 'react-dropzone'
import {
  ReprovideTheme,
  useReprovideTheme,
  useTheme,
} from '../components/theme'
import {
  Axes,
  CameraControls,
  Connections,
  PolyLine,
  SpinningCube,
  dist,
  mul,
  plus,
} from './shared'

type FileData = {
  fileName: string
  text: string
}

/**
 * Vizualizes the delay data from .txt file. Data for this route is generated
 * by Logger.cs.
 */
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
  const theme = useTheme()
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
      <Canvas style={{ height: 'auto', background: theme.base00 }}>
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
  const theme = useTheme()
  const [, set] = useControls(() => ({
    file: { editable: false, value: defaultFileName },
    local: { editable: false, value: theme.base08 },
    remote: { editable: false, value: theme.base0B },
  }))

  useEffect(() => {
    set({
      file: fileName ?? defaultFileName,
      local: theme.base08,
      remote: theme.base0B,
    })
  }, [fileName, set, theme.base08, theme.base0B])
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

  const all = [...data.local, ...data.remote]
  const sampleMin = [
    all.reduce((acc, v) => Math.min(acc, v[0]), Infinity),
    all.reduce((acc, v) => Math.min(acc, v[1]), Infinity),
    all.reduce((acc, v) => Math.min(acc, v[2]), Infinity),
  ] as const
  const sampleMax = [
    all.reduce((acc, v) => Math.max(acc, v[0]), -Infinity),
    all.reduce((acc, v) => Math.max(acc, v[1]), -Infinity),
    all.reduce((acc, v) => Math.max(acc, v[2]), -Infinity),
  ] as const

  const mov = [
    (sampleMin[0] + sampleMax[0]) / 2,
    (sampleMin[1] + sampleMax[1]) / 2,
    (sampleMin[2] + sampleMax[2]) / 2,
  ] as const

  const center =
    sampleMin && sampleMax ? mul(0.5, plus(sampleMin, sampleMax)) : undefined

  const axisMov = plus(sampleMin, [-0.1, -0.1, -0.1])
  return (
    <group position={mul(-1, center)}>
      <pointLight position={[10, 10, 10]} />
      <ambientLight />
      <OrbitControls />
      <CameraControls />

      <PolyLine points={data.local} color={theme.base08} thickness={0.002} />
      <PolyLine points={data.remote} color={theme.base0B} thickness={0.002} />

      <Connections
        points1={data.local}
        points2={data.remote}
        color={theme.base03}
      />

      <Axes pos={axisMov} />
    </group>
  )
}
