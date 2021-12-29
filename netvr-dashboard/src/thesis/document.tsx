import pdf from '@react-pdf/renderer'
import { Page, usePDFContext, View, TechnikaText } from './base'
import {
  Chapter,
  TODO,
  Paragraph,
  Section,
  Strong,
  ChapterProvider,
  Em,
} from './design'
import { LMText, registerFonts } from './font'
import { TitlePage } from './title-page'
const { Document: PDFDocument } = pdf
import useReactMarkdown, { Options } from 'react-markdown'
import sectionPlugin from 'remark-sectionize'
import commentPlugin from 'remark-remove-comments'
import remarkMath from 'remark-math'
import rehypeMathjaxSvg from 'rehype-mathjax/svg.js'
import type * as HAST from 'hast'
import { chapters } from '../thesis-text/chapters'
import { visit, SKIP } from 'unist-util-visit'
import { normalize } from '../vendor/svgpathnormalizer.js'

function getText(el: HAST.ElementContent): string {
  if (el.type === 'text') return el.value
  if (el.type === 'comment') return ''
  return el.children.map(getText).join('')
}

function remarkRemoveBreaks() {
  return (tree) => {
    visit(tree, 'text', (node, index, parent) => {
      if (node.type === 'text') {
        node.value = node.value.replace(/\n/g, ' ')
      }
    })
  }
}

function rehypeLigature() {
  const table = {
    '>=': '≥',
    '<=': '≤',
  }
  const regex = new RegExp(`(${Object.keys(table).join('|')})`)
  return (tree) => {
    visit(tree, 'text', (node, index, parent) => {
      if (node.type === 'text') {
        const match = regex.exec(node.value)
        if (match) {
          parent.children.splice(
            index,
            1,
            { type: 'text', value: node.value.substring(0, match.index) },
            {
              type: 'element',
              tagName: 'em',
              properties: { math: true },
              children: [{ type: 'text', value: table[match[0]] }],
            },
            {
              type: 'text',
              value: node.value.substring(match.index + match[0].length),
            },
          )
          return [SKIP, index + 2]
        }
      }
    })
  }
}

function normalizePath(d: string) {
  const path = document.createElementNS('http://www.w3.org/2000/svg', 'path')
  path.setAttribute('d', d)
  return normalize(path.pathSegList)._pathElement.getAttribute('d')
}

function drawSvg(node: any, painter: any, matIn = [1, 0, 0, 1, 0, 0]) {
  window.normalize = normalize
  let mat = [...matIn]
  function applyMatrix(m11, m12, m21, m22, dx, dy) {
    const [m0, m1, m2, m3, m4, m5] = mat
    mat[0] = m0 * m11 + m2 * m12
    mat[1] = m1 * m11 + m3 * m12
    mat[2] = m0 * m21 + m2 * m22
    mat[3] = m1 * m21 + m3 * m22
    mat[4] = m0 * dx + m2 * dy + m4
    mat[5] = m1 * dx + m3 * dy + m5
  }

  if (node.tagName === 'g') {
    for (const cssTransform of node.properties.transform?.split(' ') ?? []) {
      const args = cssTransform
        .split(',')
        .map((v) => Number.parseFloat(v.replace(/[^-0-9.]/g, '')))
      if (cssTransform.startsWith('translate')) {
        applyMatrix(1, 0, 0, 1, args[0], args[1])
      } else if (cssTransform.startsWith('scale')) {
        applyMatrix(args[0], 0, 0, args[1] ?? args[0], 0, 0)
      } else {
        console.log(cssTransform)
      }
    }
  } else {
    painter.save()
    painter.translate(mat[4], mat[5]).scale(mat[0], mat[3])

    if (node.tagName === 'path') {
      painter.path(normalizePath(node.properties.d))
    } else if (node.tagName === 'rect') {
      painter.rect(
        ...['x', 'y', 'width', 'height'].map((v) =>
          parseFloat(node.properties[v]),
        ),
      )
    } else {
      //console.log(node)
    }
    painter.lineWidth(1).fillAndStroke('black', 'black').restore()
  }
  const children = node.children.map((n) => drawSvg(n, painter, mat)).join('\n')
  const properties = Object.entries(node.properties)
    .map(([k, v]) => `${k}=${JSON.stringify(v)}`)
    .join(' ')

  if (!children) return `<${node.tagName} ${properties}/>`

  return `<${node.tagName} ${properties}>\n${children}\n</${node.tagName}>`
}

