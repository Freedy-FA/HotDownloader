import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'
import tauriConf from './src-tauri/tauri.conf.json' with { type: 'json' }

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    // 设置路径别名，让 import 更简洁（比如 import '@/utils'）
    alias: {
      '@': path.resolve(import.meta.dirname, './src'),
      '@components': path.resolve(import.meta.dirname, './src/components'),
    },
    // 导入时省略的扩展名（默认已支持 .js, .ts, .jsx, .tsx, .json）
    extensions: ['.mjs', '.js', '.ts', '.jsx', '.tsx', '.json', '.vue'],
  },
  // 防止 Vite 清除 Rust 显示的错误
  clearScreen: false,
  server: {
    watch: {
      // 告诉 Vite 忽略监听 `src-tauri` 目录
      ignored: ['**/src-tauri/**'],
    },
    // 热更新（HMR）配置
    hmr: {
      overlay: true, // 报错时是否在浏览器遮罩层显示
    },
  },
  // 默认只有 VITE_ 开头的变量会暴露给客户端
  // 添加有关当前构建目标的额外前缀，使这些 CLI 设置的 Tauri 环境变量可以在客户端代码中访问
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    // 在 debug 构建中不使用 minify
    minify: !process.env.TAURI_ENV_DEBUG ? 'oxc' : false,
    // 在 debug 构建中生成 sourcemap
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  define: {
    // 把版本号注入环境变量
    'import.meta.env.VITE_APP_VERSION': JSON.stringify(tauriConf.version)
  },
})
