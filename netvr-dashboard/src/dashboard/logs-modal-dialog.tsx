/** @jsxImportSource @emotion/react */
import { css } from '@emotion/react'
import { JSONView } from '../components/json-view'
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
      css={{
        display: 'flex',
        flexDirection: 'column',
        border: '1px solid var(--base-2)',
        scrollbarWidth: 'thin',
        background: 'var(--base-0)',
        color: 'var(--base-7)',
        padding: 16,
        height: 'calc(100vh - 64px)',

        borderRadius: 4,
        '::backdrop': {
          background: 'black',
          opacity: 0.6,
        },
      }}
    >
      <form
        method="dialog"
        css={{
          position: 'relative',
          alignSelf: 'flex-end',
          display: 'flex',
          alignItems: 'flex-end',
          flexDirection: 'column',
        }}
      >
        <button
          css={{
            all: 'unset',
            zIndex: 1,
            position: 'fixed',
            cursor: 'pointer',
            fontSize: '16px',
            border: '1px solid currentColor',
            borderRadius: '4px',
            padding: '8px',
            backgroundColor: 'var(--base-0)',
          }}
        >
          Close
        </button>
      </form>
      <code>
        <pre>
          {logs.map((v, i) => (
            <div
              key={i}
              css={v.type !== 'log' ? css({ color: 'var(--base-8)' }) : {}}
            >
              {v.text}
              {v.json ? <JSONView data={JSON.parse(v.json)} /> : null}
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
