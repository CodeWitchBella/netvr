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

export default function SampleVizRoute() {
  const [lastCalibration, setLastCalibration] =
    useState<LastCalibration | null>(null)
  const dropzone = useDropzone({
    noClick: true,
    multiple: false,
    onDrop: ([file]) => {
      file.text().then((v) => {
        setLastCalibration(JSON.parse(v))
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
  const [{ offset, angle, unityAxes }, set] = useControls(() => ({
    offset: [0, 0, 0],
    angle: 0,
    unityAxes: true,
    file: { editable: false, value: 'Drag and drop config.json to explore it' },
    leader: { editable: false, value: theme.base08 },
    follower: { editable: false, value: theme.base0B },
    meanDistance: { editable: false, value: '' },
    stdDev: { editable: false, value: '' },
  }))

  useEffect(() => {
    if (lastCalibration) {
      set({
        offset: Object.values(lastCalibration.resultTranslate),
        angle: (lastCalibration.resultRotate.y / Math.PI) * 180,
      })
    }
  }, [lastCalibration, set])

  useEffect(() => {
    set({ follower: theme.base08, leader: theme.base0B })
  }, [set, theme])

  const transformedSamplesStep1 = useMemo(
    () =>
      lastCalibration?.leaderSamples
        .slice(0, lastCalibration.followerSamples.length)
        .map((leader, i, list) => {
          const mov = { x: 0, y: 0, z: 0 } || list[0].position
          const follower = lastCalibration.followerSamples[i]
          return {
            leader: [
              leader.position.x - mov.x,
              leader.position.y - mov.y,
              leader.position.z - mov.z,
            ],
            follower: [
              follower.position.x - mov.x,
              follower.position.y - mov.y,
              follower.position.z - mov.z,
            ],
          } as const
        }) ?? null,
    [lastCalibration],
  )

  const cos = Math.cos((angle / 180) * Math.PI)
  const sin = Math.sin((angle / 180) * Math.PI)

  const transformedSamples = useMemo(() => {
    if (!transformedSamplesStep1) return null
    const mov = transformedSamplesStep1[0].leader
    return transformedSamplesStep1.map(({ leader: a, follower: b }) => {
      // apply offset+angle
      b = [b[0] * cos - b[2] * sin, b[1], b[0] * sin + b[2] * cos]
      b = [b[0] + offset[0], b[1] + offset[1], b[2] + offset[2]]

      // recenter
      a = minus(a, mov)
      b = minus(b, mov)
      return { leader: a, follower: b }
    })
  }, [cos, offset, sin, transformedSamplesStep1])

  useEffect(() => {
    if (!transformedSamples) return
    //const timeout = setTimeout(() => {
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
        (stdDev * 1000).toFixed(1) +
        'mm = ' +
        ((stdDev / mean) * 100).toFixed(0) +
        '%',
      meanDistance: (mean * 1000).toFixed(1) + 'mm',
    })
    //}, 15)
    //return () => void clearTimeout(timeout)
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
            points={transformedSamples.map((v) => unityToGL(v.leader))}
            color={theme.base08}
            lineWidth={1}
            dashed={false}
          />
          {/* @ts-expect-error */}
          <Line
            points={transformedSamples.map((v) => unityToGL(v.follower))}
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
      temp.position.set(...unityToGL(sample.leader))
      const d = dist(sample.leader, sample.follower)
      temp.scale.set(0.001, 0.001, d)
      temp.lookAt(...unityToGL(sample.follower))
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
  a: readonly [number, number, number],
  b: readonly [number, number, number],
) {
  const x = a[0] - b[0]
  const y = a[1] - b[1]
  const z = a[2] - b[2]

  return Math.sqrt(x * x + y * y + z * z)
}

function unityToGL(
  v: readonly [number, number, number],
): [number, number, number] {
  return [v[0], v[1], -v[2]]
}

function minus(
  a: readonly [number, number, number],
  b: readonly [number, number, number],
): [number, number, number] {
  return [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
