/** @jsxImportSource @emotion/react */
import { useFrame } from '@react-three/fiber'
import { Text, useCamera } from '@react-three/drei'
import { Fragment, useEffect, useRef } from 'react'
import * as THREE from 'three'
import { useTheme } from '../components/theme'
import { ErrorBoundary } from '../components/error-boundary'
import { InstancedMesh } from 'three'
import { button, useControls } from 'leva'

/**
 * Model of a spinning cube to be displayed as a loading indicator.
 */
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

/**
 * Line from point A to point B.
 */
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

/**
 * A string of lines. More efficient than using multiple <Segment />s.
 */
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

/**
 * Axis of a coordinate system. Also draws ticks and distance labels.
 */
export function Axes({ pos }: { pos?: readonly [number, number, number] }) {
  const tickLength = 0.05
  return (
    <group position={pos ? [pos[0], pos[1], pos[2]] : undefined}>
      <Segment from={[0, 0, 0]} to={[1, 0, 0]} color="red" thickness={0.004} />
      <AxisTicks
        color="red"
        dir={[1, 0, 0]}
        end={[0, tickLength, 0]}
        text={{
          anchorX: 'left',
          anchorY: 'middle',
          //rotation: [0, Math.PI / 2, 0],
          position: [0.005, 0.025, 0],
          desc: 'x (m)',
        }}
      />
      <Segment
        from={[0, 0, 0]}
        to={[0, 1, 0]}
        color="green"
        thickness={0.004}
      />
      <AxisTicks
        color="green"
        dir={[0, 1, 0]}
        end={[0, 0, tickLength]}
        text={{
          anchorX: 'right',
          anchorY: 'bottom-baseline',
          rotation: [0, Math.PI / 2, 0],
          position: [-0.005, 0.005, 0],
        }}
      />
      <AxisTicks
        color="green"
        dir={[0, 1, 0]}
        end={[tickLength, 0, 0]}
        text={{
          anchorX: 'left',
          anchorY: 'bottom-baseline',
          position: [0.005, 0.005, 0],
        }}
      />
      <Segment from={[0, 0, 0]} to={[0, 0, 1]} color="blue" thickness={0.004} />
      <AxisTicks
        color="blue"
        dir={[0, 0, 1]}
        end={[0, tickLength, 0]}
        text={{
          anchorX: 'right',
          anchorY: 'middle',
          rotation: [0, Math.PI / 2, 0],
          position: [0, 0.025, 0.005],
          desc: 'z (m)',
        }}
      />
    </group>
  )
}

function AxisTicks({
  color,
  dir,
  end,
  text = {},
}: {
  color: string
  dir: readonly [number, number, number]
  end: readonly [number, number, number]
  text: {
    anchorX?: 'left' | 'right' | 'center'
    anchorY?: 'bottom' | 'top' | 'bottom-baseline' | 'middle' | 'top-baseline'
    position?: readonly [number, number, number]
    rotation?: readonly [number, number, number]
    desc?: string
  }
}) {
  return (
    <>
      {Array.from({ length: 10 }, (_, i) => {
        const k = (i + 1) / 10
        const from: [number, number, number] = [
          dir[0] * k,
          dir[1] * k,
          dir[2] * k,
        ]
        return (
          <Fragment key={i}>
            <Segment
              from={from}
              to={[from[0] + end[0], from[1] + end[1], from[2] + end[2]]}
              color={color}
              thickness={0.002}
            />
            <Text
              color="black"
              anchorX={text.anchorX}
              anchorY={text.anchorY}
              position={plus(from, text.position)}
              scale={0.4}
              rotation={plus(text.rotation)}
            >
              {k.toFixed(1)}
            </Text>
          </Fragment>
        )
      })}
      <Text
        color="black"
        anchorX="center"
        anchorY="top-baseline"
        position={plus(mul(0.5, dir), mul(-1, end))}
        scale={0.4}
        rotation={plus(text.rotation)}
      >
        {text.desc ?? ''}
      </Text>
    </>
  )
}

/**
 * Function adding multiple 3d vectors
 */
export function plus(
  ...a: readonly (readonly [number, number, number] | undefined)[]
): [number, number, number] {
  return [
    a.map((v) => v?.[0] ?? 0).reduce((a, b) => a + b, 0),
    a.map((v) => v?.[1] ?? 0).reduce((a, b) => a + b, 0),
    a.map((v) => v?.[2] ?? 0).reduce((a, b) => a + b, 0),
  ]
}

/**
 * Multiply 3d vector by a scalar
 */
export function mul(
  k: number,
  v: readonly [number, number, number] | undefined,
): [number, number, number] {
  return [(v?.[0] ?? 0) * k, (v?.[1] ?? 0) * k, (v?.[2] ?? 0) * k]
}

/**
 * Adds camera control buttons to the leva panel
 */
export function CameraControls() {
  const position = useRef<null | readonly [number, number, number]>(null)
  useFrame((state) => {
    if (position.current) {
      state.camera.position.set(
        ...mul(state.camera.position.length(), position.current),
      )
      state.camera.lookAt(0, 0, 0)
      if (position.current[1]) state.camera.up.set(-1, 0, 0)
      else state.camera.up.set(0, 1, 0)
      position.current = null
    }
  })
  useControls({
    yz: button(() => {
      position.current = [1, 0, 0]
    }),
    xz: button(() => {
      position.current = [0, 1, 0]
    }),
    xy: button(() => {
      position.current = [0, 0, 1]
    }),
  })
  return null
}

/**
 * Shows connections between two sets of points. Way more efficient than using
 * multiple <Segment/> components. As in, if you tried to draw it with segments
 * you would get very low FPS and this draws at 60 FPS no problem (at least in
 * my tests).
 */
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

/**
 * Distance between two 3d points
 */
export function dist(
  a: readonly [number, number, number],
  b: readonly [number, number, number],
) {
  const x = a[0] - b[0]
  const y = a[1] - b[1]
  const z = a[2] - b[2]

  return Math.sqrt(x * x + y * y + z * z)
}
