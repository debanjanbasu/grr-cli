import type { APIRoute } from 'astro';

export const prerender = true;

const pages = [
  '',
  'install/',
  'docs/',
  'docs/getting-started/',
  'docs/commands/',
  'docs/discovery/',
  'docs/agents/',
  'docs/api-reference/',
  'docs/api-reference/analyticsadmin/',
  'docs/api-reference/analyticsdata/',
  'docs/api-reference/calendar/',
  'docs/api-reference/chat/',
  'docs/api-reference/docs/',
  'docs/api-reference/drive/',
  'docs/api-reference/forms/',
  'docs/api-reference/gmail/',
  'docs/api-reference/people/',
  'docs/api-reference/script/',
  'docs/api-reference/searchconsole/',
  'docs/api-reference/sheets/',
  'docs/api-reference/slides/',
  'docs/api-reference/tasks/',
  'changelog/',
  'compare/',
  'blog/',
  'privacy/',
  'terms/',
];

// A build-time stamp is the honest `lastmod` here: every one of these documents
// is regenerated from source on each deploy, so the build date is when the
// served HTML last changed.
const lastModified = new Date().toISOString().slice(0, 10);

export const GET: APIRoute = ({ site }) => {
  const origin = site ?? new URL('https://grr-cli.pages.dev');
  const configuredBase = import.meta.env.BASE_URL;
  const root = configuredBase.endsWith('/') ? configuredBase : `${configuredBase}/`;
  const body = pages
    .map((path) => {
      const loc = new URL(`${root}${path}`, origin).toString();
      const priority = path === '' ? '1.0' : '0.7';
      return `  <url>\n    <loc>${loc}</loc>\n    <lastmod>${lastModified}</lastmod>\n    <changefreq>weekly</changefreq>\n    <priority>${priority}</priority>\n  </url>`;
    })
    .join('\n');
  const sitemap = `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${body}\n</urlset>\n`;
  return new Response(sitemap, {
    headers: { 'Content-Type': 'application/xml; charset=utf-8' },
  });
};
