import queue from 'queue'
import { useState, useRef, useEffect } from 'react'
import { pdf } from '@react-pdf/renderer'

// adapted from https://github.com/diegomura/react-pdf/blob/1f0eb6e0d4e75480de6745a204924d5075859db7/packages/renderer/src/dom/usePDF.js
export function usePDF({ document }: { document: any }) {
  const pdfInstance = useRef<ReturnType<typeof pdf>>(null as any)

  const [state, setState] = useState({
    url: null as string | null,
    blob: null as Blob | null,
    error: null as Error | null,
    loading: false as boolean,
  })

  // Setup rendering queue
  useEffect(() => {
    const renderQueue: any = queue({ autostart: true, concurrency: 1 })

    const queueDocumentRender = () => {
      setState((prev) => ({ ...prev, loading: true }))

      renderQueue.splice(0, renderQueue.length, () =>
        pdfInstance.current.toBlob(),
      )
    }

    const onRenderFailed = (error: any) => {
      console.error(error)
      setState((prev) => ({ ...prev, error }))
    }

    const onRenderSuccessful = (blob: any) => {
      setState({
        blob,
        error: null,
        loading: false,
        url: URL.createObjectURL(blob),
      })
    }

    pdfInstance.current = pdf()
    pdfInstance.current.on('change', queueDocumentRender)
    pdfInstance.current.updateContainer(document)

    renderQueue.on('error', onRenderFailed)
    renderQueue.on('success', onRenderSuccessful)

    return () => {
      renderQueue.end()
      pdfInstance.current.removeListener('change', queueDocumentRender)
    }
  }, [])

  // Revoke old unused url instances
  useEffect(() => {
    return () => {
      if (state.url) {
        URL.revokeObjectURL(state.url)
      }
    }
  }, [state.url])

  const prevDocument = useRef(document)

  useEffect(() => {
    if (prevDocument.current !== document) {
      pdfInstance.current.updateContainer(document)
      prevDocument.current = document
    }
  })

  return state
}
