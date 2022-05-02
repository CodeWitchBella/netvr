/** @jsxImportSource @emotion/react */
import { css } from '@emotion/react'
import React, { PropsWithChildren, useRef, useState } from 'react'
import { ErrorBoundary } from './error-boundary'

export const focusableStyles = css({
  ':focus-visible': {
    outline: '2px solid var(--base-e)',
  },
})

export const selectionStyle = css({
  '::selection': {
    background: 'var(--base-d)',
    color: 'var(--base-0)',
  },
})

export function Pane({
  children,
  title,
  id,
  buttons,
}: PropsWithChildren<{
  title?: string
  id?: string
  buttons?: React.ReactNode
}>) {
  const [open, setOpen] = useState(
    id ? localStorage.getItem('pane-' + id) !== 'false' : true,
  )
  const onClick = (event: React.MouseEvent) => {
    event.stopPropagation()
    buttonRef.current?.focus()
    setOpen(!open)
    if (id) {
      if (open) localStorage.setItem('pane-' + id, 'false')
      else localStorage.removeItem('pane-' + id)
    }
  }
  const buttonRef = useRef<HTMLButtonElement>(null)
  return (
    <div
      css={{
        padding: 8,

        margin: 8,
        borderRadius: 4,
        border: '1px solid var(--base-3)',
        background: 'var(--base-0)',
        color: 'var(--base-6)',

        display: 'flex',
        flexDirection: 'column',
        gap: 6,
      }}
    >
      {title ? (
        <div
          css={[
            {
              all: 'unset',
              cursor: 'pointer',
              margin: '-8px',
              userSelect: 'none',

              padding: 8,
              display: 'flex',
              alignItems: 'center',
              gap: 8,
            },
            open
              ? css({
                  borderBlockEnd: '1px solid var(--base-2)',
                  marginBlockEnd: 0,
                })
              : {},
          ]}
          onClick={onClick}
        >
          <button
            ref={buttonRef}
            type="button"
            css={[
              {
                all: 'unset',
                border: '1px solid transparent',
                borderRadius: 4,
                margin: '-8px -4px',
              },
              focusableStyles,
            ]}
            onClick={onClick}
          >
            <svg
              viewBox="0 0 24 24"
              width={24}
              css={{ transition: 'transform 200ms ease-in-out' }}
              fill="currentColor"
              style={{
                transform: open
                  ? 'translateY(1px) rotate(0deg)'
                  : 'translateY(1px) rotate(-180deg)',
              }}
            >
              <path d="M16.59 8.59 12 13.17 7.41 8.59 6 10l6 6 6-6z" />
            </svg>
          </button>
          {title ?? 'Pane'}

          {buttons ? (
            <>
              <div css={{ flexGrow: 1 }} />
              <div>{buttons}</div>
            </>
          ) : null}
        </div>
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
      css={[
        {
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
        },
        focusableStyles,
      ]}
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
      css={[
        {
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
        },
        focusableStyles,
      ]}
    >
      {props.children}
    </select>
  )
}

export function Input(
  props: React.DetailedHTMLProps<
    React.InputHTMLAttributes<HTMLInputElement>,
    HTMLInputElement
  >,
) {
  return (
    <input
      {...props}
      css={[
        {
          border: '1px solid var(--base-3)',
          padding: '4px 8px',
          borderRadius: 4,
          fontFamily: 'inherit',
          fontSize: '1rem',
          color: 'var(--base-7)',
          background: 'var(--base-1)',
          position: 'relative',
        },
        selectionStyle,
        focusableStyles,
      ]}
    >
      {props.children}
    </input>
  )
}
