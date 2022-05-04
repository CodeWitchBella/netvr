/** @jsxImportSource @emotion/react */
import { Line, OrbitControls } from '@react-three/drei'
import { Canvas } from '@react-three/fiber'
import { useControls } from 'leva'
import { useEffect, useMemo, useState } from 'react'
import { useDropzone } from 'react-dropzone'
import {
  ReprovideTheme,
  useReprovideTheme,
  useTheme,
} from '../components/theme'

type LastCalibration = {
  followerDevice: number
  leaderDevice: number
  follower: number
  leader: number
  samples: readonly SamplePairData[]
}

type SamplePairData = {
  leader: Sample
  follower: Sample
}

type Sample = {
  position: { x: number; y: number; z: number }
  rotation: { x: number; y: number; z: number }
  timestamp: number
}

export default function SampleVizRoute() {
  const [lastCalibration, setLastCalibration] =
    useState<LastCalibration | null>(null)
  const dropzone = useDropzone({
    noClick: true,
    multiple: false,
    onDrop: ([file]) => {
      file.text().then((v) => {
        setLastCalibration(JSON.parse(v).lastCalibration)
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
          <Scene lastCalibration={lastCalibration} />
        </ReprovideTheme>
      </Canvas>
    </div>
  )
}

function Scene({
  lastCalibration,
}: {
  lastCalibration: LastCalibration | null
}) {
  const theme = useTheme()
  const { offset, angle } = useControls({
    offset: [0.9869, 0.1152, 0.2842],
    angle: -116.15,
    file: { editable: false, value: 'Drag and drop config.json to explore it' },
  })

  const [, set] = useControls(() => ({
    leader: { editable: false, value: theme.base08 },
    follower: { editable: false, value: theme.base0B },
    stdDev: { editable: false, value: '' },
  }))

  useEffect(() => {
    set({ follower: theme.base08, leader: theme.base0B })
  }, [set, theme])

  const transformedSamplesStep1 = useMemo(
    () =>
      lastCalibration?.samples.map((sample) => {
        const a: [number, number, number] = Object.values(
          sample.leader.position,
        ) as any
        const b: [number, number, number] = Object.values(
          sample.follower.position,
        ) as any
        return { leader: a, follower: b }
      }) ?? null,
    [lastCalibration?.samples],
  )

  const cos = Math.cos((angle / 180) * Math.PI)
  const sin = Math.sin((angle / 180) * Math.PI)

  const transformedSamples = useMemo(
    () =>
      transformedSamplesStep1?.map(({ leader: a, follower: b }) => {
        b = [b[0] * cos - b[2] * sin, b[1], b[0] * sin + b[2] * cos]
        b = [b[0] + offset[0], b[1] + offset[1], b[2] + offset[2]]
        return { leader: a, follower: b }
      }) ?? null,
    [cos, offset, sin, transformedSamplesStep1],
  )

  useEffect(() => {
    if (!transformedSamples) return
    const timeout = setTimeout(() => {
      const dists = transformedSamples.map(({ leader, follower }) => {
        const x = leader[0] - follower[0]
        const y = leader[1] - follower[1]
        const z = leader[2] - follower[2]

        return Math.sqrt(x * x + y * y + z * z)
      })
      const mean = dists.reduce((a, b) => a + b, 0) / dists.length
      const variance =
        dists.reduce((acc, dist) => acc + (mean - dist) * (mean - dist), 0) /
        dists.length
      const stdDev = Math.sqrt(variance)
      set({ stdDev: stdDev.toFixed(8) })
    }, 15)
    return () => void clearTimeout(timeout)
  }, [set, transformedSamples])

  return (
    <>
      <OrbitControls />
      <ambientLight />
      <pointLight position={[10, 10, 10]} />
      {transformedSamples
        ? transformedSamples.map((s, i) => <SamplePair key={i} sample={s} />)
        : null}
      {/* <mesh>
  <boxGeometry args={[1, 1, 1]} />
  <meshStandardMaterial color="hotpink" />
</mesh> */}
    </>
  )
}

function SamplePair({
  sample,
}: {
  sample: {
    leader: readonly [number, number, number]
    follower: readonly [number, number, number]
  }
}) {
  const theme = useTheme()
  // Return the view, these are regular Threejs elements expressed in JSX

  return (
    <>
      <mesh position={sample.leader}>
        <boxGeometry args={[0.01, 0.01, 0.01]} />
        <meshStandardMaterial color={theme.base08} />
      </mesh>
      {/* @ts-expect-error */}
      <Line
        points={[sample.leader, sample.follower]}
        color={theme.base03}
        lineWidth={1}
        dashed={false}
      />
      <mesh position={sample.follower}>
        <boxGeometry args={[0.01, 0.01, 0.01]} />
        <meshStandardMaterial color={theme.base0B} />
      </mesh>
    </>
  )
}