import pdf from '@react-pdf/renderer'
import { createContext, PropsWithChildren, useContext, useMemo } from 'react'
import { TechnikaText, usePDFContext, View } from './base'
import { colors } from './colors'
import { LMText } from './font'
import type { Style } from '@react-pdf/types/style'

export function Paragraph({
  children,
  first,
}: PropsWithChildren<{ first?: boolean }>) {
  return (
    <LMText
      fontFamily="lmroman10-regular"
      style={{
        fontSize: 11,
        textIndent: first ? 0 : '4.338mm',
        textAlign: 'justify',
      }}
    >
      {children}
    </LMText>
  )
}

export function Link({
  children,
  style,
  setFont = false,
  ...rest
}: pdf.LinkProps & { setFont?: boolean }) {
  return (
    <pdf.Link
      {...rest}
      style={[
        {
          textDecoration: 'none',
          color: 'black',
          fontFamily: setFont ? 'lmmono10-regular' : undefined,
        } as Style,
      ]
        .concat(style ?? [])
        .flat()}
    >
      {children}
    </pdf.Link>
  )
}

export function TODO({ children }: PropsWithChildren<{}>) {
  const ctx = usePDFContext()
  if (ctx.production) return null
  return (
    <LMText fontFamily="lmromanslant10-regular" style={{ fontSize: 11 }}>
      <Strong>TODO: </Strong>
      {children}
    </LMText>
  )
}

export function Em({ children }: PropsWithChildren<{}>) {
  return (
    <LMText fontFamily="lmroman10-italic" style={{ fontSize: 11 }}>
      {children}
    </LMText>
  )
}

export function Strong({ children }: PropsWithChildren<{}>) {
  return (
    <LMText fontFamily="lmroman10-bold" style={{ fontSize: 11 }}>
      {children}
    </LMText>
  )
}

const chapterContext = createContext({ no: 0, id: '' })

export function ChapterProvider({
  index,
  id,
  children,
}: PropsWithChildren<{ index: number; id: string }>) {
  return (
    <chapterContext.Provider
      value={useMemo(() => ({ no: index, id }), [index, id])}
    >
      {children}
    </chapterContext.Provider>
  )
}

export function Chapter({
  children,
  title,
  ...rest
}: PropsWithChildren<{ title: string; no?: number; id?: string }>) {
  const ctx = useContext(chapterContext)
  const pdfContext = usePDFContext()
  const no = rest.no ?? ctx.no
  const id = rest.id ?? ctx.id
  const childContext = useMemo(() => ({ id, no }), [id, no])
  return (
    <>
      <View
        id={'chapter-' + id}
        break={no !== 1}
        wrap={false}
        style={{
          position: 'relative',
          flexDirection: 'row',
          alignItems: 'flex-end',
          marginBottom: '5mm',
        }}
      >
        <View
          style={{
            width: '4mm',
            height: '21mm',
            marginRight: '4.5mm',
            backgroundColor: colors.blue,
          }}
        />
        {pdfContext.production ? null : (
          <View style={{ position: 'absolute', top: 0, left: 0, right: 0 }}>
            <TechnikaText style={{ textAlign: 'right' }}>
              #chapter-{id}
            </TechnikaText>
          </View>
        )}
        <TechnikaText
          style={{
            fontWeight: 'bold',
            fontSize: 17.5,
            marginBottom: '1.5mm',
          }}
        >
          {no ? (
            <>
              Chapter <pdf.Text style={{ fontSize: 30 }}>{no}</pdf.Text>
            </>
          ) : null}
          {'\n'}
          <pdf.Text style={{ color: colors.blue }}>{title}</pdf.Text>
        </TechnikaText>
      </View>
      <chapterContext.Provider value={childContext}>
        {children}
      </chapterContext.Provider>
    </>
  )
}

const sectionContext = createContext(0)

