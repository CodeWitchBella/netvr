import pdf from '@react-pdf/renderer'
import { Chapter, Paragraph, Section, Strong, Em, TODO, Link } from './design'
import { LMText } from './font'
import sectionPlugin from 'remark-sectionize'
import commentPlugin from 'remark-remove-comments'
import remarkMath from 'remark-math'
import rehypeMathjaxSvg from 'rehype-mathjax/svg.js'
import type * as HAST from 'hast'
import { visit, SKIP } from 'unist-util-visit'
import remarkDirective from 'remark-directive'
import remarkUnwrapImages from 'remark-unwrap-images'
import remarkGfm from 'remark-gfm'
import { rehypeLigature } from './ligatures'
import { remarkTruncateLinks } from 'remark-truncate-links'
// @ts-ignore
import { normalize } from '../vendor/svgpathnormalizer.js'
import { Fragment } from 'react'
import { VFile } from 'vfile'
import { unified } from 'unified'
import remarkParse from 'remark-parse'
import remarkRehype from 'remark-rehype'
import type { Options as ReactMarkdownOptions } from 'react-markdown'
import { html } from 'property-information'
import rehypeFilter from 'react-markdown/lib/rehype-filter.js'
import { uriTransformer } from 'react-markdown'
import { childrenToReact } from 'react-markdown/lib/ast-to-react.js'

function getText(el: HAST.ElementContent): string {
  if (el.type === 'text') return el.value
  if (el.type === 'comment') return ''
  return ('children' in el ? el.children : []).map(getText).join('')
}

function remarkRemoveBreaks() {
  return (tree: import('mdast').Root) => {
    visit(tree, 'text', (node, index, parent) => {
      if (node.type === 'text') {
        node.value = node.value.replace(/\n/g, ' ')
      }
    })
  }
}

function reduce<V, Res>(
  iterable: Iterable<V>,
  def: Res,
  reducer: (prev: Res, cur: V) => Res,
): Res {
  let value = def
  for (const v of iterable) value = reducer(value, v)
  return value
}

function remarkCiteCounter() {
  return (tree: import('mdast').Root, file: VFile) => {
    let map: Map<string, number> = file.data.citeMap as any
    if (!map) {
      map = new Map<string, number>()
      file.data.citeMap = map
    }
    let counter = reduce(map.values(), 0, (a, b) => Math.max(a, b))

    visit(tree, 'textDirective', (node, index, parent) => {
      if (
        node.name === 'cite' &&
        node.children.length > 0 &&
        node.children[0].type === 'text'
      ) {
        const ref = node.children[0].value
        let index = map.get(ref)
        let firstOccurence = false
        if (!index) {
          firstOccurence = true
          index = ++counter
          map.set(ref, index)
        }

        const data = node.data || (node.data = {})

        data.hName = 'isbl-cite'
        data.hProperties = {
          ...node.attributes,
          index,
          target: ref,
          firstOccurence,
        }
      }
    })
  }
}

function remarkGenericDirective() {
  return (tree: import('mdast').Root) => {
    visit(tree, (node) => {
      const isDirective =
        node.type === 'textDirective' ||
        node.type === 'leafDirective' ||
        node.type === 'containerDirective'
      if (isDirective && !node.data) {
        node.data = {
          hName: 'isbl-directive',
          hProperties: {
            ...node.attributes,
            directive: node.name.toLowerCase(),
          },
        }
      }
    })
  }
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
    if (props.math)
      return <LMText fontFamily="latinmodern-math">{props.children}</LMText>
    return <Em>{props.children}</Em>
  },
  a: (props) =>
    props.href ? (
      <Link src={props.href}>{props.children}</Link>
    ) : (
      <>{props.children}</>
    ),
  img: (props) => <pdf.Image src={props.src} style={{ maxWidth: '100%' }} />,
  code: (props) => <pdf.Text>{props.children}</pdf.Text>,

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
  rect: pdf.Rect as any,
  ...({ 'mjx-container': (props: any) => <>{props.children}</> } as any),
  style: () => null,

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
      console.warn('Unknown directive', props)
      return null
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
          <Section title={getText(firstChild)} no={(props.index ?? -1) + 1}>
            {props.children.slice(1)}
          </Section>
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
  remarkPlugins: [
    remarkDirective,
    remarkCiteCounter,
    remarkRemoveBreaks,
    sectionPlugin,
    commentPlugin,
    remarkMath,
    remarkGenericDirective,
    remarkUnwrapImages,
    remarkGfm,
    [remarkTruncateLinks, { style: 'smart', length: 40 }],
  ],
  rehypePlugins: [
    rehypeLigature,
    [rehypeMathjaxSvg, { svg: { fontCache: 'none' } }],
    //() => (tree) => void console.log(tree),
  ],
  transformLinkUri: uriTransformer,
}

export function markdownListToAst(markdowns: string[]) {
  const processor = unified()
    .use(remarkParse)
    .use(options.remarkPlugins || [])
    .use(remarkRehype, { allowDangerousHtml: true })
    .use(options.rehypePlugins || [])
    .use(rehypeFilter, options)

  const file = new VFile()
  const asts = markdowns.map((markdown) => {
    file.value = markdown

    const hastNode = processor.runSync(processor.parse(file), file)

    if (hastNode.type !== 'root') {
      throw new TypeError('Expected a `root` node')
    }

    return hastNode
  })
  return {
    asts,
    citeMap: Object.fromEntries(
      (file.data.citeMap as Map<string, number>).entries(),
    ),
  }
}

export function ReactMarkdown({ hast }: { hast: any }) {
  return (
    <>
      {childrenToReact({ options, schema: html, listDepth: 0 }, hast).filter(
        (v: any) => typeof v !== 'string',
      )}
    </>
  )
}
