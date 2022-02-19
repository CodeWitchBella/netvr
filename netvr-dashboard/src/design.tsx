import { PropsWithChildren } from 'react'
import { useTheme } from './use-theme'

export function Pane({ children }: PropsWithChildren<{}>) {
  const theme = useTheme()
  return (
    <div
      style={{
        padding: 8,

        margin: 8,
        borderRadius: 4,
        border: '1px solid gray',
        background: theme.resolved.base00,
        color: theme.resolved.base06,
      }}
    >
      {children}
    </div>
  )
}
