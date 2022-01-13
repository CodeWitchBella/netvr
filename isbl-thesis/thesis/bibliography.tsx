import { Chapter, Link, Section } from './design'
import pdf from '@react-pdf/renderer'
import { LMText } from './font'
import { usePDFContext } from './base'

export type BibReference = {
  url: string
  authors?: readonly (
    | { firstname: string; surname: string }
    | { group: string }
  )[]
  title?: string
  subtitle?: string
  date?: number | string
  location?: string
  publisher?: string
  in?:
    | string
    | { journal: string; page?: string; volume?: string; number?: string }
  accessed?: string
  doi?: string
}

export function Bibliography({
  citations,
  unused,
  denomination,
}: {
  citations: readonly { id: string; index: number; data: BibReference | null }[]
  unused?: readonly (BibReference & { id: string })[]
  denomination: string
}) {
  const { production } = usePDFContext()
  return (
    <Chapter title="Bibliography" id="bibliography" denomination={denomination}>
      {citations.map((cite) => (
        <pdf.View
          key={cite.id}
          id={'cite-forward-' + cite.id}
          style={{ flexDirection: 'row' }}
        >
          <pdf.View
            style={{ width: '0.875cm', flexShrink: 0, paddingRight: '0.15cm' }}
          >
            <LMText
              fontFamily="lmroman10-regular"
              style={{ textAlign: 'right' }}
            >
              <Link
                src={'#cite-back-' + cite.id}
                style={{ fontFamily: 'lmroman10-regular' }}
              >
                [{cite.index}]
              </Link>
            </LMText>
          </pdf.View>
          {cite.data ? (
            <Citation data={cite.data} />
          ) : (
            <LMText fontFamily="lmroman10-regular">
              invalid reference {cite.id}
            </LMText>
          )}
        </pdf.View>
      ))}
      {production ? null : (
        <Section title="Unused references">
          {unused?.map((u) => (
            <pdf.View key={u.id}>
              <LMText
                fontFamily="lmroman10-regular"
                style={{ fontSize: 10, marginRight: '3mm' }}
              >
                [{u.id}]
              </LMText>
              <pdf.View
                style={{ flexDirection: 'row', paddingLeft: '0.875cm' }}
              >
                <Citation data={u} />
              </pdf.View>
            </pdf.View>
          )) ?? null}
        </Section>
      )}
    </Chapter>
  )
}

// https://security.fd.cvut.cz/wp-content/uploads/2016/02/jakpsatdp_1.pdf
function Citation({ data }: { data: BibReference }) {
  const { production } = usePDFContext()
  const titleAndSubtitle = data.title ? (
    <LMText fontFamily="lmroman10-italic">
      {data.title}
      {data.subtitle ? (
        <LMText fontFamily="lmroman10-italic">: {data.subtitle}</LMText>
      ) : null}
      .{' '}
    </LMText>
  ) : data.subtitle ? (
    <LMText fontFamily="lmroman10-italic">{data.subtitle}. </LMText>
  ) : null

  const authorsText = data.authors
    ?.map((cur, i) => {
      if ('group' in cur) return cur.group.toLocaleUpperCase()
      if (i === 0) return cur.surname.toLocaleUpperCase() + ', ' + cur.firstname
      return cur.firstname + ' ' + cur.surname.toLocaleUpperCase()
    })
    .reduce(
      (prev, cur, i, list) =>
        i === 0
          ? cur
          : i === list.length - 1
          ? prev + ', and ' + cur
          : prev + ', ' + cur,
      '',
    )
  return (
    <pdf.View style={{ flexGrow: 1 }}>
      <LMText fontFamily="lmroman10-regular">
        {authorsText ? (
          <>
            {authorsText}
            {authorsText.endsWith('.') ? '' : '.'}{' '}
          </>
        ) : null}
        {titleAndSubtitle}
        {!data.in ? null : typeof data.in === 'string' ? (
          <>
            In: <LMText fontFamily="lmroman10-italic">{data.in}</LMText>{' '}
          </>
        ) : (
          <>
            In:{' '}
            <LMText fontFamily="lmroman10-italic">
              {data.in.journal}
              {data.in.volume ? ', ' + data.in.volume : null}
            </LMText>
            {data.in.number ? `(${data.in.number})` : null}
            {data.in.page ? `, ${data.in.page}` : null}
            {', '}
          </>
        )}
        {data.url && !data.doi ? <>[online] </> : null}
        {data.location ? (
          <>
            {data.location}
            {data.publisher ? ': ' : data.date ? ', ' : '. '}
          </>
        ) : null}
        {data.publisher ? (
          <>
            {data.publisher}
            {data.date ? ', ' : '. '}
          </>
        ) : null}
        {data.date ? <>{date(data.date + '')}. </> : null}
        {data.accessed ? <>Accessed {date(data.accessed)}. </> : null}
      </LMText>
      {data.url ? (
        <LMText fontFamily="lmroman10-regular">
          <Link src={data.url}>{data.url}</Link>
        </LMText>
      ) : null}
      {data.doi ? (
        production ? (
          <LMText fontFamily="lmroman10-regular">DOI:{data.doi}</LMText>
        ) : (
          <LMText fontFamily="lmroman10-regular">
            <Link src={'https://sci-hub.se/' + data.doi}>
              {'DOI:' + data.doi}
            </Link>
          </LMText>
        )
      ) : null}
    </pdf.View>
  )
}

function date(v: string) {
  const [year, month, day] = v.split('-')
  const narrowNbsp = '\u202F'
  return `${day ?? ''} ${
    [
      '',
      'Jan',
      'Feb',
      'Mar',
      'Apr',
      'May',
      'Jun',
      'Jul',
      'Aug',
      'Sep',
      'Oct',
      'Nov',
      'Dec',
    ][parseInt(month, 10) as any] ?? ''
  } ${year}`
    .trim()
    .replace(/ /g, narrowNbsp)
}