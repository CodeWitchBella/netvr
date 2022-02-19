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

export function Button(
  props: React.DetailedHTMLProps<
    React.ButtonHTMLAttributes<HTMLButtonElement>,
    HTMLButtonElement
  >,
) {
  const theme = useTheme()
  return (
    <button
      {...props}
      style={{
        all: 'unset',
        border: `1px solid ${theme.resolved.base03}`,
        padding: '4px 8px',
        borderRadius: 4,
        fontFamily: 'sans-serif',
        fontSize: '1rem',
        color: theme.resolved.base07,
        background: theme.resolved.base01,
        ...props.style,
      }}
    />
  )
}
