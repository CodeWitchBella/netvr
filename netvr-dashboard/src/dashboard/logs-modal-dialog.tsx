import { useTheme } from '../components/theme'
import { MessageTransmitLogs } from '../protocol/recieved-messages'

export function LogsModalDialog({
  logs,
  onClose,
}: {
  logs: MessageTransmitLogs['logs']
  onClose: () => void
}) {
  const theme = useTheme()
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
      style={{ borderWidth: 0, scrollbarWidth: 'thin' }}
    >
      <form method="dialog" style={{ position: 'fixed', right: 24, top: 16 }}>
        <button
          style={{
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
            <div style={v.type !== 'log' ? { color: theme.base08 } : {}}>
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
