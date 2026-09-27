import { defineConfig } from 'astro/config';

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
  compressHTML: true,
  vite: {
    build: {
      cssMinify: 'esbuild',
    },
  },
});
