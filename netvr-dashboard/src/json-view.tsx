import { JSONTree } from 'react-json-tree'
import { Pane } from './design'
import { useTheme } from './use-theme'

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
      theme={theme.name}
      invertTheme={theme.inverted}
      shouldExpandNode={shouldExpandNode}
      keyPath={name ? [name] : undefined}
      isCustomNode={(value) => {
        return (
          typeof value === 'object' &&
          value &&
          Object.keys(value).length === 3 &&
          typeof value.x === 'number' &&
          typeof value.y === 'number' &&
          typeof value.z === 'number'
        )
      }}
      valueRenderer={(valueAsString, value) => {
        if (typeof value === 'object' && value) {
          return (
            <span
              style={{ color: theme.resolved.base09 }}
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

export function JSONPane(props: Props) {
  return (
    <Pane>
      <div style={{ marginTop: -8 }}>
        <JSONView {...props} />
      </div>
    </Pane>
  )
}
