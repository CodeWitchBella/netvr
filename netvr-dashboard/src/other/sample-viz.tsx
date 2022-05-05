/** @jsxImportSource @emotion/react */
import { OrbitControls } from '@react-three/drei'
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
  const [{ translate, rotate }, set] = useControls(() => ({
    translate: [0, 0, 0],
    rotate: [0, 0, 0],
    file: { editable: false, value: 'Drag and drop config.json to explore it' },
    leader: { editable: false, value: theme.base08 },
    follower: { editable: false, value: theme.base0B },
    meanDistance: { editable: false, value: '' },
    stdDev: { editable: false, value: '' },
  }))

  useEffect(() => {
    if (lastCalibration) {
      set({
        translate: Object.values(lastCalibration.resultTranslate),
        rotate: Object.values(lastCalibration.resultRotate),
      })
    }
  }, [lastCalibration, set])

  useEffect(() => {
    set({ follower: theme.base08, leader: theme.base0B })
  }, [set, theme])

  const transformedSamples = useMemo(() => {
    if (!lastCalibration) return null
    const rotateThree = new THREE.Quaternion()
    rotateThree.setFromEuler(new THREE.Euler(...multiply(rotate, 1), 'XYZ'))

    return {
      leader: lastCalibration.leaderSamples
        .slice(0, lastCalibration.followerSamples.length)
        .map((v) => objectToArray(v.position)),
      follower: lastCalibration.followerSamples
        .slice(0, lastCalibration.leaderSamples.length)
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
  }, [lastCalibration, rotate, translate])

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

  const timePointsLeader = lastCalibration?.leaderSamples.map(
    (s, i, list) =>
      [
        0,
        i === 0 ? 0 : s.timestamp - list[i - 1].timestamp,
        s.timestamp,
      ] as const,
  )
  const timePointsFollower = lastCalibration?.followerSamples.map(
    (s, i, list) =>
      [
        0,
        i === 0 ? 0 : s.timestamp - list[i - 1].timestamp + 0.05,
        s.timestamp,
      ] as const,
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
