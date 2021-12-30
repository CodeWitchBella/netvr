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

export function Link({ children, style, ...rest }: pdf.LinkProps) {
  return (
    <pdf.Link
      {...rest}
      style={[
        {
          textDecoration: 'none',
          color: 'black',
          fontFamily: 'lmmono10-regular',
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
        }}
      >
        {no ? (
          <>
            <TechnikaText
              style={{
                fontWeight: 'bold',
                fontSize: 14.52,
                marginBottom: '0.5mm',
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
            marginBottom: '0.5mm',
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
