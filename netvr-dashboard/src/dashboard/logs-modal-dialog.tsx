/** @jsxImportSource @emotion/react */
import { css } from '@emotion/react'
import { MessageTransmitLogs } from '../protocol/recieved-messages'

export function LogsModalDialog({
  logs,
  onClose,
}: {
  logs: MessageTransmitLogs['logs']
  onClose: () => void
}) {
  return (
    <dialog
      ref={autoOpen}
      // @ts-expect-error
      onClose={() => {
        onClose?.()
      }}
      onClick={(event) => {
        // @ts-expect-error
        if (event.target === event.currentTarget) event.currentTarget.close()
      }}
      id="fullscreen-logs"
      css={{ borderWidth: 0, scrollbarWidth: 'thin' }}
    >
      <form method="dialog" css={{ position: 'fixed', right: 24, top: 16 }}>
        <button
          css={{
            all: 'unset',
            cursor: 'pointer',
            fontSize: '16px',
            border: '1px solid currentColor',
            borderRadius: '4px',
            padding: '8px',
          }}
        >
          Close
        </button>
      </form>
      <code>
        <pre>
          {logs.map((v) => (
            <div css={v.type !== 'log' ? css({ color: 'var(--base-8)' }) : {}}>
              {v.text}
            </div>
          ))}
        </pre>
      </code>
    </dialog>
  )
}

function autoOpen(v: HTMLDialogElement) {
  // @ts-expect-error
  if (v && !v.hasAttribute('open')) v.showModal()
}
