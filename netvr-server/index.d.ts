declare module '__STATIC_CONTENT_MANIFEST' {
  const v: string
  export default v
}

declare const Deno: any
declare namespace Deno {
  export type Conn = any
  export type RequestEvent = any
}
