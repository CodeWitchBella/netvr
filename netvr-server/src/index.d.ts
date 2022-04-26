declare module '__STATIC_CONTENT_MANIFEST' {
  const v: string
  export default v
}

interface ImportMeta {
  url: string
}

declare const Deno: {
  readTextFile(path: string | URL): Promise<string>
  writeTextFile(path: string | URL, data: string): Promise<void>
  run: any
  build: {
    target: string
    arch: 'x86_64' | 'aarch64'
    os: 'darwin' | 'linux' | 'windows'
    vendor: string
    env?: string
  }
  version: { deno: string; v8: string; typescript: string }
  permissions: any
  listen: any
  serveHttp: any
  upgradeWebSocket: any
  readFile: any
  args: string[]
  networkInterfaces(): any[]
  errors: { InvalidData: any }
}
declare namespace Deno {
  export type Conn = any
  export type RequestEvent = any
}
