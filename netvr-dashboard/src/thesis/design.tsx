import pdf from '@react-pdf/renderer'
import { createContext, PropsWithChildren, useContext, useMemo } from 'react'
import { Page, TechnikaText } from './base'
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
  no,
  id,
}: PropsWithChildren<{ title: string; no?: number; id?: string }>) {
  const ctx = useContext(chapterContext)
  no = no ?? ctx.no
  id = id ?? ctx.id
  return (
    <>
      <pdf.View
        id={'chapter-' + id}
        break={no !== 1}
        wrap={false}
        style={{
          flexDirection: 'row',
          alignItems: 'flex-end',
          marginBottom: '5mm',
        }}
      >
        <pdf.View
          style={{
            width: '4mm',
            height: '21mm',
            marginRight: '4.5mm',
            backgroundColor: colors.blue,
          }}
        />
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
      </pdf.View>
      <chapterContext.Provider value={no}>{children}</chapterContext.Provider>
    </>
  )
}

export function Section({
  children,
  title,
  no = 0,
}: PropsWithChildren<{ title: string; no?: number }>) {
  return (
    <pdf.View style={{ marginTop: '7.8mm' }}>
      <pdf.View
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
              {useContext(chapterContext)}.{no}
            </TechnikaText>
            <pdf.View style={{ width: '5.4mm' }} />
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
      </pdf.View>
      {children}
    </pdf.View>
  )
}

export function SplitPage() {}
