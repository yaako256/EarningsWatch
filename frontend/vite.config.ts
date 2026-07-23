import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
 plugins: [react()],
  server: {
    host: '0.0.0.0',
    proxy: {
      // SameSite=Strict Cookie を壊さないためにプロキシ経由で /api を転送
      '/api': {
        target: 'http://backend:9010',
        changeOrigin: true,
      },
    },
  },
})
