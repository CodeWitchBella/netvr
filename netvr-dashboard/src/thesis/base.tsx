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
        { paddingHorizontal: '32mm', paddingVertical: '30mm' },
        ...[style ?? []],
      ].flat()}
    >
      {children}
    </pdf.Page>
  )
}

export function Text({ children, style, ...rest }: pdf.TextProps) {
  return (
    <pdf.Text
      {...rest}
      style={[{ fontFamily: 'Technika' } as Style].concat(style ?? []).flat()}
    >
      {children}
    </pdf.Text>
  )
}

const context = createContext<{ lang: 'cs' | 'en' }>({ lang: 'en' })
export function usePDFContext() {
  return useContext(context)
}

export const PDFContextProvider = context.Provider
