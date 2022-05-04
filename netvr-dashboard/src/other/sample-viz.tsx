/** @jsxImportSource @emotion/react */
import { Line, OrbitControls } from '@react-three/drei'
import { Canvas } from '@react-three/fiber'
import { useControls } from 'leva'
import { useEffect, useMemo, useRef, useState } from 'react'
import { useDropzone } from 'react-dropzone'
import * as THREE from 'three'
import { InstancedMesh } from 'three'
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
  const { offset, angle, unityAxes } = useControls({
    offset: [-0.9754, 0.1466, -1.323],
    angle: 116.15,
    unityAxes: true,
    file: { editable: false, value: 'Drag and drop config.json to explore it' },
  })

  const [, set] = useControls(() => ({
    leader: { editable: false, value: theme.base08 },
    follower: { editable: false, value: theme.base0B },
    meanDistance: { editable: false, value: '' },
    stdDev: { editable: false, value: '' },
  }))

  useEffect(() => {
    set({ follower: theme.base08, leader: theme.base0B })
  }, [set, theme])

  const transformedSamplesStep1 = useMemo(
    () =>
      lastCalibration?.samples.map((sample, _, list) => {
        return {
          leader: [
            sample.leader.position.x - list[0].leader.position.x,
            sample.leader.position.y - list[0].leader.position.y,
            -1 * (sample.leader.position.z - list[0].leader.position.z),
          ],
          follower: [
            sample.follower.position.x - list[0].leader.position.x,
            sample.follower.position.y - list[0].leader.position.y,
            -1 * (sample.follower.position.z - list[0].leader.position.z),
          ],
        } as const
      }) ?? null,
    [lastCalibration?.samples],
  )

  const cos = Math.cos((angle / 180) * Math.PI)
  const sin = Math.sin((angle / 180) * Math.PI)

  const transformedSamples = useMemo(
    () =>
      transformedSamplesStep1?.map(({ leader: a, follower: b }) => {
        b = [b[0] * cos - b[2] * sin, b[1], b[0] * sin + b[2] * cos]
        b = [b[0] + offset[0], b[1] + offset[1], b[2] - offset[2]]
        return { leader: a, follower: b }
      }) ?? null,
    [cos, offset, sin, transformedSamplesStep1],
  )

  useEffect(() => {
    if (!transformedSamples) return
    const timeout = setTimeout(() => {
      const dists = transformedSamples.map(({ leader, follower }) =>
        dist(leader, follower),
      )
      const mean = dists.reduce((a, b) => a + b, 0) / dists.length
      const variance =
        dists.reduce((acc, dist) => acc + (mean - dist) * (mean - dist), 0) /
        dists.length
      const stdDev = Math.sqrt(variance)
      set({
        stdDev:
          stdDev.toFixed(8) + ' = ' + ((stdDev / mean) * 100).toFixed(0) + '%',
        meanDistance: mean.toFixed(4),
      })
    }, 15)
    return () => void clearTimeout(timeout)
  }, [set, transformedSamples])

  return (
    <>
      <OrbitControls />
      <ambientLight />
      <pointLight position={[10, 10, 10]} />
      {transformedSamples ? (
        <>
          {/* @ts-expect-error */}
          <Line
            points={transformedSamples.map((v) => v.leader)}
            color={theme.base08}
            lineWidth={1}
            dashed={false}
          />
          {/* @ts-expect-error */}
          <Line
            points={transformedSamples.map((v) => v.follower)}
            color={theme.base0B}
            lineWidth={1}
            dashed={false}
          />
          <group position={[-0.25, -0.25, unityAxes ? 0.25 : -0.25]}>
            {/* @ts-expect-error */}
            <Line
              points={[
                [0, 0, 0],
                [1, 0, 0],
              ]}
              color="red"
              lineWidth={0.5}
              dashed={false}
            />
            {/* @ts-expect-error */}
            <Line
              points={[
                [0, 0, 0],
                [0, 1, 0],
              ]}
              color="green"
              lineWidth={0.5}
              dashed={false}
            />
            {/* @ts-expect-error */}
            <Line
              points={[
                [0, 0, 0],
                [0, 0, unityAxes ? -1 : 1],
              ]}
              color="blue"
              lineWidth={0.5}
              dashed={false}
            />
          </group>
          <Connections samples={transformedSamples} color={theme.base03} />
        </>
      ) : null}
      {/* <mesh>
  <boxGeometry args={[1, 1, 1]} />
  <meshStandardMaterial color="hotpink" />
</mesh> */}
    </>
  )
}

function Connections({
  samples,
  color,
}: {
  samples: readonly {
    leader: readonly [number, number, number]
    follower: readonly [number, number, number]
  }[]
  color: any
}) {
  const ref = useRef<InstancedMesh | undefined>()
  useEffect(() => {
    const mesh = ref.current
    if (!mesh) return
    const temp = new THREE.Object3D()
    let id = 0
    for (const sample of samples) {
      temp.position.set(...sample.leader)
      const d = dist(sample.leader, sample.follower)
      temp.scale.set(0.001, 0.001, d)
      temp.lookAt(...sample.follower)
      temp.translateZ(d / 2)
      temp.updateMatrix()
      mesh.setMatrixAt(id++, temp.matrix)
    }
    mesh.instanceMatrix.needsUpdate = true
  }, [samples])
  return (
    <instancedMesh
      ref={ref as any}
      args={[undefined, undefined, samples.length]}
    >
      <boxGeometry />
      <meshStandardMaterial color={color} />
    </instancedMesh>
  )
}

function dist(
  leader: readonly [number, number, number],
  follower: readonly [number, number, number],
) {
  const x = leader[0] - follower[0]
  const y = leader[1] - follower[1]
  const z = leader[2] - follower[2]

  return Math.sqrt(x * x + y * y + z * z)
}
