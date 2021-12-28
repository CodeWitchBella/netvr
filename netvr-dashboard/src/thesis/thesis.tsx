import pdf from '@react-pdf/renderer'
import { useEffect, useReducer, useRef } from 'react'
import { PDFContextProvider } from './base'
import { Document as DocumentI } from './document'
// @ts-ignore
import * as pdfjsLib from 'pdfjs-dist/build/pdf.js'
import * as pdfjsViewer from 'pdfjs-dist/web/pdf_viewer'
import 'pdfjs-dist/web/pdf_viewer.css'

pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  '../../node_modules/pdfjs-dist/build/pdf.worker.min.js',
  import.meta.url,
).toString()

let updateInstanceSet = new Set<(doc: typeof DocumentI) => void>()

export function Thesis() {
  const [Document, setDocument] = useReducer(
    (_: typeof DocumentI, doc: typeof DocumentI) => doc,
    DocumentI,
  )
  const document = (
    <PDFContextProvider value={{ lang: 'en' }}>
      <Document />
    </PDFContextProvider>
  )

  //return (<pdf.PDFViewer style={{ flexGrow: 1, border: 0 }}>{document}</pdf.PDFViewer>)

  const [instance, updateInstance] = pdf.usePDF({ document })

  if (import.meta.hot) {
    useEffect(() => {
      updateInstanceSet.add(setDocument)
      return () => {
        updateInstanceSet.delete(setDocument)
      }
    }, [setDocument])

    useEffect(() => {
      if (Document !== DocumentI) updateInstance()
    }, [Document])
  }

  const src = instance.url ? `${instance.url}#toolbar=1` : ''

  const ref = useRef<{
    div: HTMLDivElement
    viewer: pdfjsViewer.PDFViewer
    linkService: pdfjsViewer.PDFLinkService
    eventBus: pdfjsViewer.EventBus
  }>()

  useEffect(() => {
    let ended = false
    const instance = ref.current
    if (src && instance) {
      const task = pdfjsLib.getDocument(src)
      const scroll = instance.div.scrollTop
      task.promise.then((pdf: any) => {
        if (!ended) {
          instance.viewer.setDocument(pdf)
          instance.linkService.setDocument(pdf, null)
          const pagesinit = () => {
            instance.eventBus.off('pagesinit', pagesinit)
            instance.div.scrollTop = scroll
          }
          instance.eventBus.on('pagesinit', pagesinit)
        }
      })
      return () => {
        ended = true
        task.destroy()
      }
    }
  }, [src])

  return (
    <div style={{ position: 'relative', flexGrow: 1 }}>
      <div
        style={{
          position: 'absolute',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          overflow: 'auto',
        }}
        ref={
          useRef((container: HTMLDivElement | null) => {
            ref.current?.viewer.cleanup()
            ref.current = undefined
            if (container) {
              const eventBus = new pdfjsViewer.EventBus()
              const linkService = new pdfjsViewer.PDFLinkService({
                eventBus,
              })
              ref.current = {
                div: container,
                eventBus,
                linkService: linkService,
                viewer: new pdfjsViewer.PDFViewer({
                  container,
                  eventBus,
                  linkService,
                } as any),
              }
            }
          }).current
        }
      >
        <div id="viewer" className="pdfViewer"></div>
      </div>
    </div>
  )
}
export default Thesis

if (import.meta.hot) {
  import.meta.hot.accept('./document', (dep: any) => {
    for (const update of updateInstanceSet.values()) {
      update(dep.Document)
    }
  })
}
