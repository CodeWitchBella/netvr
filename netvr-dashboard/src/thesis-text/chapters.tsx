import test from './test.md?raw'
import technicalDesign from './technical-design.md?raw'
import chapter1 from './1-introduction.md?raw'
import chapter2 from './2-analysis.md?raw'
import bib from './bibliography.json?raw'

export const chapters: readonly (readonly [
  id: string,
  data: string,
  extra?: { removeInProduction: boolean },
])[] = [
  ['introduction', chapter1],
  ['analysis', chapter2],
  ['technical-design', technicalDesign],
  ['test', test, { removeInProduction: true }],
]

export type Reference = {
  url: string
  authors?: readonly (
    | { firstname: string; surname: string }
    | { group: string }
  )[]
  title?: string
  subtitle?: string
  date?: number | string
  edition?: string
  location?: string
  publisher?: string
  in?: string
  accessed?: string
}

export const bibliography: { [key: string]: Reference } =
  JSON.parse(bib).references
