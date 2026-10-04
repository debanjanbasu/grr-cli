import { defineConfig } from 'astro/config';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  site: 'https://grr-cli.pages.dev',
  base: '/',
  trailingSlash: 'always',
  build: {
    format: 'directory',
    // Emit the stylesheet as a separate cached file rather than inlining it:
    // the Tailwind + DaisyUI bundle is ~140KB, so inlining it would make every
    // document carry it and delay first paint until the whole HTML lands. As
    // a <link> it downloads in parallel with the HTML, is fetched once, and
    // the ClientRouter reuses it across every client-side navigation.
    inlineStylesheets: 'auto',
  },
  prefetch: {
    // Every page is same-origin and static, so hovering any link can safely
    // warm the next document before the click — this is what makes the
    // ClientRouter swaps feel instant instead of like a page refresh.
    prefetchAll: true,
    defaultStrategy: 'hover',
  },
  // Astro 7 defaults compressHTML to 'jsx', whose React-style whitespace
  // rules strip line breaks around elements. That would join the words either
  // side of the <span>/<code> segments InlineCode renders from a single copy
  // string, so the lossless boolean mode is deliberate here.
  compressHTML: true,
  vite: {
    plugins: [tailwindcss()],
    build: {
      // Lightning CSS drops scroll-driven animation rules; esbuild keeps them.
      cssMinify: 'esbuild',
    },
  },
});
