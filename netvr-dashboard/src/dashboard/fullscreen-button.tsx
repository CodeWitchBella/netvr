/** @jsxImportSource @emotion/react */
import { useState, useEffect } from 'react'
import { focusableStyles } from '../components/design'

function useFullscreen() {
  const [enabled, setEnabled] = useState(!!document.fullscreenElement)
  useEffect(() => {
    document.addEventListener('fullscreenchange', handler)
    return () => void document.removeEventListener('fullscreenchange', handler)
    function handler() {
      setEnabled(!!document.fullscreenElement)
    }
  })

  return {
    enabled,
    exit: () => {
      document.exitFullscreen()
    },
    request: () => {
      document.documentElement.requestFullscreen({ navigationUI: 'hide' })
    },
  }
}

export function FullscreenButton() {
  const fullscreen = useFullscreen()
  return (
    <button
      type="button"
      onClick={(event) => {
        event.stopPropagation()
        if (fullscreen.enabled) fullscreen.exit()
        else fullscreen.request()
      }}
      css={[
        {
          all: 'unset',
          width: 24,
          height: 24,
          margin: -4,
          padding: 0,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          background: 'transparent',
          border: '1px solid transparent',
          cursor: 'pointer',
          borderRadius: 4,
          ':hover': {
            background: 'var(--base-1)',
            border: '1px solid var(--base-2)',
          },
        },
        focusableStyles,
      ]}
    >
      <svg viewBox="0 0 24 24" fill="currentColor">
        {fullscreen.enabled ? (
          // from https://fonts.google.com/icons?selected=Material%20Icons%3Afullscreen_exit%3A
          <path d="M5 16h3v3h2v-5H5v2zm3-8H5v2h5V5H8v3zm6 11h2v-3h3v-2h-5v5zm2-11V5h-2v5h5V8h-3z" />
        ) : (
          // from https://fonts.google.com/icons?selected=Material%20Icons%3Afullscreen%3A
          <path d="M7 14H5v5h5v-2H7v-3zm-2-4h2V7h3V5H5v5zm12 7h-3v2h5v-5h-2v3zM14 5v2h3v3h2V5h-5z" />
        )}
      </svg>
    </button>
  )
}
