import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  build: {
    // Output to src/admin_assets for Rust embedding
    outDir: '../src/assets',
    emptyDirOnBuild: true,
    // Single CSS file
    cssCodeSplit: false,
    // Predictable filenames for Rust embedding
    rollupOptions: {
      output: {
        entryFileNames: 'admin.js',
        chunkFileNames: '[name].js',
        assetFileNames: (assetInfo) => {
          // Name CSS file predictably
          if (assetInfo.name?.endsWith('.css')) {
            return 'admin.css';
          }
          return '[name][extname]';
        },
      },
    },
  },
})
