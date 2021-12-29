import pdf from '@react-pdf/renderer'
import { Page, usePDFContext } from './base'
import { ChapterProvider } from './design'
import { LMText, registerFonts } from './font'
import { TitlePage } from './title-page'
const { Document: PDFDocument } = pdf
import { ReactMarkdown, markdownListToAst } from './react-markdown'
import { chapters, citations } from '../thesis-text/chapters'
import { Literature } from './literature'
import { notNull } from '@isbl/ts-utils'

function MarkdownChapter({
  children,
  index,
}: {
  children: any
  index: number
}) {
  return (
    <ChapterProvider index={index}>
      <ReactMarkdown hast={children} />
    </ChapterProvider>
  )
}

export function Document() {
  registerFonts()
  const { lang } = usePDFContext()
  const parsed = markdownListToAst(chapters.map(([key, text]) => text))

  console.log(parsed.citeMap)

  return (
    <PDFDocument>
      <TitlePage
        title={
          lang === 'en'
            ? 'Tracking multiple VR users in a shared physical space'
            : 'Sledování více uživatelů VR světa ve sdíleném fyzickém prostoru'
        }
      />
      <Page style={{ alignItems: 'center', justifyContent: 'flex-end' }}>
        <LMText fontFamily="lmroman10-regular">
          Page intentionally left blank
        </LMText>
      </Page>

      <Page>
        {parsed.asts.map((ast, index) => (
          <MarkdownChapter index={index + 1} key={chapters[index][0]}>
            {ast}
          </MarkdownChapter>
        ))}
        <Literature
          citations={Object.entries(citations)
            .map(([id, value]) => {
              const index = parsed.citeMap[id]
              if (!index) return null
              return { ...value, id, index }
            })
            .filter(notNull)}
        />
      </Page>
    </PDFDocument>
  )
}
