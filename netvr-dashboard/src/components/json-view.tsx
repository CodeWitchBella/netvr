/** @jsxImportSource @emotion/react */
import { JSONTree } from 'react-json-tree'
import { Pane } from './design'
import { Fragment } from 'react'

type Props = {
  data: any
  shouldExpandNode?: (
    keyPath: (string | number)[],
    data: any,
    level: number,
  ) => boolean
  name?: string
}

const theme = [...'0123456789ABCDEF'].reduce((object, l) => {
  object['base0' + l] = `var(--base-${l})`.toLowerCase()
  return object
}, {} as any)

export function JSONView({ data, shouldExpandNode, name }: Props) {
  return (
    <JSONTree
      data={data}
      theme={theme}
      invertTheme={false}
      shouldExpandNode={shouldExpandNode}
      keyPath={name ? [name] : undefined}
      isCustomNode={(value) => {
        return (
          (typeof value === 'object' &&
            value &&
            Object.keys(value).length === 3 &&
            typeof value.x === 'number' &&
            typeof value.y === 'number' &&
            typeof value.z === 'number') ||
          (typeof value === 'object' &&
            value &&
            Array.isArray(value) &&
            value.every((v) => typeof v === 'string' && v.startsWith('/'))) ||
          (typeof value === 'string' && value.startsWith('/')) ||
          (typeof value === 'object' &&
            value &&
            Object.keys(value).length === 4 &&
            typeof value.x === 'number' &&
            typeof value.y === 'number' &&
            typeof value.z === 'number' &&
            typeof value.w === 'number')
        )
      }}
      valueRenderer={(valueAsString, value, key) => {
        if (typeof value === 'string' && value.startsWith('/')) {
          return (
            <span
              css={{ color: 'var(--base-9)' }}
              title={JSON.stringify(value)}
            >
              {value}
            </span>
          )
        }
        if (typeof value === 'object' && value) {
          if (Array.isArray(value))
            return (
              <span
                title={JSON.stringify(value)}
                css={{ color: 'var(--base-3)' }}
              >
                [
                {value.map((v, i) => (
                  <Fragment key={i}>
                    {i > 0 ? ', ' : ''}
                    <span css={{ color: 'var(--base-9)' }}>{v}</span>
                  </Fragment>
                ))}
                ]
              </span>
            )
          if (typeof value.w === 'number')
            return (
              <span
                css={{ color: 'var(--base-9)' }}
                title={JSON.stringify(value)}
              >
                Quaternion[{value.x}, {value.y}, {value.z}, {value.w}]
              </span>
            )
          return (
            <span
              css={{ color: 'var(--base-9)' }}
              title={JSON.stringify(value)}
            >
              Vector3[{value.x}, {value.y}, {value.z}]
            </span>
          )
        }

        return valueAsString
      }}
    />
  )
}

export function JSONPane({
  title,
  id,
  ...props
}: Props & { title?: string; id?: string }) {
  return (
    <Pane title={title} id={id}>
      <div css={{ marginTop: -8 }}>
        <JSONView {...props} />
      </div>
    </Pane>
  )
}
