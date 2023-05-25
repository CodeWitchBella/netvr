/** @jsxImportSource @emotion/react */
import { css } from '@emotion/react'
import React, { PropsWithChildren, useId, useState } from 'react'
import { ErrorBoundary } from './error-boundary'

/**
 * Styles to be applied to all focusable elements.
 */
export const focusableStyles = css({
  ':focus-visible': {
    outline: '2px solid var(--base-e)',
  },
})

/**
 * Styles to be applied to the root so that selection follows the theme.
 */
export const selectionStyle = css({
  '::selection': {
    background: 'var(--base-d)',
    color: 'var(--base-0)',
  },
})

/**
 * How a Pane should look and work. Stores its open state in local storage based
 * on the id.
 *
 * @param param0
 * @returns
 */
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

    setOpen(!open)
    if (id) {
      if (open) localStorage.setItem('pane-' + id, 'false')
      else localStorage.removeItem('pane-' + id)
    }
  }
  const htmlId = useId()
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
              margin: '-8px',
              userSelect: 'none',

              padding: 8,
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              position: 'relative',
            },
            open
              ? css({
                  borderBlockEnd: '1px solid var(--base-2)',
                  marginBlockEnd: 0,
                })
              : {},
          ]}
        >
          <label
            css={{
              position: 'absolute',
              left: 0,
              right: 0,
              top: 0,
              bottom: 0,
            }}
            htmlFor={htmlId}
          />
          <button
            type="button"
            css={[
              {
                all: 'unset',
                border: '1px solid transparent',
                borderRadius: 4,
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                padding: 4,
                margin: -4,
                'label:hover + &': {
                  textDecoration: 'underline',
                },
              },
              focusableStyles,
            ]}
            onClick={onClick}
            id={htmlId}
            data-pane-header
            aria-expanded={open}
            aria-controls={htmlId + 'contents'}
            onKeyDown={(event) => {
              const selector = 'button[data-pane-header]'
              if (event.key === 'ArrowUp' || event.key === 'ArrowDown') {
                const headers = Array.from(
                  document.querySelectorAll(selector),
                ) as HTMLButtonElement[]
                const selfIdx = headers.findIndex(
                  (h) => h === event.currentTarget,
                )
                const diff = event.key === 'ArrowUp' ? -1 : 1

                focus(
                  headers[(selfIdx + diff + headers.length) % headers.length],
                )
              } else if (event.key === 'Home') {
                focus(document.querySelector(selector))
              } else if (event.key === 'End') {
                const headers = document.querySelectorAll(selector)
                focus(headers.item(headers.length - 1))
              }

              function focus(el: Element | undefined | null) {
                ;(el as any)?.focus?.()
              }
            }}
          >
            <svg
              viewBox="0 0 24 24"
              width={24}
              css={{
                transition: 'transform 200ms ease-in-out',
                margin: '-8px -4px',
              }}
              fill="currentColor"
              style={{
                transform: open ? 'rotate(0deg)' : 'rotate(-180deg)',
              }}
            >
              <path d="M16.59 8.59 12 13.17 7.41 8.59 6 10l6 6 6-6z" />
            </svg>
            {title ?? 'Pane'}
          </button>

          {buttons ? (
            <>
              <div css={{ flexGrow: 1 }} />
              <div css={{ position: 'relative' }}>{buttons}</div>
            </>
          ) : null}
        </div>
      ) : null}
      {open || !title ? (
        <ErrorBoundary>
          <div id={htmlId + 'contents'}>{children}</div>
        </ErrorBoundary>
      ) : null}
    </div>
  )
}

/**
 * A button that follows the theme. This is here so that I don't have to style
 * every damn button manually.
 *
 * @param props
 * @returns
 */
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
          },
          ':not([disabled]):hover': {
            backgroundColor: 'var(--base-2)',
          },
        },
        focusableStyles,
      ]}
    />
  )
}

/**
 * Select element that follows the theme.
 * @param props
 * @returns
 */
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
          ':not([disabled]):hover': {
            backgroundColor: 'var(--base-2)',
          },
        },
        focusableStyles,
      ]}
    >
      {props.children}
    </select>
  )
}

/**
 * Input element that follows the theme.
 * @param props
 * @returns
 */
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
