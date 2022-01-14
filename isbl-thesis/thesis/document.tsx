import pdf from '@react-pdf/renderer'
import { Page, TechnikaText, usePDFContext, View } from './base'
import { ChapterProvider, Link, SplitView, RefContextProvider } from './design'
import { LMText, registerFonts } from './font'
import { TitlePage } from './title-page'
const { Document: PDFDocument } = pdf
import { ReactMarkdown, markdownListToAst } from './react-markdown'
import { BibReference, Bibliography } from './bibliography'
import { FootnoteRenderer } from './footnotes'
import { Fragment, PropsWithChildren } from 'react'
import type * as hast from 'hast'
import { notNull } from '@isbl/ts-utils'

function MarkdownChapter({
  data,
}: {
  data: ReturnType<typeof chapterParser>['asts'][0]
}) {
  if (!data.ast) return null
  return (
    <ChapterProvider denomination={data.denomination ?? ''} id={data.id}>
      <ReactMarkdown hast={data.ast} />
    </ChapterProvider>
  )
}

export type DocumentProps = {
  bibliography: { [key: string]: BibReference }
  chapters: readonly (
    | readonly [
        id: string,
        data: string,
        extra?: { removeInProduction: boolean; appendix: boolean },
      ]
    | 'bibliography'
    | 'toc'
    | 'begin'
    | {
        id: string
        type: 'split'
        text: readonly [string, string]
        titles: readonly [string, string]
        removeInProduction?: boolean
      }
  )[]
  onlyChapter?: string
}

function chapterParser(
  chapters: DocumentProps['chapters'],
  { production }: { production: boolean },
) {
  let numericCounter = 1
  let letterCounter = 'A'.charCodeAt(0)

  return markdownListToAst(
    chapters
      .map((chapterIn) => {
        if (typeof chapterIn === 'object' && 'type' in chapterIn) {
          if (chapterIn.removeInProduction) return null
          return {
            text: null,
            id: chapterIn.id,
            type: chapterIn.type,
            asts: markdownListToAst(
              chapterIn.text.map((text) => ({ text })),
            ).asts.map((v) => v.ast),
            titles: chapterIn.titles,
          }
        }

        const chapter: readonly [
          id: string,
          data: string | null,
          extra?: { removeInProduction: boolean; appendix: boolean },
        ] =
          typeof chapterIn === 'string'
            ? [chapterIn, null, { appendix: true, removeInProduction: false }]
            : chapterIn

        if (production && chapter[2]?.removeInProduction) return null

        return {
          text: chapter[1],
          id: chapter[0],
          denomination: chapter[2]?.appendix
            ? String.fromCharCode(letterCounter++)
            : `${numericCounter++}`,
        }
      })
      .filter(notNull),
  )
}

function Wrapper({
  refContext,
  children,
}: PropsWithChildren<{ refContext: { [key: string]: string } }>) {
  return (
    <PDFDocument>
      <RefContextProvider value={refContext}>{children}</RefContextProvider>
    </PDFDocument>
  )
}

