import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import { resolve } from 'path'

// https://vitest.dev/config/
export default defineConfig({
  plugins: [react({ jsxRuntime: 'automatic' })],
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
  },
  // 测试配置
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    globals: true,
    env: {
      NODE_ENV: 'test',
    },
    // Windows 终端输出优化：禁用线程隔离以提高稳定性
    poolOptions: {
      threads: {
        isolate: false,
      },
    },
  },
})
