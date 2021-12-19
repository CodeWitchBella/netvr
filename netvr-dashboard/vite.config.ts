import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-refresh'
import shimReactPdf from 'vite-plugin-shim-react-pdf'

export default defineConfig({
  plugins: [react(), shimReactPdf()],
  esbuild: {
    jsxInject: `import React from 'react'`,
  },
  build: {
    polyfillModulePreload: false,
  },
})
