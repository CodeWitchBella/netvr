import test from './test.md?raw'
import technicalDesign from './technical-design.md?raw'
import chapter1 from './1-introduction.md?raw'
import chapter2 from './2-analysis.md?raw'
import chapter3 from './3-architecture.md?raw'
import chapter4 from './4-accuracy.md?raw'
import bib from './bibliography.json?raw'
import chartSvg from './chart.svg?raw'

export const chapters: readonly (readonly [
  id: string,
  data: string,
  extra?: { removeInProduction: boolean },
])[] = [
  ['introduction', chapter1],
  ['analysis', chapter2],
  ['architecture', chapter3],
  ['accuracy', chapter4],
  ['technical-design', technicalDesign],
  ['test', test, { removeInProduction: true }],
]

export const files = {
  'quest2-optitrack.png': new URL('quest2-optitrack.png', import.meta.url).href,
  'chart.svg': chartSvg,
}

export const bibliography = JSON.parse(bib).references
