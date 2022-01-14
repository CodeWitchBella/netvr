import pdf from '@react-pdf/renderer'
import {
  Chapter,
  Paragraph,
  Section,
  Strong,
  Em,
  TODO,
  Link,
  Reference,
  Image,
  SubSection,
} from './design'
import { LMText } from './font'
import { remarkPlugins } from './remark-plugins'
import type * as HAST from 'hast'
// @ts-ignore
import { normalize } from './vendor/svgpathnormalizer.js'
import { VFile } from 'vfile'
import { unified } from 'unified'
import remarkParse from 'remark-parse'
import remarkRehype from 'remark-rehype'
import type { Options as ReactMarkdownOptions } from 'react-markdown'
import { html } from 'property-information'
import rehypeFilter from 'react-markdown/lib/rehype-filter.js'
import { uriTransformer } from 'react-markdown'
import { childrenToReact } from 'react-markdown/lib/ast-to-react.js'
import { FootnoteRef } from './footnotes'
import { usePDFContext, View } from './base'
import { rehypePlugins } from './rehype-plugins'

function getText(el: HAST.ElementContent): string {
  if (el.type === 'text') return el.value
  if (el.type === 'comment') return ''
  return ('children' in el ? el.children : []).map(getText).join('')
}

function normalizePath(d: string) {
  const path = document.createElementNS('http://www.w3.org/2000/svg', 'path')
  path.setAttribute('d', d)
  return normalize((path as any).pathSegList).getAttribute('d')
}

function serializeSvg(node: HAST.Element | HAST.ElementContent): string {
  const children = ('children' in node ? node.children : [])
    .map((n) => serializeSvg(n))
    .join('\n')
  let properties = Object.entries(
    'properties' in node ? node.properties ?? {} : {},
  )
    .filter(([k]) => !['dataMmlNode', 'dataC'].includes(k))
    .map(([k, v]) => `${k}=${JSON.stringify(v)}`)
    .join(' ')
  if (properties) properties = ' ' + properties
  if (!children && !properties) return ''
  if (!('tagName' in node)) return ''
  if (!children) return `<${node.tagName}${properties}/>`
  if (!properties && node.tagName === 'g') return children

  return `<${node.tagName}${properties}>\n${children}\n</${node.tagName}>`
}

