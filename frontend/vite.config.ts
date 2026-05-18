import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { resolve } from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  base: './',
  plugins: [react()],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    outDir: 'dist',
    sourcemap: process.env.VITE_SOURCEMAP === 'true',
    rollupOptions: {
      output: {
        manualChunks: {
          'vendor-react': ['react', 'react-dom'],
          'vendor-state': ['zustand'],
          'vendor-ui': ['motion', 'clsx', 'lucide-react'],
          'vendor-i18n': ['i18next', 'react-i18next'],
          'vendor-virtual': ['@tanstack/react-virtual'],
          'vendor-dropzone': ['react-dropzone'],
        },
      },
    },
  },
})