export function Document({
  bibliography,
  chapters,
  onlyChapter,
}: DocumentProps) {
  registerFonts()
  const { lang, production } = usePDFContext()

  let title = 'Tracking multiple VR users in a shared physical space'

  const beginIndex = chapters.indexOf('begin')
  const introChapters = chapterParser(chapters.slice(0, beginIndex), {
    production,
  })
  const contentChapters = chapterParser(chapters.slice(beginIndex + 1), {
    production,
  })

  return (
    <Wrapper refContext={contentChapters.refIds}>
      {onlyChapter && onlyChapter !== 'technical' ? null : (
        <TitlePage title={title} />
      )}
      <Page>
        {onlyChapter && onlyChapter !== 'technical' ? null : (
          <>
            <View
              style={{
                alignItems: 'center',
                justifyContent: 'flex-end',
                height: '99%',
              }}
            >
              <LMText fontFamily="lmroman10-regular">
                Page intentionally left blank
              </LMText>
            </View>
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
                <pdf.Text
                  style={{ fontFamily: 'lmroman10-regular', fontSize: 11 }}
                >
                  {
                    [
                      'i',
                      'ii',
                      'iii',
                      'iv',
                      'v',
                      'vi',
                      'vii',
                      'viii',
                      'ix',
                      'x',
                    ][pageNumber - 1]
                  }
                </pdf.Text>
              )}
            />
            <pdf.View break={true}>
              <TechnikaText style={{ fontSize: 20 }}>Zadání</TechnikaText>
              <LMText fontFamily="lmroman10-regular" style={{ fontSize: 11 }}>
                1) Proveďte rešerši technik a postupů umožňujících sledování
                více uživatelů virtuální reality (VR) ve sdíleném fyzickém
                prostředí.
                {'\n'}2) Vyberte vhodné metody s ohledem na dostupné zařízení a
                náročnost nastavení systému.{'\n'}3) Vybrané metody
                implementujte a porovnejte s referenčním stabilním systémem
                (např. optické sledování systémem Vicon / Optitrack či
                podobnými).{'\n'}4) Důraz bude kladen na bezpečnost uživatelů VR
                světa. {'\n\n'}-- rozsireni pro DP{'\n'}
                nejlepe - najit diru v resenich a vyresit ji :){'\n'}pri
                nejhorsim - vymyslet UseCase a aplikovat to na nej pro TEMVR
                pripravit tutorial na jedno cviceni (side efect DP)
              </LMText>
            </pdf.View>
            <pdf.View
              style={{
                alignItems: 'center',
                justifyContent: 'flex-end',
                height: '99%',
              }}
              break={true}
            >
              <LMText fontFamily="lmroman10-regular">
                Page intentionally left blank
              </LMText>
            </pdf.View>
            <pdf.View break={true} />
          </>
        )}
        <AstListRenderer
          data={introChapters.asts}
          onlyChapter={onlyChapter}
          bibliography={bibliography}
          citeMap={contentChapters.citeMap}
          contentChapters={contentChapters.asts}
        />
      </Page>

      <Page>
        <FootnoteRenderer>
          <AstListRenderer
            data={contentChapters.asts}
            onlyChapter={onlyChapter}
            bibliography={bibliography}
            citeMap={contentChapters.citeMap}
            contentChapters={contentChapters.asts}
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
              <pdf.Text
                style={{ fontFamily: 'lmroman10-regular', fontSize: 11 }}
              >
                {pageNumber}
              </pdf.Text>
            )}
          />
        </FootnoteRenderer>
      </Page>
    </Wrapper>
  )
}
function AstListRenderer({
  data,
  citeMap,
  onlyChapter,
  bibliography,
  contentChapters,
}: {
  data: ReturnType<typeof chapterParser>['asts']
  citeMap: ReturnType<typeof chapterParser>['citeMap']
  onlyChapter?: string
  bibliography: { [key: string]: BibReference }
  contentChapters: ReturnType<typeof chapterParser>['asts']
}) {
  return (
    <>
      {data.map((chapter, index) => {
        const { ast, id, denomination } = chapter
        if (onlyChapter && id !== onlyChapter) return null

        return (
          <Fragment key={id}>
            <pdf.View break={!onlyChapter && index !== 0} />
            {ast ? (
              <MarkdownChapter data={chapter} />
            ) : id === 'bibliography' ? (
              <Bibliography
                citations={Object.entries(citeMap)
                  .sort(([_1, a], [_2, b]) => a - b)
                  .map(([id, index]) => {
                    return { data: bibliography[id], id, index }
                  })}
                unused={Object.entries(bibliography)
                  .filter(([key]) => !(key in citeMap))
                  .map(([id, value]) => ({ ...value, id }))}
                denomination={denomination}
              />
            ) : id === 'toc' ? (
              <SplitView
                rightTitle=""
                leftTitle="Contents"
                right={<View />}
                left={
                  <View>
                    {contentChapters.map((chapter, index) => {
                      if (!chapter.denomination) return null
                      return (
                        <TableOfContentsChapter
                          key={chapter.id}
                          denomination={chapter.denomination}
                          chapter={chapter.ast}
                          id={chapter.id}
                        />
                      )
                    })}
                  </View>
                }
              />
            ) : chapter.asts ? (
              <SplitView
                left={<ReactMarkdown hast={chapter.asts[0]} />}
                right={<ReactMarkdown hast={chapter.asts[1]} />}
                leftTitle={chapter.titles?.[0] ?? ''}
                rightTitle={chapter.titles?.[1] ?? ''}
              />
            ) : (
              void console.log('unknown chapter type', chapter)
            )}
          </Fragment>
        )
      })}
    </>
  )
}

function getText(node: hast.ElementContent): string {
  if (node.type === 'text') return node.value
  if (node.type === 'element')
    return node.children.map((v) => getText(v)).join('')
  return ''
}

function findSections(
  node: hast.ElementContent | hast.Root | hast.RootContent,
  start = true,
): hast.Element[] {
  if (node.type === 'root') {
    return node.children.map((child) => findSections(child, false)).flat()
  }

  if (node.type === 'element') {
    if (node.tagName === 'section' && !start) return [node]
    return node.children.map((child) => findSections(child, false)).flat()
  }
  return []
}

function TableOfContentsChapter({
  chapter,
  id,
  denomination,
}: {
  chapter: hast.Root | null
  id: string
  denomination: string
}) {
  const rootSection = chapter
    ? findSections(chapter)[0] ?? null
    : {
        type: 'root' as const,
        children: [
          { type: 'text', value: id[0].toUpperCase() + id.slice(1) } as const,
        ],
      }
  if (!rootSection) return null
  return (
    <View>
      <Link src={`#${id}`}>
        <TechnikaText>
          {denomination} {getText(rootSection?.children?.[0])}
        </TechnikaText>
      </Link>
      <View style={{ paddingLeft: '5mm' }}>
        {findSections(rootSection).map((section, i) => {
          const sectionId: string | undefined = (section.children[0] as any)
            ?.properties?.id
          const sectionDenomination = `${denomination}.${i + 1}`

          return (
            <Link
              src={
                sectionId
                  ? '#' + sectionId
                  : '#automatic-' + sectionDenomination
              }
              key={i}
            >
              <TechnikaText key={i}>
                {sectionDenomination} {getText(section.children[0])}
              </TechnikaText>
            </Link>
          )
        })}
      </View>
    </View>
  )
}
