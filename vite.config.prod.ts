import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import AutoImport from "unplugin-auto-import/vite";
import Components from "unplugin-vue-components/vite";
import { ElementPlusResolver } from "unplugin-vue-components/resolvers";
import { visualizer } from "rollup-plugin-visualizer";
import viteCompression from "vite-plugin-compression";
import { fileURLToPath, URL } from "url";
import { resolve as pathResolve } from "path";

const __dirname = fileURLToPath(new URL(".", import.meta.url));

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    AutoImport({
      resolvers: [ElementPlusResolver()],
    }),
    Components({
      resolvers: [ElementPlusResolver()],
    }),
    // 生产环境压缩
    viteCompression({
      algorithm: "gzip",
      ext: ".gz",
      threshold: 10240, // 10KB以上才压缩
      deleteOriginFile: false,
    }),
    // Brotli压缩
    viteCompression({
      algorithm: "brotliCompress",
      ext: ".br",
      threshold: 10240,
      deleteOriginFile: false,
    }),
    // 打包分析
    visualizer({
      filename: "dist/stats.html",
      open: false,
      gzipSize: true,
      brotliSize: true,
    }),
  ],

  // 构建优化
  build: {
    target: "es2015",
    minify: "terser",
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true,
        pure_funcs: ["console.log", "console.info", "console.warn"],
      },
      format: {
        comments: false,
      },
    },
    rollupOptions: {
      output: {
        // 代码分割
        manualChunks: {
          // Vue相关
          vue: ["vue", "@vue/runtime-core", "@vue/shared"],
          // Element Plus
          elementPlus: ["element-plus"],
          // Tauri API
          tauri: ["@tauri-apps/api"],
          // 工具库
          utils: ["lodash-es", "dayjs"],
        },
        // 资源文件命名
        assetFileNames: "assets/[name].[hash].[ext]",
        chunkFileNames: "js/[name].[hash].js",
        entryFileNames: "js/[name].[hash].js",
      },
    },
    // 启用CSS代码分割
    cssCodeSplit: true,
    // 启用sourcemap
    sourcemap: false,
    // 报告压缩大小
    reportCompressedSize: true,
    //  chunk大小警告限制
    chunkSizeWarningLimit: 1000,
  },

  // 路径别名
  resolve: {
    alias: {
      "@": pathResolve(__dirname, "src"),
      "@components": pathResolve(__dirname, "src/components"),
      "@services": pathResolve(__dirname, "src/services"),
      "@types": pathResolve(__dirname, "src/types"),
      "@assets": pathResolve(__dirname, "src/assets"),
      "@utils": pathResolve(__dirname, "src/utils"),
    },
  },

  // 优化选项
  optimizeDeps: {
    include: [
      "vue",
      "element-plus",
      "@element-plus/icons-vue",
      "@tauri-apps/api",
    ],
    exclude: ["@tauri-apps/plugin-opener"],
  },

  // 服务器配置（仅开发环境）
  server: {
    port: 1420,
    strictPort: true,
    host: false,
    hmr: undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },

  // 预览配置
  preview: {
    port: 4173,
    strictPort: true,
    host: true,
  },

  // CSS预处理
  css: {
    preprocessorOptions: {
      scss: {
        additionalData: `@import "@/styles/variables.scss";`,
      },
    },
  },
});