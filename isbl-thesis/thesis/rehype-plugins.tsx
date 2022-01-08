import { visit, SKIP } from 'unist-util-visit'
import type { Options as ReactMarkdownOptions } from 'react-markdown'
import unimath from './unimath-table.opm?raw'

import rehypeMathjaxSvg from 'rehype-mathjax/svg.js'

let table: Map<string, string>
let escapes: Map<string, string>
let escapesRegex = /\\([a-z]+)/
let regex: RegExp
function initTable() {
  if (!table) {
    table = new Map(
      Object.entries({
        '>=': '≥',
        '<=': '≤',
        '<-': '\\leftarrow',
        '->': '\\rightarrow',
      }),
    )
    let regStr = ''
    for (const key of table.keys()) {
      if (regStr) regStr += '|'
      regStr += key
    }
    regex = new RegExp(`(${regStr})`)

    escapes = new Map()

    for (const line of unimath.split('\n')) {
      const match =
        /^\\UnicodeMathSymbol{"0([0-9A-F]+)}{\\([a-z]+) *}{\\[a-z]+}{([^}]+)}%$/.exec(
          line,
        )
      if (!match) continue
      const sym = String.fromCodePoint(Number.parseInt(match[1], 16))
      escapes.set(match[2], sym)
    }
  }
}

export function replaceEscapes(text: string) {
  if (text.startsWith('\\')) return escapes.get(text.slice(1)) ?? text
  return text
}

function rehypeLigature() {
  initTable()
  return (tree: import('hast').Root) => {
    visit(tree, (node, index, parent) => {
      if (node.type === 'element') {
        const cn = node.properties?.className
        if (
          (Array.isArray(cn) || typeof cn === 'string') &&
          cn.includes('math')
        )
          return [SKIP]
      }

      if (node.type === 'text' && parent && index !== null) {
        const match = regex.exec(node.value)
        const match2 = escapesRegex.exec(node.value)

        if (match && match2 && match2.index < match.index) {
          return replace(node, parent, match2, index)
        }

        if (match) {
          return replace(node, parent, match, index)
        }

        if (match2) {
          return replace(node, parent, match2, index)
        }
      }
    })
  }
}

function replace(
  node: any,
  parent: any,
  match: any,
  index: number,
): ['skip', number] {
  let replacement = table.get(match[1]) ?? match[0]
  if (replacement.startsWith('\\'))
    replacement = escapes.get(replacement.slice(1))

  parent.children.splice(
    index,
    1,
    { type: 'text', value: node.value.substring(0, match.index) },
    {
      type: 'element',
      tagName: 'isbl-math',
      children: [{ type: 'text', value: replacement }],
    },
    {
      type: 'text',
      value: node.value.substring(match.index + match[0].length),
    },
  )
  return [SKIP, index + 2]
}

export const rehypePlugins: ReactMarkdownOptions['rehypePlugins'] = [
  rehypeLigature,
  [rehypeMathjaxSvg, { svg: { fontCache: 'none' } }],
  //() => (tree) => void console.log(tree),
]
