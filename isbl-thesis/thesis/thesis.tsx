import pdf from '@react-pdf/renderer'
import { useEffect, useMemo, useReducer, useRef } from 'react'
import { PDFContext, PDFContextProvider } from './base'
import { Document, DocumentProps } from './document'
// @ts-ignore
import * as pdfjsLib from 'pdfjs-dist/build/pdf.js'
import * as pdfjsViewer from 'pdfjs-dist/web/pdf_viewer'
import 'pdfjs-dist/web/pdf_viewer.css'

pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  '../node_modules/pdfjs-dist/build/pdf.worker.min.js',
  import.meta.url,
).toString()

type Config = { useBuiltIn: boolean; production: boolean }
const defaultConfig: Config = {
  useBuiltIn: false,
  production: true,
}

export function Thesis(props: DocumentProps) {
  const [config, setConfig] = useReducer(
    (state: Config, partial: Partial<Config>): Config => {
      const nextState = { ...state, ...partial }
      localStorage.setItem('thesis-config', JSON.stringify(nextState))
      return nextState
    },
    null,
    (): Config => ({
      ...defaultConfig,
      ...JSON.parse(localStorage.getItem('thesis-config') || '{}'),
    }),
  )

  const context = useMemo(
    (): PDFContext => ({
      lang: 'en',
      production: config.production,
    }),
    [config],
  )

  if (config.useBuiltIn) {
    return (
      <>
        <ThesisConfig config={config} setConfig={setConfig} />
        <pdf.PDFViewer style={{ flexGrow: 1, border: 0 }}>
          <PDFContextProvider value={context}>
            <Document {...props} />
          </PDFContextProvider>
        </pdf.PDFViewer>
      </>
    )
  }
  return (
    <>
      <ThesisConfig config={config} setConfig={setConfig} />
      <ThesisCustom Document={Document} context={context} />
    </>
  )
}

function ThesisConfig({
  config,
  setConfig,
}: {
  config: Config
  setConfig: (cfg: Partial<Config>) => void
}) {
  return (
    <div
      style={{
        gap: 8,
        display: 'flex',
        padding: 4,
        borderBottom: '1px solid gray',
      }}
    >
      <div>Config:</div>
      <label>
        <input
          type="checkbox"
          defaultChecked={config.useBuiltIn}
          onChange={(event) => {
            setTimeout(
              () => void setConfig({ useBuiltIn: event.target.checked }),
              0,
            )
          }}
        />{' '}
        use built-in viewer
      </label>
      <label>
        <input
          type="checkbox"
          defaultChecked={config.production}
          onChange={(event) => {
            setTimeout(
              () => void setConfig({ production: event.target.checked }),
              0,
            )
          }}
        />{' '}
        only final-ready
      </label>
    </div>
  )
}

function ThesisCustom({
  Document,
  context,
  documentProps,
}: {
  Document: any
  context: PDFContext
  documentProps: DocumentProps
}) {
  const [instance, updateInstance] = pdf.usePDF({
    document: (
      <PDFContextProvider value={context}>
        <Document {...documentProps} />
      </PDFContextProvider>
    ),
  })

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
              const viewer = new pdfjsViewer.PDFViewer({
                container,
                eventBus,
                linkService,
              } as any)
              linkService.setViewer(viewer)
              ref.current = {
                div: container,
                eventBus,
                linkService: linkService,
                viewer,
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
