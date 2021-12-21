import pdf from '@react-pdf/renderer'
import type { Style } from '@react-pdf/types/style'
import { TechnikaText } from './base'
const { Font } = pdf

let registered = false

// https://www.ctan.org/tex-archive/fonts/lm/
const lmdb = {
  'lmmono10-italic': new URL('./lm/lmmono10-italic.otf', import.meta.url),
  'lmmono10-regular': new URL('./lm/lmmono10-regular.otf', import.meta.url),
  'lmmono12-regular': new URL('./lm/lmmono12-regular.otf', import.meta.url),
  'lmmono8-regular': new URL('./lm/lmmono8-regular.otf', import.meta.url),
  'lmmono9-regular': new URL('./lm/lmmono9-regular.otf', import.meta.url),
  'lmmonocaps10-oblique': new URL(
    './lm/lmmonocaps10-oblique.otf',
    import.meta.url,
  ),
  'lmmonocaps10-regular': new URL(
    './lm/lmmonocaps10-regular.otf',
    import.meta.url,
  ),
  'lmmonolt10-bold': new URL('./lm/lmmonolt10-bold.otf', import.meta.url),
  'lmmonolt10-boldoblique': new URL(
    './lm/lmmonolt10-boldoblique.otf',
    import.meta.url,
  ),
  'lmmonolt10-oblique': new URL('./lm/lmmonolt10-oblique.otf', import.meta.url),
  'lmmonolt10-regular': new URL('./lm/lmmonolt10-regular.otf', import.meta.url),
  'lmmonoltcond10-oblique': new URL(
    './lm/lmmonoltcond10-oblique.otf',
    import.meta.url,
  ),
  'lmmonoltcond10-regular': new URL(
    './lm/lmmonoltcond10-regular.otf',
    import.meta.url,
  ),
  'lmmonoprop10-oblique': new URL(
    './lm/lmmonoprop10-oblique.otf',
    import.meta.url,
  ),
  'lmmonoprop10-regular': new URL(
    './lm/lmmonoprop10-regular.otf',
    import.meta.url,
  ),
  'lmmonoproplt10-bold': new URL(
    './lm/lmmonoproplt10-bold.otf',
    import.meta.url,
  ),
  'lmmonoproplt10-boldoblique': new URL(
    './lm/lmmonoproplt10-boldoblique.otf',
    import.meta.url,
  ),
  'lmmonoproplt10-oblique': new URL(
    './lm/lmmonoproplt10-oblique.otf',
    import.meta.url,
  ),
  'lmmonoproplt10-regular': new URL(
    './lm/lmmonoproplt10-regular.otf',
    import.meta.url,
  ),
  'lmmonoslant10-regular': new URL(
    './lm/lmmonoslant10-regular.otf',
    import.meta.url,
  ),
  'lmroman10-bold': new URL('./lm/lmroman10-bold.otf', import.meta.url),
  'lmroman10-bolditalic': new URL(
    './lm/lmroman10-bolditalic.otf',
    import.meta.url,
  ),
  'lmroman10-italic': new URL('./lm/lmroman10-italic.otf', import.meta.url),
  'lmroman10-regular': new URL('./lm/lmroman10-regular.otf', import.meta.url),
  'lmroman12-bold': new URL('./lm/lmroman12-bold.otf', import.meta.url),
  'lmroman12-italic': new URL('./lm/lmroman12-italic.otf', import.meta.url),
  'lmroman12-regular': new URL('./lm/lmroman12-regular.otf', import.meta.url),
  'lmroman17-regular': new URL('./lm/lmroman17-regular.otf', import.meta.url),
  'lmroman5-bold': new URL('./lm/lmroman5-bold.otf', import.meta.url),
  'lmroman5-regular': new URL('./lm/lmroman5-regular.otf', import.meta.url),
  'lmroman6-bold': new URL('./lm/lmroman6-bold.otf', import.meta.url),
  'lmroman6-regular': new URL('./lm/lmroman6-regular.otf', import.meta.url),
  'lmroman7-bold': new URL('./lm/lmroman7-bold.otf', import.meta.url),
  'lmroman7-italic': new URL('./lm/lmroman7-italic.otf', import.meta.url),
  'lmroman7-regular': new URL('./lm/lmroman7-regular.otf', import.meta.url),
  'lmroman8-bold': new URL('./lm/lmroman8-bold.otf', import.meta.url),
  'lmroman8-italic': new URL('./lm/lmroman8-italic.otf', import.meta.url),
  'lmroman8-regular': new URL('./lm/lmroman8-regular.otf', import.meta.url),
  'lmroman9-bold': new URL('./lm/lmroman9-bold.otf', import.meta.url),
  'lmroman9-italic': new URL('./lm/lmroman9-italic.otf', import.meta.url),
  'lmroman9-regular': new URL('./lm/lmroman9-regular.otf', import.meta.url),
  'lmromancaps10-oblique': new URL(
    './lm/lmromancaps10-oblique.otf',
    import.meta.url,
  ),
  'lmromancaps10-regular': new URL(
    './lm/lmromancaps10-regular.otf',
    import.meta.url,
  ),
  'lmromandemi10-oblique': new URL(
    './lm/lmromandemi10-oblique.otf',
    import.meta.url,
  ),
  'lmromandemi10-regular': new URL(
    './lm/lmromandemi10-regular.otf',
    import.meta.url,
  ),
  'lmromandunh10-oblique': new URL(
    './lm/lmromandunh10-oblique.otf',
    import.meta.url,
  ),
  'lmromandunh10-regular': new URL(
    './lm/lmromandunh10-regular.otf',
    import.meta.url,
  ),
  'lmromanslant10-bold': new URL(
    './lm/lmromanslant10-bold.otf',
    import.meta.url,
  ),
  'lmromanslant10-regular': new URL(
    './lm/lmromanslant10-regular.otf',
    import.meta.url,
  ),
  'lmromanslant12-regular': new URL(
    './lm/lmromanslant12-regular.otf',
    import.meta.url,
  ),
  'lmromanslant17-regular': new URL(
    './lm/lmromanslant17-regular.otf',
    import.meta.url,
  ),
  'lmromanslant8-regular': new URL(
    './lm/lmromanslant8-regular.otf',
    import.meta.url,
  ),
  'lmromanslant9-regular': new URL(
    './lm/lmromanslant9-regular.otf',
    import.meta.url,
  ),
  'lmromanunsl10-regular': new URL(
    './lm/lmromanunsl10-regular.otf',
    import.meta.url,
  ),
  'lmsans10-bold': new URL('./lm/lmsans10-bold.otf', import.meta.url),
  'lmsans10-boldoblique': new URL(
    './lm/lmsans10-boldoblique.otf',
    import.meta.url,
  ),
  'lmsans10-oblique': new URL('./lm/lmsans10-oblique.otf', import.meta.url),
  'lmsans10-regular': new URL('./lm/lmsans10-regular.otf', import.meta.url),
  'lmsans12-oblique': new URL('./lm/lmsans12-oblique.otf', import.meta.url),
  'lmsans12-regular': new URL('./lm/lmsans12-regular.otf', import.meta.url),
  'lmsans17-oblique': new URL('./lm/lmsans17-oblique.otf', import.meta.url),
  'lmsans17-regular': new URL('./lm/lmsans17-regular.otf', import.meta.url),
  'lmsans8-oblique': new URL('./lm/lmsans8-oblique.otf', import.meta.url),
  'lmsans8-regular': new URL('./lm/lmsans8-regular.otf', import.meta.url),
  'lmsans9-oblique': new URL('./lm/lmsans9-oblique.otf', import.meta.url),
  'lmsans9-regular': new URL('./lm/lmsans9-regular.otf', import.meta.url),
  'lmsansdemicond10-oblique': new URL(
    './lm/lmsansdemicond10-oblique.otf',
    import.meta.url,
  ),
  'lmsansdemicond10-regular': new URL(
    './lm/lmsansdemicond10-regular.otf',
    import.meta.url,
  ),
  'lmsansquot8-bold': new URL('./lm/lmsansquot8-bold.otf', import.meta.url),
  'lmsansquot8-boldoblique': new URL(
    './lm/lmsansquot8-boldoblique.otf',
    import.meta.url,
  ),
  'lmsansquot8-oblique': new URL(
    './lm/lmsansquot8-oblique.otf',
    import.meta.url,
  ),
  'lmsansquot8-regular': new URL(
    './lm/lmsansquot8-regular.otf',
    import.meta.url,
  ),
} as const