export function Section({
  children,
  title,
  no = 0,
}: PropsWithChildren<{ title: string; no?: number }>) {
  return (
    <View style={{ marginTop: '7.8mm' }}>
      <View
        wrap={false}
        minPresenceAhead={10}
        style={{
          flexDirection: 'row',
          alignItems: 'flex-end',
          marginBottom: '3mm',
          borderColor: colors.blue,
          borderLeftWidth: '4mm',
          height: '7mm',
          paddingLeft: '4.5mm',
          paddingBottom: '0.5mm',
        }}
      >
        {no ? (
          <>
            <TechnikaText
              style={{
                fontWeight: 'bold',
                fontSize: 14.52,
              }}
            >
              {useContext(chapterContext).no}.{no}
            </TechnikaText>
            <View style={{ width: '5.4mm' }} />
          </>
        ) : null}
        <TechnikaText
          style={{
            fontWeight: 'bold',
            fontSize: 14.52,
            color: colors.blue,
          }}
        >
          {title}
        </TechnikaText>
      </View>
      <sectionContext.Provider value={no}>{children}</sectionContext.Provider>
    </View>
  )
}

export function Footnote() {
  return <View style={{ position: 'absolute' }}></View>
}

export function SubSection({
  children,
  title,
  no = 0,
}: PropsWithChildren<{ title: string; no: number }>) {
  const chapter = useContext(chapterContext)
  const section = useContext(sectionContext)

  return (
    <View style={{ marginTop: '7.8mm' }}>
      <View
        wrap={false}
        minPresenceAhead={10}
        style={{
          flexDirection: 'row',
          alignItems: 'flex-end',
          marginBottom: '1.7mm',
          borderColor: colors.blue,
          borderLeftWidth: '4mm',
          minHeight: '3.8mm',
          paddingLeft: '4.5mm',
        }}
      >
        {no ? (
          <>
            <TechnikaText style={{ fontWeight: 'bold', fontSize: 12 }}>
              {chapter.no}.{section}.{no}
            </TechnikaText>
            <View style={{ width: '5.4mm' }} />
          </>
        ) : null}
        <TechnikaText
          style={{
            fontWeight: 'bold',
            fontSize: 12,
            color: colors.blue,
          }}
        >
          {title}
        </TechnikaText>
      </View>
      {children}
    </View>
  )
}

export function Image({
  src,
  description,
  index,
  title,
}: {
  src: string
  description: string
  index: number
  title: string
}) {
  const ctx = usePDFContext()
  const chapter = useContext(chapterContext)
  return (
    <View id={'figure-' + title}>
      <pdf.Image src={src} />
      <LMText
        fontFamily="lmroman10-regular"
        style={{ fontSize: 11, textAlign: 'justify' }}
      >
        <TechnikaText>
          {ctx.lang === 'en' ? 'Figure' : 'Obrázek'} {chapter.no}.{index}:{' '}
        </TechnikaText>
        {description}
      </LMText>
    </View>
  )
}

export function ImageRef({ number, title }: { number: number; title: string }) {
  const chapter = useContext(chapterContext)
  return (
    <Link href={'#figure-' + title} setFont={false}>
      {chapter.no + '.' + number}
    </Link>
  )
}

export function SplitView({
  leftTitle,
  rightTitle,
  left,
  right,
}: {
  leftTitle: string
  rightTitle: string
  left: JSX.Element
  right: JSX.Element
}) {
  const headerStyle = {
    fontSize: 19,
    color: colors.blue,
    fontWeight: 'bold' as const,
    marginBottom: '6.5mm',
  }
  return (
    <View style={{ flexDirection: 'row', height: '100%', width: '100%' }}>
      <View style={{ flexGrow: 1, flexBasis: 1 }}>
        <TechnikaText style={[headerStyle, { alignSelf: 'flex-end' }]}>
          {leftTitle}
        </TechnikaText>
        {left}
      </View>
      <View
        style={{
          width: '11mm',
          flexGrow: 0,
          flexShrink: 0,
          paddingHorizontal: '3.5mm',
          alignItems: 'center',
        }}
      >
        <TechnikaText style={headerStyle}>/</TechnikaText>
        <View
          style={{ flexGrow: 1, width: '4mm', backgroundColor: colors.blue }}
        />
      </View>
      <View style={{ flexGrow: 1, flexBasis: 1 }}>
        <TechnikaText style={headerStyle}>{rightTitle}</TechnikaText>
        {right}
      </View>
    </View>
  )
}