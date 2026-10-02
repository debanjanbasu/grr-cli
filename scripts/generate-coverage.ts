#!/usr/bin/env node
// Generate the site's coverage table (site/src/data/discovery-coverage.ts) from
// the committed Discovery index (src/discovery/*.json + manifest.json).
//
// Why generate: the site prints per-service method counts, scope counts and
// Discovery revisions. Transcribing those by hand meant they were wrong the
// moment the daily discovery refresh landed — which is exactly how the table
// ended up advertising revisions a month old. Deriving them makes the class of
// bug impossible rather than merely unlikely.
//
// What is derived (never hand-written): name, version, method count, scope
// count, revision, and every total the site renders. What is authored here:
// the display order and the two prose fields (label, note) — the parts a human
// owns. The assert() below is the contract: every service in the index must
// have prose, and every service in this table must exist in the index, so a
// new or removed service fails the generator instead of silently vanishing
// from the site.
//
// Determinism: no wall-clock timestamps (the marker reuses the manifest's
// own), and the order is the explicit table below — never a filesystem walk.
//
// Usage:
//   node scripts/generate-coverage.ts         # write site/src/data/discovery-coverage.ts
//   node scripts/generate-coverage.ts --check # exit 1 if stale (CI gate)

import { readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const INDEX_DIR = resolve(ROOT, 'src', 'discovery');
const MANIFEST = resolve(INDEX_DIR, 'manifest.json');
const OUT_FILE = resolve(ROOT, 'site', 'src', 'data', 'discovery-coverage.ts');

const CHECK_ONLY = process.argv.includes('--check');

interface ManifestService {
  methods: number;
  scopes: number;
  revision: string;
}

interface Manifest {
  generatedAt: string;
  services: Record<string, ManifestService>;
}

/** The distilled per-service document — only the header fields are read. */
interface ServiceDocument {
  name: string;
  version: string;
}

interface Prose {
  /** Human label used in headings and prose. */
  label: string;
  /** One-line note on what the service covers. */
  note: string;
}

// The display order is the order the site's cards and table read in: the ten
// core Workspace services, then the Apps Script and Analytics family, then
// Search Console. It is authored, not sorted, because it is an editorial
// choice — but it is still a closed set, checked against the index below.
const PROSE: ReadonlyArray<readonly [service: string, Prose]> = [
  ['gmail', {
    label: 'Gmail',
    note: 'Messages, threads, drafts, labels, history, filters, forwarding, send-as, and settings.',
  }],
  ['calendar', {
    label: 'Calendar',
    note: 'Calendars, events, instances, ACL, free/busy, colors, and settings.',
  }],
  ['drive', {
    label: 'Drive',
    note: 'Files, permissions, comments, revisions, changes, drives, and team drives.',
  }],
  ['people', {
    label: 'People (Contacts)',
    note: 'Contacts, connections, other contacts, contact groups, and directory people.',
  }],
  ['chat', {
    label: 'Chat',
    note: 'Spaces, messages, memberships, reactions, attachments, sections, and availability.',
  }],
  ['forms', {
    label: 'Forms',
    note: 'Form bodies, responses, publish settings, and push-notification watches.',
  }],
  ['sheets', {
    label: 'Sheets',
    note: 'Spreadsheet values, batch updates, data filters, and developer metadata.',
  }],
  ['tasks', {
    label: 'Tasks',
    note: 'Task lists and the tasks inside them. See the scope caveat below.',
  }],
  ['slides', {
    label: 'Slides',
    note: 'Presentation batch updates, page reads, and thumbnails.',
  }],
  ['docs', {
    label: 'Docs',
    note: 'Document create, read, and batch update.',
  }],
  ['script', {
    label: 'Apps Script',
    note: 'Script projects, deployments, versions, processes, and running functions.',
  }],
  ['analyticsadmin', {
    label: 'Analytics Admin',
    note: 'Accounts, properties, data streams, custom dimensions and metrics, conversion events, and the Ads and Firebase links.',
  }],
  ['analyticsdata', {
    label: 'Analytics Data',
    note: 'Run reports — standard, realtime, pivot, and batch — plus compatibility checks, metadata, and audience exports.',
  }],
  ['searchconsole', {
    label: 'Search Console',
    note: 'Search analytics queries, sitemaps, sites, and URL inspection.',
  }],
];

function assert(condition: unknown, message: string): void {
  if (!condition) {
    console.error(`generate-coverage: ${message}`);
    process.exit(1);
  }
}

/** Single-quoted TS string literal — the file is hand-diffable, so match its style. */
const quote = (value: string): string => `'${value.replace(/\\/g, '\\\\').replace(/'/g, "\\'")}'`;

const manifest = JSON.parse(await readFile(MANIFEST, 'utf8')) as Manifest;
const indexServices = Object.keys(manifest.services).sort();

const declared = PROSE.map(([service]) => service);
const missingProse = indexServices.filter((service) => !declared.includes(service));
const missingDocument = declared.filter((service) => !indexServices.includes(service));

// The two failure modes this script exists to prevent, both loud:
//   - a service Google's index gained has no label/note to render;
//   - a service the index dropped is still listed here and would render a row
//     full of zeroes against a file that no longer exists.
assert(
  missingProse.length === 0,
  `no prose for service(s) in the index: ${missingProse.join(', ')}. Add a [name, { label, note }] row to PROSE in scripts/generate-coverage.ts.`,
);
assert(
  missingDocument.length === 0,
  `prose for service(s) no longer in the index: ${missingDocument.join(', ')}. Remove the row from PROSE in scripts/generate-coverage.ts.`,
);
// Static name -> prose lookup, and the duplicate guard that goes with it.
const byName: Record<string, Prose> = Object.fromEntries(PROSE);
assert(
  Object.keys(byName).length === PROSE.length,
  'PROSE lists a service twice.',
);


const rows: string[] = [];
let methodTotal = 0;
let scopeTotal = 0;

for (const [service] of PROSE) {
  const prose = byName[service];
  const summary = manifest.services[service];
  // The version string is not in the manifest (it is a document header, not a
  // count), so it is read from the distilled service document itself.
  const document = JSON.parse(await readFile(resolve(INDEX_DIR, `${service}.json`), 'utf8')) as ServiceDocument;
  assert(
    document.name === service,
    `${service}.json declares name "${document.name}", not "${service}".`,
  );

  methodTotal += summary.methods;
  scopeTotal += summary.scopes;

  rows.push(
    '  {',
    `    name: ${quote(service)},`,
    `    label: ${quote(prose.label)},`,
    `    version: ${quote(document.version)},`,
    `    methods: ${summary.methods},`,
    `    scopes: ${summary.scopes},`,
    `    revision: ${quote(summary.revision)},`,
    `    note: ${quote(prose.note)},`,
    '  },',
  );
}

const serviceTotal = PROSE.length;

const body = `/**
 * Coverage figures for the generated command surface.
 *
 * GENERATED by scripts/generate-coverage.ts from the committed Discovery index
 * (\`src/discovery/manifest.json\` plus each \`src/discovery/<service>.json\`),
 * which is what \`grr api list\` reads at runtime. Do not edit by hand —
 * regenerate with \`npm run coverage\`; CI gates the result with
 * \`npm run check:coverage\`.
 *
 * Every method count, scope count and revision below is read from that index,
 * so a row can never advertise a revision the index has already moved past. The
 * label, note and row order are the authored parts and live in PROSE inside the
 * generator.
 *
 * Every service has dedicated \`grr <service>\` commands — the tree is
 * generated from the index, so all ${serviceTotal} namespaces are on equal
 * footing and \`grr api\` is the flat, id-based way into the same methods.
 */
export interface DiscoveryService {
  /** Key accepted by \`grr api list --service\` and the \`grr api call\` id prefix. */
  name: string;
  /** Human label used in prose. */
  label: string;
  /** Google API version this index was distilled from. */
  version: string;
  methods: number;
  /** Distinct scopes declared across the service's methods. */
  scopes: number;
  /** \`revision\` field of the Discovery document at refresh time. */
  revision: string;
  /** One-line note on what the service covers. */
  note: string;
}

export const discoveryServices: DiscoveryService[] = [
${rows.join('\n')}
];

/** Sum of \`methods\` across every service. Matches \`grr api list\`'s own total. */
export const discoveryMethodTotal = discoveryServices.reduce(
  (total, service) => total + service.methods,
  0,
);

/** Sum of \`scopes\` across every service, counting repeats between services. */
export const discoveryScopeTotal = discoveryServices.reduce(
  (total, service) => total + service.scopes,
  0,
);

/** How many services the index carries. */
export const discoveryServiceCount = discoveryServices.length;
`;

if (CHECK_ONLY) {
  const current = await readFile(OUT_FILE, 'utf8').catch(() => '');
  if (current !== body) {
    console.error('site/src/data/discovery-coverage.ts is out of date. Run: node scripts/generate-coverage.ts');
    process.exit(1);
  }
  console.log('site/src/data/discovery-coverage.ts is current.');
} else {
  await writeFile(OUT_FILE, body);
  console.log(
    `wrote site/src/data/discovery-coverage.ts — ${serviceTotal} services, ${methodTotal} methods, ${scopeTotal} service-scopes (index generated ${manifest.generatedAt})`,
  );
}
