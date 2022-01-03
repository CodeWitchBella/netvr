import pdf from '@react-pdf/renderer'
import type { Style } from '@react-pdf/types/style'
import { createContext, PropsWithChildren, useContext } from 'react'

export function Page({
  children,
  style,
  ...rest
}: PropsWithChildren<Omit<pdf.PageProps, 'size'>>) {
  return (
    <pdf.Page
      {...rest}
      size="A4"
      style={[
        {
          paddingHorizontal: '32mm',
          paddingBottom: '30mm',
          paddingTop: '35mm',
        },
        ...[style ?? []],
      ].flat()}
    >
      {children}
    </pdf.Page>
  )
}

export function TechnikaText({ children, style, ...rest }: pdf.TextProps) {
  return (
    <pdf.Text
      {...rest}
      style={[{ fontFamily: 'Technika', fontSize: 11 } as Style]
        .concat(style ?? [])
        .flat()}
    >
      {children}
    </pdf.Text>
  )
}

export type PDFContext = {
  lang: 'cs' | 'en'
  production: boolean
  files: { [key: string]: string }
}
const context = createContext<PDFContext>({
  lang: 'en',
  production: false,
  files: {},
})
export function usePDFContext() {
  return useContext(context)
}

export const PDFContextProvider = context.Provider

export const View = pdf.View
