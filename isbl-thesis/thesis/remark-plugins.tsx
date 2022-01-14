import sectionPlugin from 'remark-sectionize'
import commentPlugin from 'remark-remove-comments'
import remarkMath from 'remark-math'
import { visit, SKIP } from 'unist-util-visit'
import remarkDirective from 'remark-directive'
import remarkUnwrapImages from 'remark-unwrap-images'
import remarkGfm from 'remark-gfm'
import remarkHeadingId from 'remark-heading-id'
import { remarkTruncateLinks } from 'remark-truncate-links'
import { VFile } from 'vfile'

function remarkRemoveBreaks() {
  return (tree: import('mdast').Root) => {
    visit(tree, 'text', (node, index, parent) => {
      if (node.type === 'text') {
        node.value = node.value.replace(/\n/g, ' ')
      }
    })
  }
}

function remarkImageNumbering() {
  return (tree: import('mdast').Root) => {
    let counter = 0
    const map = new Map<string, number>()
    visit(tree, 'image', (node, index, parent) => {
      if (node.type === 'image') {
        const data = node.data || (node.data = {})
        data.hProperties = {
          imageIndex: ++counter,
        }
        if (node.title) map.set(node.title, counter)
      }
    })

    visit(tree, 'textDirective', (node) => {
      if (
        node.name === 'ref' &&
        node.children.length > 0 &&
        node.children[0].type === 'text'
      ) {
        const attributes: any = node.attributes || (node.attributes = {})

        const ref = attributes.title || node.children[0].value
        const number = map.get(ref)
        if (!attributes.title) {
          attributes.title = ref
          node.children = []
        }
        if (number) {
          attributes.number = number
        }
      }
    })
  }
}

function remarkSectionNumbering() {
  return (tree: import('mdast').Root, file: VFile) => {
    let counter: number[] = []
    visit(tree, 'section', (node, index, parent) => {
      const depth = node.depth
      if (node.type === 'section' && typeof depth === 'number') {
        while (counter.length < depth)
          counter.push(depth === 1 ? (file.data as any)?.chapterIndex ?? 0 : 0)
        while (counter.length > depth) counter.pop()
        counter[depth - 1]++

        const number = counter[depth - 1]

        const data = node.data || (node.data = {})
        data.textNumber = counter.join('.')
        data.hProperties = { number }
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

function remarkCollectIds() {
  return (tree: import('mdast').Root, file: VFile) => {
    let map: Map<string, string> = file.data.refIds as any
    visit(tree, 'heading', (node, index, parent) => {
      const id: string = node?.data?.id
      if (id) map.set(id, (parent.data as any)?.textNumber)
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

export const remarkPlugins = [
  remarkDirective,
  remarkCiteCounter,
  remarkRemoveBreaks,
  sectionPlugin,
  remarkSectionNumbering,
  commentPlugin,
  remarkMath,
  remarkImageNumbering,
  remarkGenericDirective,
  remarkUnwrapImages,
  remarkGfm,
  [remarkTruncateLinks, { style: 'smart', length: 40 }],
  remarkHeadingId,
  remarkCollectIds,
]
