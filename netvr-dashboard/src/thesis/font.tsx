import pdf from '@react-pdf/renderer'
const { Font } = pdf
import TechnikaBold from './Technika-Bold.otf'
import TechnikaBoldItalic from './Technika-BoldItalic.otf'
import TechnikaBook from './Technika-Book.otf'
import TechnikaBookItalic from './Technika-BookItalic.otf'
import TechnikaItalic from './Technika-Italic.otf'
import TechnikaRegular from './Technika-Regular.otf'

let registered = false
export function registerFonts() {
  if (registered) return
  registered = true
  Font.register({
    family: 'Technika',
    src: TechnikaBold,
    fontWeight: 'bold',
  })
  Font.register({
    family: 'Technika',
    src: TechnikaBoldItalic,
    fontWeight: 'bold',
    fontStyle: 'italic',
  })
  Font.register({
    family: 'Technika',
    src: TechnikaBook,
    fontWeight: 'light',
  })
  Font.register({
    family: 'Technika',
    src: TechnikaBookItalic,
    fontWeight: 'light',
    fontStyle: 'italic',
  })
  Font.register({
    family: 'Technika',
    src: TechnikaItalic,
    fontStyle: 'italic',
  })
  Font.register({
    family: 'Technika',
    src: TechnikaRegular,
  })
}
