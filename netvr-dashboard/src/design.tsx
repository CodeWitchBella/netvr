import { PropsWithChildren } from 'react'
import { ErrorBoundary } from './error-boundary'
import { useTheme } from './use-theme'

export const fontFamily = 'Inter, sans-serif'

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
      <ErrorBoundary>{children}</ErrorBoundary>
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
        cursor: 'pointer',
        border: `1px solid ${theme.resolved.base03}`,
        padding: '4px 8px',
        borderRadius: 4,
        fontFamily,
        fontSize: '1rem',
        color: theme.resolved.base07,
        background: theme.resolved.base01,
        ...props.style,
      }}
    />
  )
}

export function Select(
  props: React.DetailedHTMLProps<
    React.SelectHTMLAttributes<HTMLSelectElement>,
    HTMLSelectElement
  >,
) {
  const theme = useTheme()
  return (
    <select
      {...props}
      style={{
        border: `1px solid ${theme.resolved.base03}`,
        padding: '2px 4px',
        borderRadius: 4,
        fontFamily,
        fontSize: '1rem',
        color: theme.resolved.base07,
        background: theme.resolved.base01,
        position: 'relative',
        ...props.style,
      }}
    >
      {props.children}
    </select>
  )
}
