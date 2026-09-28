import { defineConfig } from 'astro/config';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  site: 'https://grr-cli.pages.dev',
  base: '/',
  output: 'static',
  trailingSlash: 'always',
  build: {
    format: 'directory',
    // Inline the whole stylesheet into every document: the site ships a single
    // CSS bundle, so inlining removes the last render-blocking request and lets
    // the first paint happen on the HTML round trip alone.
    inlineStylesheets: 'always',
  },
  prefetch: {
    // Every page is same-origin and static, so hovering any link can safely
    // warm the next document before the click — this is what makes the
    // ClientRouter swaps feel instant instead of like a page refresh.
    prefetchAll: true,
    defaultStrategy: 'hover',
  },
  compressHTML: true,
  vite: {
    plugins: [tailwindcss()],
    build: {
      // Lightning CSS drops scroll-driven animation rules; esbuild keeps them.
      cssMinify: 'esbuild',
    },
  },
});
