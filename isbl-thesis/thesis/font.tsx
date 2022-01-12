import pdf from '@react-pdf/renderer'
import type { Style } from '@react-pdf/types/style'
// @ts-ignore
import { hellvetica, lmdb, technika } from '../assets.js'
const { Font } = pdf

let registered = false

export function registerFonts() {
  if (registered) return
  registered = true

  Font.registerEmojiSource({
    format: 'png',
    url: 'https://twemoji.maxcdn.com/2/72x72/',
  })
  Font.register({
    family: 'Technika',
    fonts: technika,
  })
  Font.register({
    family: 'Hellvetica',
    src: hellvetica,
  })

  for (const [longName, src] of Object.entries(lmdb)) {
    Font.register({ family: longName, src, format: 'otf' })

    const [family, mod] = longName.split('-')
    Font.register({
      family,
      src,
      fontStyle: mod.includes('oblique')
        ? 'oblique'
        : mod.includes('italic')
        ? 'italic'
        : 'regular',
      format: 'otf',
      fontWeight: mod.includes('bold') ? 'bold' : 'normal',
    })
  }
}

export function LMText({
  children,
  style,
  fontFamily,
  automaticFontSize = true,
  ...rest
}: pdf.TextProps & {
  fontFamily: keyof typeof assets['lmdb']
  automaticFontSize?: boolean
}) {
  return (
    <pdf.Text
      {...rest}
      style={[
        {
          fontFamily: fontFamily,
          fontSize: automaticFontSize
            ? parseInt(fontFamily.replace(/[^0-9]/g, ''), 10) || undefined
            : undefined,
        } as Style,
      ]
        .concat(style ?? [])
        .flat()}
    >
      {children}
    </pdf.Text>
  )
}
