import { JSONTree } from 'react-json-tree'
import { Pane } from './design'
import { useTheme } from './theme'

type Props = {
  data: any
  shouldExpandNode?: (
    keyPath: (string | number)[],
    data: any,
    level: number,
  ) => boolean
  name?: string
}

export function JSONView({ data, shouldExpandNode, name }: Props) {
  const theme = useTheme()
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
            value.every((v) => typeof v === 'string'))
        )
      }}
      valueRenderer={(valueAsString, value, key) => {
        if (typeof value === 'object' && value) {
          if (Array.isArray(value))
            return (
              <span
                style={{ color: theme.base09 }}
                title={JSON.stringify(value)}
              >
                {value.join(', ')}
              </span>
            )
          return (
            <span style={{ color: theme.base09 }} title={JSON.stringify(value)}>
              Vector3[{value.x}, {value.y}, {value.z}]
            </span>
          )
        }
        if (key === 'operatingSystem')
          return (
            <span title={valueAsString}>
              {valueAsString.substring(0, 35) + '…'}
            </span>
          )
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
      <div style={{ marginTop: -8 }}>
        <JSONView {...props} />
      </div>
    </Pane>
  )
}
