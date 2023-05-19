/** @jsxImportSource @emotion/react */
import { useFrame } from '@react-three/fiber'
import { useEffect, useRef } from 'react'
import * as THREE from 'three'
import { useTheme } from '../components/theme'
import { ErrorBoundary } from '../components/error-boundary'
import { InstancedMesh } from 'three'

export function SpinningCube() {
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

export function Segment({
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

export function PolyLine(props: {
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

export function Connections({
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

export function dist(
  a: readonly [number, number, number],
  b: readonly [number, number, number],
) {
  const x = a[0] - b[0]
  const y = a[1] - b[1]
  const z = a[2] - b[2]

  return Math.sqrt(x * x + y * y + z * z)
}
