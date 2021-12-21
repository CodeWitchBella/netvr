import pdf from '@react-pdf/renderer'
import { createContext, PropsWithChildren, useContext } from 'react'
import { Page, TechnikaText } from './base'
import { colors } from './colors'
import { LMText } from './font'

export function Paragraph({
  children,
  first,
  title,
}: PropsWithChildren<{ first?: boolean; title?: string }>) {
  return (
    <LMText
      fontFamily="lmroman10-regular"
      style={{
        fontSize: 11,
        textIndent: first ? 0 : '4.338mm',
        marginTop: title ? '2mm' : 0,
      }}
    >
      {title ? <ParagraphTitle>{title}</ParagraphTitle> : null}
      {children}
    </LMText>
  )
}

export function TODO({ children }: PropsWithChildren<{}>) {
  return (
    <LMText fontFamily="lmromanslant10-regular" style={{ fontSize: 11 }}>
      <ParagraphTitle>TODO:</ParagraphTitle>
      {children}
    </LMText>
  )
}

export function ParagraphTitle({ children }: PropsWithChildren<{}>) {
  return (
    <LMText fontFamily="lmroman10-bold" style={{ fontSize: 11 }}>
      {children}{' '}
    </LMText>
  )
}

const chapterContext = createContext(0)

export function Chapter({
  children,
  title,
  no = 0,
}: PropsWithChildren<{ title: string; no?: number }>) {
  return (
    <Page>
      <pdf.View
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
    </Page>
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
        style={{
          flexDirection: 'row',
          alignItems: 'flex-end',
          marginBottom: '3mm',
        }}
      >
        <pdf.View
          style={{
            width: '4mm',
            height: '7mm',
            marginRight: '4.5mm',
            backgroundColor: colors.blue,
          }}
        />
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