const components: Options['components'] = {
  h1: (props) => <Text>{props.children}</Text>,
  h2: (props) => <Text>{props.children}</Text>,
  p: (props) => {
    if (
      props.node.children.length === 1 &&
      props.node.children[0].tagName === 'mjx-container'
    ) {
      return <>{props.children}</>
    }
    return <Paragraph>{props.children}</Paragraph>
  },
  strong: (props) => <Strong>{props.children}</Strong>,
  em: (props) => {
    if (props.math)
      return <LMText fontFamily="latinmodern-math">{props.children}</LMText>
    return <Em>{props.children}</Em>
  },

  // MathJax
  svg: ({ children, width, height, ...props }) => {
    const size = {
      width: parseFloat(width.replace(/[^0-9.]/g, '')) * 11,
      height: parseFloat(height.replace(/[^0-9.]/g, '')) * 11,
    }
    const style = {
      width: size.width + 'pt',
      height: size.height + 'pt',
    }
    return (
      <pdf.View style={{ position: 'relative', alignItems: 'center' }}>
        {/*<pdf.Canvas
          style={style}
          paint={(painter, w, h) => {
            //painter.rect(0, 0, w, h).fill('gray')

            const viewBox = props.viewBox.split(' ').map(parseFloat)

            painter.scale(w / viewBox[2], h / viewBox[3])
            painter.translate(-viewBox[0], -viewBox[1])

            drawSvg(props.node, painter)
          }}
        />*/}
        <pdf.Svg {...props} style={style}>
          {children}
        </pdf.Svg>
      </pdf.View>
    )
  },
  g: pdf.G,
  defs: pdf.Defs,
  path: ({ d, ...props }) => <pdf.Path d={normalizePath(d)} {...props} />,
  rect: pdf.Rect,
  //span: (props) => console.log(props) || <>{props.children}</>,
  'mjx-container': (props) => <>{props.children}</>,

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
    return <View>{props.children}</View>
  },
}

const allowElement: Options['allowElement'] = (element, index, parent) => {
  const allow = element.tagName in components
  if (!allow) console.warn('Disallowed element:', element)
  return allow
}

function MarkdownChapter({
  children,
  index,
}: {
  children: string
  index: number
}) {
  const result = useReactMarkdown({
    components: components,
    includeElementIndex: true,
    allowElement,
    unwrapDisallowed: true,
    children,
    remarkPlugins: [
      remarkRemoveBreaks,
      sectionPlugin,
      commentPlugin,
      remarkMath,
    ],
    rehypePlugins: [
      rehypeLigature,
      [rehypeMathjaxSvg, { svg: { fontCache: 'none' } }],
      //() => (tree) => void console.log(tree),
    ],
  })
  return (
    <ChapterProvider index={index}>
      {(result.props as any).children.filter((v: any) => typeof v !== 'string')}
    </ChapterProvider>
  )
}

export function Document() {
  registerFonts()
  const { lang } = usePDFContext()
  return (
    <PDFDocument>
      <TitlePage
        title={
          lang === 'en'
            ? 'Tracking multiple VR users in a shared physical space'
            : 'Sledování více uživatelů VR světa ve sdíleném fyzickém prostoru'
        }
      />
      <Page style={{ alignItems: 'center', justifyContent: 'flex-end' }}>
        <LMText fontFamily="lmroman10-regular">
          Page intentionally left blank
        </LMText>
      </Page>

      <Page>
        {chapters.map(([key, text], index) => (
          <MarkdownChapter index={index + 1} key={key}>
            {text}
          </MarkdownChapter>
        ))}
      </Page>
    </PDFDocument>
  )
}