export function registerFonts() {
  if (registered) return
  registered = true

  Font.registerEmojiSource({
    format: 'png',
    url: 'https://twemoji.maxcdn.com/2/72x72/',
  })
  Font.register({
    family: 'Technika',
    fonts: [
      {
        src: new URL('./Technika-Bold.otf', import.meta.url).toString(),
        fontWeight: 'bold',
      },
      {
        src: new URL('./Technika-BoldItalic.otf', import.meta.url).toString(),
        fontWeight: 'bold',
        fontStyle: 'italic',
      },
      {
        src: new URL('./Technika-Book.otf', import.meta.url).toString(),
        fontWeight: 'light',
      },
      {
        src: new URL('./Technika-BookItalic.otf', import.meta.url).toString(),
        fontWeight: 'light',
        fontStyle: 'italic',
      },
      {
        src: new URL('./Technika-Italic.otf', import.meta.url).toString(),
        fontStyle: 'italic',
      },
      {
        src: new URL('./Technika-Regular.otf', import.meta.url).toString(),
      },
    ],
  })

  for (const [longName, src] of Object.entries(lmdb)) {
    Font.register({ family: longName, src: src.toString() })

    const [family, mod] = longName.split('-')
    Font.register({
      family,
      src: src.toString(),
      fontStyle: mod.includes('oblique')
        ? 'oblique'
        : mod.includes('italic')
        ? 'italic'
        : 'regular',
      fontWeight: mod.includes('bold') ? 'bold' : 'normal',
    })
  }
}

export function LMText({
  children,
  style,
  fontFamily,
  ...rest
}: pdf.TextProps & { fontFamily: keyof typeof lmdb }) {
  return (
    <pdf.Text
      {...rest}
      style={[
        {
          fontFamily: fontFamily,
          fontSize: parseInt(fontFamily.replace(/[^0-9]/g, ''), 10),
        } as Style,
      ]
        .concat(style ?? [])
        .flat()}
    >
      {children}
    </pdf.Text>
  )
}
