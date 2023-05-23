import { defineConfig, Plugin } from 'vite'
import react from '@vitejs/plugin-react'
import shimReactPdf from 'vite-plugin-shim-react-pdf'
import wasm from 'vite-plugin-wasm'

export default defineConfig({
  plugins: [
    wasm(),
    react({
      exclude: /thesis\/[^/]+.tsx?$/,
      babel: {
        plugins: [['@emotion', { sourceMap: false, autoLabel: 'never' }]],
      },
    }),
    shimReactPdf(),
    openerFixPlugin(),
  ],
  server: {
    fs: { allow: ['..'] },
  },
  optimizeDeps: {
    exclude: ['@isbl/thesis'],
  },
  build: {
    polyfillModulePreload: false,
    target: 'esnext',
  },
})

function openerFixPlugin(): Plugin {
  return {
    name: 'opener-fix',

    configureServer(server) {
      server.middlewares.use((req: any, res, next) => {
        console.log(req.url)
        const prefix = '/__open-in-editor?file='
        if (req.url.startsWith(prefix)) {
          let path = req.url.slice(prefix.length)
          if (path.startsWith('%')) path = decodeURIComponent(path)
          req.url = prefix + path.slice(1)
        }
        console.log(req.url)
        next()
      })
    },
  }
}
