import pdf, { PDFViewer } from '@react-pdf/renderer'
import { Page, TechnikaText, usePDFContext, View } from './base'
import {
  ChapterProvider,
  Link,
  Paragraph,
  SplitView,
  Strong,
  TODO,
} from './design'
import { LMText, registerFonts } from './font'
import { TitlePage } from './title-page'
const { Document: PDFDocument } = pdf
import { ReactMarkdown, markdownListToAst } from './react-markdown'
import { BibReference, References } from './references'
import { FootnoteRenderer } from './footnotes'
import { Fragment } from 'react'
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
    | 'references'
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
        const chapter =
          typeof chapterIn === 'string'
            ? ([
                chapterIn,
                null,
                { appendix: true, removeInProduction: false },
              ] as const)
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

export function Document({
  bibliography,
  chapters,
  onlyChapter,
}: DocumentProps) {
  registerFonts()
  const { lang, production } = usePDFContext()

  const parsed = chapterParser(chapters, { production })

  const unused = Object.entries(bibliography)
    .filter(([key]) => !(key in parsed.citeMap))
    .map(([id, value]) => ({ ...value, id }))

  let titles = [
    'Tracking multiple VR users in a shared physical space',
    'Sledování více uživatelů VR světa ve sdíleném fyzickém prostoru',
  ]
  if (lang === 'cs') titles = titles.reverse()

  return (
    <PDFDocument>
      {onlyChapter && onlyChapter !== 'technical' ? null : (
        <>
          <TitlePage title={titles[0]} />
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
              <TODO>Insert official zadání</TODO>
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
            <pdf.View break={true}>
              <LMText fontFamily="lmroman10-regular">
                Page intentionally left blank
              </LMText>
            </pdf.View>
            <View break={true} />
            <SplitView
              leftTitle="Acknowledgements"
              rightTitle="Declaration"
              left={
                <View>
                  <Paragraph>
                    I would like to express my gratitude to my supervisor, Ing.
                    David Sedláček, Ph.D., for his guidance over the course of
                    writing semester project and later of this thesis. I would
                    also like to thank him and the Department of Computer
                    Graphics and Interaction for giving me access to equipment I
                    needed for the completion of this thesis.
                  </Paragraph>
                </View>
              }
              right={
                <View>
                  <Paragraph>
                    I declare that this thesis represents my work and that I
                    have listed all the literature used in the bibliography.
                  </Paragraph>
                  <Paragraph>Prague, 20 June 2022</Paragraph>
                  <View style={{ height: '1cm' }} />
                  <Paragraph>
                    Prohlašuji, že jsem předloženou práci vypracovala
                    samostatně, a že jsem uvedla veškerou použitou literaturu.
                  </Paragraph>
                  <Paragraph>V Praze, 20. června 2022</Paragraph>
                  <TODO>Update dates, both of them</TODO>
                </View>
              }
            />
            <SplitView
              rightTitle="Abstrakt"
              leftTitle="Abstract"
              right={
                <View>
                  <Paragraph>
                    (placeholder from blabot.cz) Vyšla ať té příslušník světa
                    nové prozkoumány struktury o ságy rok místnost naši a bude
                    superstrun jídelny i výstavě od one náš vlhkost pódia
                    velkým. Tím slunce drží níž i básník zradit
                    sedmikilometrového sledování vláknité a multi-dimenzionálním
                    systematicky. Jednom, 80 ℃ kratší ptal ně indickým životním
                    přetlakovaný i větší vystoupám tím instituce o hladinou šest
                    psychologických starosta. Amoku kroje v nejprve i sociální
                    existuje minerálů s potvrzují rozvoji, vznikly pás tisíc
                    představ klidné kdysi by správní nadšenců hlavě. Nejméně jí
                    tu chobotnice skákat, oxidu cíl posílily vláken testům. Z
                    oprášil plyne vědecké třetí jednom ani itálie ekologickou
                    ohrožení objeveny s vodorovně chorvati, o teoretickým snila
                    nejlepší z predátorů kterou z zlata božská kanadské. Horečky
                    k národností! Či EU sága vrátit řadu mohlo z svědčí spouští
                    testy o zápory? Odlišné nebo marná mám, běžnou kontinentu.
                    Ně ze včera vlna cestou po polopotopenou. Ať okouzlí, hlavní
                    klecích zkoušet dosahu s s historkám ochlazení, mým pomocí
                    od petr rozloučím slunečního skončení kostely, každý pravdou
                    tj. 1963–1977 starala o reprezentační.
                  </Paragraph>
                  <TODO>Replace blabot.cz output with real abstract</TODO>
                  <View style={{ height: '3mm' }} />
                  <Paragraph>
                    <Strong>Klíčová slova:</Strong> VR, Stuff, Things{' '}
                  </Paragraph>
                  <TODO>keywords</TODO>
                  <View style={{ height: '3mm' }} />
                  <Paragraph>
                    <Strong>Překlad názvu:</Strong> {titles[1]}
                  </Paragraph>
                </View>
              }
              left={
                <View>
                  <Paragraph>
                    (placeholder, real abstract will be provided for the thesis)
                    Lorem ipsum dolor sit amet, consectetur adipiscing elit.
                    Etiam commodo orci imperdiet volutpat malesuada. Vestibulum
                    quis massa tristique, lobortis arcu quis, pharetra ligula.
                    Aliquam vestibulum metus eget sapien porta laoreet. Sed ut
                    posuere urna. Sed quis mi hendrerit, cursus ligula in,
                    luctus tellus. Integer rhoncus, mauris in eleifend volutpat,
                    arcu elit semper ante, a luctus nisi metus id odio.
                    Suspendisse potenti. Aliquam nec ante eget arcu sollicitudin
                    vehicula. Aliquam faucibus, lorem pulvinar tristique
                    blandit, ipsum nunc eleifend est, at feugiat velit quam vel
                    ante. Sed sapien libero, volutpat sed mauris quis, molestie
                    iaculis risus. Morbi pharetra, mi in fermentum vehicula,
                    nisi ipsum ullamcorper turpis, finibus feugiat lectus eros
                    nec turpis. Cras in orci ligula. Aenean sagittis, velit in
                    ultricies lobortis, augue justo tempus orci, sit amet
                    iaculis tellus elit vitae nunc. Etiam elementum sollicitudin
                    lorem, eget ornare erat bibendum non.
                  </Paragraph>
                  <TODO>Replace lorem ipsum with real abstract</TODO>
                  <View style={{ height: '3mm' }} />
                  <Paragraph>
                    <Strong>Keywords:</Strong> VR, Stuff, Things
                  </Paragraph>
                  <TODO>keywords</TODO>
                </View>
              }
            />
            <SplitView
              rightTitle=""
              leftTitle="Contents"
              right={<View />}
              left={
                <View>
                  {parsed.asts.map((chapter, index) => {
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
          </Page>
        </>
      )}

      <Page>
        <FootnoteRenderer>
          {parsed.asts.map((chapter, index) => {
            const { ast, id, denomination } = chapter
            if (onlyChapter && id !== onlyChapter) return null

            return (
              <Fragment key={id}>
                <pdf.View break={!onlyChapter && index !== 0} />
                {ast ? (
                  <MarkdownChapter data={chapter} />
                ) : id === 'references' ? (
                  <References
                    citations={Object.entries(parsed.citeMap)
                      .sort(([_1, a], [_2, b]) => a - b)
                      .map(([id, index]) => {
                        return { data: bibliography[id], id, index }
                      })}
                    unused={unused}
                    denomination={denomination}
                  />
                ) : null}
              </Fragment>
            )
          })}
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
    </PDFDocument>
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
