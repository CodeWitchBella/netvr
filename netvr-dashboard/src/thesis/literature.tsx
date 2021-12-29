import type { Citation } from '../thesis-text/chapters'
import { Chapter, Link } from './design'
import pdf from '@react-pdf/renderer'
import { LMText } from './font'

export function Literature({
  citations,
}: {
  citations: readonly (Citation & { id: string; index: number })[]
}) {
  return (
    <Chapter title="References">
      {citations.map((cite) => (
        <pdf.View key={cite.id}>
          <LMText
            fontFamily="lmroman10-regular"
            style={{ fontSize: 10 }}
            id={'cite-forward-' + cite.id}
          >
            <LMText fontFamily="lmroman10-regular">
              <Link
                src={'#cite-back-' + cite.id}
                style={{ fontFamily: 'lmroman10-regular' }}
              >
                [{cite.index}]
              </Link>
              :{' '}
            </LMText>
            {cite.url ? <Link src={cite.url}>{cite.url}</Link> : null}
          </LMText>
        </pdf.View>
      ))}
    </Chapter>
  )
}
