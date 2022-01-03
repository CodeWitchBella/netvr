import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import shimReactPdf from 'vite-plugin-shim-react-pdf'
import path from 'path'
import packageJson from './package.json'
const external = [
  ...Object.keys(packageJson.dependencies || {}),
  ...Object.keys(packageJson.peerDependencies || {}),
  '../assets.js',
]

export default defineConfig({
  plugins: [react({}), shimReactPdf()],
  build: {
    target: 'esnext',
    rollupOptions: {
      external: (im) =>
        im !== 'react/jsx-runtime' &&
        external.some((dep) => im.startsWith(dep)),
    },
    polyfillModulePreload: false,
    lib: {
      entry: path.join(__dirname, 'thesis/thesis.tsx'),

      name: 'isbl-thesis',
      formats: ['es'],
    },
  },
})
