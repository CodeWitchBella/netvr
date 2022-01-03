import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import shimReactPdf from 'vite-plugin-shim-react-pdf'
import path from 'path'
import { fileURLToPath } from 'url'
import packageJson from './package.json'

export default defineConfig({
  plugins: [
    react({
      exclude: /thesis\/[^/]+.tsx?$/,
    }),
    shimReactPdf(),
  ],
  build: {
    target: 'esnext',
    rollupOptions: {
      external: [
        ...Object.keys(packageJson.dependencies || {}),
        ...Object.keys(packageJson.peerDependencies || {}),
        ...Object.keys(packageJson.imports || {}),
      ],
    },
    polyfillModulePreload: false,
    lib: {
      entry: path.join(__dirname, 'thesis/thesis.tsx'),

      name: 'isbl-thesis',
      formats: ['es'],
    },
  },
})
