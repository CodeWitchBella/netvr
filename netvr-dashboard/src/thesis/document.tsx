import pdf from '@react-pdf/renderer'
import { Page, usePDFContext } from './base'
import { ChapterProvider, TODO } from './design'
import { LMText, registerFonts } from './font'
import { TitlePage } from './title-page'
const { Document: PDFDocument } = pdf
import { ReactMarkdown, markdownListToAst } from './react-markdown'
import { chapters, bibliography } from '../thesis-text/chapters'
import { References } from './references'
import { notNull } from '@isbl/ts-utils'

function MarkdownChapter({
  children,
  index,
  id,
}: {
  children: any
  index: number
  id: string
}) {
  return (
    <ChapterProvider index={index} id={id}>
      <ReactMarkdown hast={children} />
    </ChapterProvider>
  )
}

export function Document() {
  registerFonts()
  const { lang, production } = usePDFContext()

  const usedChapters = chapters.filter(
    (c) => !production || !c[2]?.removeInProduction,
  )

  const parsed = markdownListToAst(usedChapters.map(([key, text]) => text))

  const unused = Object.entries(bibliography)
    .filter(([key]) => !(key in parsed.citeMap))
    .map(([id, value]) => ({ ...value, id }))

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
        <pdf.View
          fixed
          style={{
            position: 'absolute',
            bottom: '2cm',
            left: '32mm',
            right: '32mm',
            alignItems: 'center',
          }}
          render={({ pageNumber }) => (
            <pdf.Text style={{ fontFamily: 'lmroman10-regular', fontSize: 11 }}>
              {
                ['i', 'ii', 'iii', 'iv', 'v', 'vi', 'vii', 'viii', 'ix', 'x'][
                  pageNumber - 1
                ]
              }
            </pdf.Text>
          )}
        />
        <pdf.View break={true}>
          <TODO>Insert official zadání</TODO>
          <LMText fontFamily="lmroman10-regular" style={{ fontSize: 11 }}>
            1) Proveďte rešerši technik a postupů umožňujících sledování více
            uživatelů virtuální reality (VR) ve sdíleném fyzickém prostředí.
            {'\n'}2) Vyberte vhodné metody s ohledem na dostupné zařízení a
            náročnost nastavení systému.{'\n'}3) Vybrané metody implementujte a
            porovnejte s referenčním stabilním systémem (např. optické sledování
            systémem Vicon / Optitrack či podobnými).{'\n'}4) Důraz bude kladen
            na bezpečnost uživatelů VR světa. {'\n\n'}-- rozsireni pro DP{'\n'}
            nejlepe - najit diru v resenich a vyresit ji :){'\n'}pri nejhorsim -
            vymyslet UseCase a aplikovat to na nej pro TEMVR pripravit tutorial
            na jedno cviceni (side efect DP)
          </LMText>
        </pdf.View>
        <pdf.View break={true}>
          <LMText fontFamily="lmroman10-regular">
            Page intentionally left blank
          </LMText>
        </pdf.View>
      </Page>

      <Page>
        {parsed.asts.map((ast, index) => {
          const id = usedChapters[index][0]
          return (
            <MarkdownChapter index={index + 1} key={id} id={id}>
              {ast}
            </MarkdownChapter>
          )
        })}
        <References
          citations={Object.entries(parsed.citeMap)
            .sort(([_1, a], [_2, b]) => a - b)
            .map(([id, index]) => {
              return { data: bibliography[id], id, index }
            })}
          unused={unused}
        />
        <pdf.View
          fixed
          style={{
            position: 'absolute',
            bottom: '2cm',
            left: '32mm',
            right: '32mm',
            alignItems: 'center',
          }}
          render={({ pageNumber }) => (
            <pdf.Text style={{ fontFamily: 'lmroman10-regular', fontSize: 11 }}>
              {pageNumber}
            </pdf.Text>
          )}
        />
      </Page>
    </PDFDocument>
  )
}
