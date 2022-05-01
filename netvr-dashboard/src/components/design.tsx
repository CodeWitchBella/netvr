/** @jsxImportSource @emotion/react */
import { css } from '@emotion/react'
import { PropsWithChildren, useState } from 'react'
import { ErrorBoundary } from './error-boundary'

export function Pane({
  children,
  title,
  id,
}: PropsWithChildren<{ title?: string; id?: string }>) {
  const [open, setOpen] = useState(
    id ? localStorage.getItem('pane-' + id) !== 'false' : true,
  )
  return (
    <div
      css={{
        padding: 8,

        margin: 8,
        borderRadius: 4,
        border: '1px solid gray',
        background: 'var(--base-0)',
        color: 'var(--base-6)',

        display: 'flex',
        flexDirection: 'column',
        gap: 6,
      }}
    >
      {title ? (
        <button
          css={[
            {
              all: 'unset',
              cursor: 'pointer',
              margin: '-8px',
              userSelect: 'none',

              padding: 8,
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            },
            open
              ? css({
                  borderBlockEnd: '1px solid var(--base-2)',
                  marginBlockEnd: 0,
                })
              : {},
          ]}
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
  return (
    <button
      {...props}
      css={{
        all: 'unset',
        cursor: 'pointer',
        border: '1px solid var(--base-3)',
        padding: '4px 8px',
        borderRadius: 4,
        fontFamily: 'inherit',
        fontSize: '1rem',
        color: 'var(--base-7)',
        background: 'var(--base-1)',
        userSelect: 'none',
        '&[disabled]': {
          color: 'var(--base-4)',
          borderColor: 'var(--base-2)',
          cursor: 'default',
        },
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
  return (
    <select
      {...props}
      css={{
        border: '1px solid var(--base-3)',
        padding: '4px 8px',
        borderRadius: 4,
        fontFamily: 'inherit',
        fontSize: '1rem',
        color: 'var(--base-7)',
        background: 'var(--base-1)',
        position: 'relative',
        '&[disabled]': {
          color: 'var(--base-4)',
          borderColor: 'var(--base-2)',
          cursor: 'default',
        },
      }}
    >
      {props.children}
    </select>
  )
}
