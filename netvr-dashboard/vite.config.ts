import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import shimReactPdf from 'vite-plugin-shim-react-pdf'

export default defineConfig({
  plugins: [
    react({
      exclude: /thesis\/[^/]+.tsx?$/,
    }),
    shimReactPdf(),
  ],
  esbuild: {
    jsxInject: `import React from 'react'`,
  },
  build: {
    polyfillModulePreload: false,
  },
})
