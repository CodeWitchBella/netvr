import test from './test.md?raw'
import technicalDesign from './technical-design.md?raw'

export const chapters: readonly (readonly [string, string])[] = Object.entries({
  test,
  technicalDesign,
})