type NotUndefined<T> = T extends undefined ? never : T
const components: NotUndefined<ReactMarkdownOptions['components']> = {
  h1: (props) => <pdf.Text>{props.children}</pdf.Text>,
  h2: (props) => <pdf.Text>{props.children}</pdf.Text>,
  h3: (props) => <pdf.Text>{props.children}</pdf.Text>,
  p: (props) => {
    const citeBack: string[] = []
    function traverse(node: any) {
      if (node.properties?.target) citeBack.push(node.properties.target)
      if ('children' in node) node.children.forEach(traverse)
    }
    traverse(props.node)

    if (
      props.node.children.length === 1 &&
      (props.node.children[0] as any).tagName === 'mjx-container'
    ) {
      return <>{props.children}</>
    }
    return (
      <>
        {citeBack.map((cite) => (
          <pdf.Text key={cite} id={'cite-back-' + cite}></pdf.Text>
        ))}
        <Paragraph first={props.index === 1}>{props.children}</Paragraph>
      </>
    )
  },
  strong: (props) => <Strong>{props.children}</Strong>,
  em: (props) => {
    return <Em>{props.children}</Em>
  },
  a: (props) =>
    props.href ? (
      <Link src={props.href}>{props.children}</Link>
    ) : (
      <>{props.children}</>
    ),
  img: (props) => {
    const ctx = usePDFContext()
    return (
      <Image
        title={props.title ?? ''}
        src={ctx.files[props.src ?? ''] ?? props.src ?? ''}
        description={props.alt ?? ''}
        index={(props as any).imageIndex}
      />
    )
  },
  code: (props) => <pdf.Text>{props.children}</pdf.Text>,
  pre: (props) => (
    <LMText fontFamily="lmmono10-regular" style={{ fontSize: 11 }}>
      {props.children}
    </LMText>
  ),

  // MathJax
  svg: ({ children, width, height, ...props }) => {
    const size = {
      width: parseFloat((width + '').replace(/[^0-9.]/g, '')) * 11,
      height: parseFloat((height + '').replace(/[^0-9.]/g, '')) * 11,
    }
    const style = {
      width: size.width + 'pt',
      height: size.height + 'pt',
    }
    if (false) {
      return (
        <pdf.Text
          style={{
            letterSpacing: style.width,
            lineHeight: size.height / 11,
            textDecorationStyle: 'solid',
            textDecorationColor: 'black',
            textDecoration: 'underline',
            position: 'relative',
            backgroundColor: 'green',
          }}
        >
          <pdf.View
            style={{ position: 'absolute', backgroundColor: 'green', ...style }}
            debug
          ></pdf.View>
          {'\u200B\u200B' /* two zero-width spaces */}
        </pdf.Text>
      )
    }
    //console.log()

    return (
      <pdf.View style={{ alignSelf: 'center', ...style }}>
        <pdf.Svg {...(props as any)} style={style}>
          {children}
        </pdf.Svg>
      </pdf.View>
    )
  },
  g: pdf.G as any,
  defs: pdf.Defs as any,
  path: ({ d, ...props }) => (
    <pdf.Path d={d ? normalizePath(d) : d} {...(props as any)} />
  ),
  text: pdf.Text as any,
  rect: pdf.Rect as any,
  ...({ 'mjx-container': (props: any) => <>{props.children}</> } as any),
  style: () => null,
  ul: (props) => (
    <View>{props.children.filter((v) => typeof v !== 'string')}</View>
  ),
  ol: (props) => (
    <View>{props.children.filter((v) => typeof v !== 'string')}</View>
  ),
  li: (props) => (
    <View>
      <LMText fontFamily="lmroman10-regular" style={{ fontSize: 11 }}>
        - {props.children}
      </LMText>
    </View>
  ),
  title: (props) => null,

  ...{
    // cite
    'isbl-cite': (props: any) => (
      <pdf.Text>
        <pdf.Link
          style={{ textDecoration: 'none', color: 'black' }}
          src={'#cite-forward-' + props.node.properties.target}
        >
          [{props.node.properties.index}]
        </pdf.Link>
      </pdf.Text>
    ),

    'isbl-directive': (props: any) => {
      if (props.directive === 'todo') return <TODO>{props.children}</TODO>
      if (props.directive === 'pagebreak') return <pdf.View break={true} />
      if (props.directive === 'nowrap')
        return <pdf.View wrap={false}>{props.children}</pdf.View>
      if (props.directive === 'ref') return <Reference {...props} />
      if (props.directive === 'space')
        return <pdf.View style={{ height: 11 }} />
      if (props.directive === 'nbsp') return '\u00A0'
      if (props.directive === 'footnote') {
        return (
          <FootnoteRef
            sign={props.sign}
            content={
              props.node?.children
                ?.map((child: any) =>
                  child?.type === 'text' ? child.value : '',
                )
                .join('') ?? ''
            }
          />
        )
      }
      console.warn('Unknown directive', props)
      return null
    },

    'isbl-math': (props: any) => {
      return (
        <LMText fontFamily="latinmodern-math" automaticFontSize={false}>
          {props.children}
        </LMText>
      )
    },
  },

  // Numbered sections
  section: (props) => {
    const firstChild = props.node.children[0]
    if (firstChild?.type === 'element' && /^h[1-6]$/.test(firstChild.tagName)) {
      // is heading
      if (firstChild.tagName === 'h1') {
        return (
          <Chapter title={getText(firstChild)}>
            {props.children.slice(1)}
          </Chapter>
        )
      }
      if (firstChild.tagName === 'h2') {
        return (
          <Section
            title={getText(firstChild)}
            no={(props as any).number}
            id={firstChild.properties?.id ?? undefined}
          >
            {props.children.slice(1)}
          </Section>
        )
      }
      if (firstChild.tagName === 'h3') {
        return (
          <SubSection title={getText(firstChild)} no={(props as any).number}>
            {props.children.slice(1)}
          </SubSection>
        )
      }
    }
    return <pdf.View>{props.children}</pdf.View>
  },
}

const allowElement: ReactMarkdownOptions['allowElement'] = (
  element,
  index,
  parent,
) => {
  const allow = element.tagName in components
  if (
    element.tagName === 'span' &&
    (element.properties?.className as any)?.includes('math')
  ) {
    return false
  }
  if (!allow) console.warn('Disallowed element:', element)
  return allow
}

const options: Omit<ReactMarkdownOptions, 'children'> = {
  components,
  includeElementIndex: true,
  allowElement,
  unwrapDisallowed: true,
  remarkPlugins,
  rehypePlugins,
  transformLinkUri: uriTransformer,
}

export function markdownListToAst<T extends { text: string | null }>(
  markdowns: readonly T[],
) {
  const processor = unified()
    .use(remarkParse)
    .use(options.remarkPlugins || [])
    .use(remarkRehype, { allowDangerousHtml: true })
    .use(options.rehypePlugins || [])
    .use(rehypeFilter, options)

  const file = new VFile()
  file.data.citeMap = new Map<string, number>()
  file.data.refIds = new Map()
  const asts = markdowns.map(
    (
      entry: T & { ast?: any },
      chapterIndex,
    ): { ast: HAST.Root | null } & Omit<T, 'text' | 'ast'> => {
      const { text, ast: astIn, ...meta } = entry
      if (!text) return { ...meta, ast: null }

      file.value = text
      file.data.chapterIndex = chapterIndex

      const hastNode = processor.runSync(processor.parse(file), file)

      if (hastNode.type !== 'root') {
        throw new TypeError('Expected a `root` node')
      }

      return { ...meta, ast: hastNode }
    },
  )
  return {
    asts,
    citeMap: Object.fromEntries(
      (file?.data?.citeMap as Map<string, number>).entries(),
    ),
    refIds: Object.fromEntries(
      (file?.data?.refIds as Map<string, string>).entries(),
    ),
  }
}

export function ReactMarkdown({ hast }: { hast: any }) {
  const children = childrenToReact(
    { options, schema: html, listDepth: 0 },
    hast,
  ).filter((v: any) => typeof v !== 'string')
  return <>{children}</>
}
