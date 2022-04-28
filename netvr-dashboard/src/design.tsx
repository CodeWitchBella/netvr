import { PropsWithChildren, useState } from 'react'
import { ErrorBoundary } from './error-boundary'
import { useTheme } from './use-theme'

export const fontFamily = 'Inter, sans-serif'

export function Pane({
  children,
  title,
  id,
}: PropsWithChildren<{ title?: string; id?: string }>) {
  const theme = useTheme()
  const [open, setOpen] = useState(
    id ? localStorage.getItem('pane-' + id) !== 'false' : true,
  )
  return (
    <div
      style={{
        padding: 8,

        margin: 8,
        borderRadius: 4,
        border: '1px solid gray',
        background: theme.resolved.base00,
        color: theme.resolved.base06,

        display: 'flex',
        flexDirection: 'column',
        gap: 6,
      }}
    >
      {title ? (
        <button
          style={{
            all: 'unset',
            cursor: 'pointer',
            ...(open
              ? {
                  borderBlockEnd: '1px solid ' + theme.resolved.base02,
                  marginBlockEnd: 8,
                  paddingBlockEnd: 8,
                }
              : {}),

            marginInline: -8,
            paddingInline: 8,
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
          type="button"
          onClick={() => {
            setOpen(!open)
            if (id) {
              if (open) localStorage.setItem('pane-' + id, 'false')
              else localStorage.removeItem('pane-' + id)
            }
          }}
        >
          {title ?? 'Pane'}
          <div>{open ? '🔽' : '🔼'}</div>
        </button>
      ) : null}
      {open || !title ? <ErrorBoundary>{children}</ErrorBoundary> : null}
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
        ...(props.disabled
          ? {
              color: theme.resolved.base04,
              borderColor: 'transparent',
              cursor: 'pointer',
            }
          : {}),
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
