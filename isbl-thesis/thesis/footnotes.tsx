import pdf from '@react-pdf/renderer'
import { createContext, PropsWithChildren, useContext, useEffect } from 'react'
import { replaceEscapes } from './rehype-ligatures'

class FootnoteContext {
  pages = new Map<number, string[]>()
  footnotes = new Map<string, { pageNumber: number; sign: string }>()
}

const context = createContext(new FootnoteContext())

export function FootnoteRef({
  content,
  sign: signInput,
}: {
  content: string
  sign?: string
}) {
  const ctx = useContext(context)
  const sign = typeof signInput === 'string' ? replaceEscapes(signInput) : '*'
  useEffect(
    () => () => {
      const footnote = ctx.footnotes.get(content)
      if (footnote) {
        ctx.footnotes.delete(content)
        const page = ctx.pages.get(footnote.pageNumber)
        if (page && page.includes(content)) {
          page.splice(page.indexOf(content), 1)
        }
      }
    },
    [content],
  )
  return (
    <>
      <pdf.View
        render={({ pageNumber }) => {
          const footnote = ctx.footnotes.get(content) ?? { pageNumber: 0, sign }
          ctx.footnotes.set(content, footnote)
          footnote.sign = sign

          if (footnote.pageNumber && footnote.pageNumber !== pageNumber) {
            const l = ctx.pages.get(footnote.pageNumber)
            if (l) l.splice(l.indexOf(content), 1)
          }

          const list = ctx.pages.get(pageNumber) ?? []
          if (footnote.pageNumber !== pageNumber) {
            footnote.pageNumber = pageNumber
            if (!list.includes(content)) {
              list.push(content)
              ctx.pages.set(pageNumber, list)
            }
          }
          return null
        }}
      />
      <pdf.Text>{sign}</pdf.Text>
    </>
  )
}

export function FootnoteRenderer({ children }: PropsWithChildren<{}>) {
  const ctx = useContext(context)
  return (
    <>
      {children}
      <pdf.View
        fixed
        style={{
          position: 'absolute',
          bottom: '3cm',
          left: '32mm',
          right: '32mm',
        }}
        render={({ pageNumber }) => {
          const footnotes = ctx.pages.get(pageNumber)
          if (!footnotes?.length) return null
          return (
            <pdf.View>
              {footnotes.map((content, i) => {
                const note = ctx.footnotes.get(content)
                if (!note) return null
                return (
                  <pdf.Text
                    key={i}
                    style={{ fontFamily: 'lmroman10-regular', fontSize: 11 }}
                  >
                    {note?.sign}: {content}
                  </pdf.Text>
                )
              })}
            </pdf.View>
          )
        }}
      />
    </>
  )
}
