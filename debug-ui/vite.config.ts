import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')
  return {
    plugins: [react()],
    server: {
      port: parseInt(env.VITE_PORT || '5173'),
      strictPort: true,
      proxy: {
        '/metrics': {
          target: env.VITE_API_URL || 'http://localhost:9090',
          changeOrigin: true,
        },
      },
    },
  }
})
