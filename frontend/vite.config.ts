import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { resolve } from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  // Tauri 2.x 开发服务器配置
  server: {
    port: 1420,
    strictPort: true,
  },
  // 构建配置
  build: {
    outDir: 'dist',
    sourcemap: true,
    minify: 'esbuild',
    esbuild: {
      drop: ['console'],
    },
    rollupOptions: {
      output: {
        manualChunks: {
          'vendor-react': ['react', 'react-dom'],
          'vendor-state': ['zustand'],
          'vendor-ui': ['framer-motion', 'clsx', 'lucide-react'],
          'vendor-i18n': ['i18next', 'react-i18next'],
          'vendor-virtual': ['@tanstack/react-virtual'],
          'vendor-dropzone': ['react-dropzone'],
        },
      },
    },
  },
})
