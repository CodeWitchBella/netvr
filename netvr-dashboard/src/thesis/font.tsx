import pdf from '@react-pdf/renderer'
const { Font } = pdf

let registered = false
export function registerFonts() {
  if (registered) return
  registered = true
  Font.register({
    family: 'Technika',
    src: new URL('./Technika-Bold.otf', import.meta.url).toString(),
    fontWeight: 'bold',
  })
  Font.register({
    family: 'Technika',
    src: new URL('./Technika-BoldItalic.otf', import.meta.url).toString(),
    fontWeight: 'bold',
    fontStyle: 'italic',
  })
  Font.register({
    family: 'Technika',
    src: new URL('./Technika-Book.otf', import.meta.url).toString(),
    fontWeight: 'light',
  })
  Font.register({
    family: 'Technika',
    src: new URL('./Technika-BookItalic.otf', import.meta.url).toString(),
    fontWeight: 'light',
    fontStyle: 'italic',
  })
  Font.register({
    family: 'Technika',
    src: new URL('./Technika-Italic.otf', import.meta.url).toString(),
    fontStyle: 'italic',
  })
  Font.register({
    family: 'Technika',
    src: new URL('./Technika-Regular.otf', import.meta.url).toString(),
  })
}
