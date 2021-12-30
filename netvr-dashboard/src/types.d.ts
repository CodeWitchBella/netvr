declare module 'remark-sectionize'
declare module 'remark-remove-comments'
declare module '*.otf'
declare module '*?raw' {
  const str: string
  export default str
}

interface ImportMeta {
  hot: any
}
