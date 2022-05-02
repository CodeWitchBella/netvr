import { useState, useEffect } from 'react'
import { Button } from '../components/design'

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
    <Button
      type="button"
      onClick={() => {
        if (fullscreen.enabled) fullscreen.exit()
        else fullscreen.request()
      }}
    >
      {fullscreen.enabled ? 'Exit fullscreen' : 'Fullscreen'}
    </Button>
  )
}
