import test from './test.md?raw'
import technicalDesign from './technical-design.md?raw'

export const chapters: readonly (readonly [string, string])[] = Object.entries({
  test,
  technicalDesign,
})

export type Citation = { url: string }

export const citations: { [key: string]: Citation } = {
  'steam-hardware-survey': {
    url: 'https://www.roadtovr.com/steam-survey-vr-headsets-on-steam-data-july-2021/',
  },
}
