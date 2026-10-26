#!/usr/bin/env node
// Fetch Google's Discovery documents and distil them into a compact index
// that grr embeds.
//
// Why distil instead of embedding the raw documents:
//   - The raw Gmail doc is ~1.5 MB. Every service together would add
//     several MB to a binary whose size is a selling point.
//   - To *invoke* a method we only need: the method id, HTTP verb, path
//     template, its scopes, and its parameters. Full request/response
//     schemas are only needed to generate typed models, which the generic
//     invoker does not use.
//
// The output is deterministic (sorted keys) so a re-run with unchanged
// upstream produces an empty diff, which keeps the scheduled refresh
// workflow quiet.
//
// Usage:
//   node scripts/fetch-discovery.ts            # write src/discovery/*.json
//   node scripts/fetch-discovery.ts --check    # exit 1 if stale (CI)

import { mkdir, readdir, writeFile, readFile, stat } from 'node:fs/promises';
import { resolve } from 'node:path';
import type { DistilledIndex, DistilledMethod, DistilledSchema, Manifest, ManifestServiceEntry } from './discovery-index.ts';

const ROOT = resolve(import.meta.dirname, '..');
const OUT_DIR = resolve(ROOT, 'src', 'discovery');

// The Workspace family grr covers: every CONSUMER-available Workspace API
// in Google's Discovery index. Enterprise-only APIs (Keep, Admin SDK,
// Cloud Identity, Cloud Search, Meet, Licensing, Reseller, Groups
// Migration/Settings) are deliberately excluded — consumer accounts cannot
// authorise them, so their methods would only ever 403.
//
// Versions and endpoint URLs are resolved FROM THE LIVE DISCOVERY INDEX at
// fetch time, not hardcoded: when Google ships a new preferred version
// (drive v4, ...) or moves an endpoint, the daily refresh picks it up
// without a script change. A NEW Workspace API is one line added here, and
// the daily workflow then regenerates the index, the command tree, and
// every doc page that renders from them.
export const WORKSPACE_APIS: readonly string[] = [
  'gmail',
  'calendar',
  'drive',
  'people',
  'chat',
  'forms',
  'tasks',
  'docs',
  'sheets',
  'slides',
  'script',
  'analyticsadmin',
  'analyticsdata',
  'searchconsole',
];

interface DiscoveryIndexEntry {
  name: string;
  version: string;
  preferred?: boolean;
  discoveryRestUrl?: string;
  title?: string;
}

/** Resolve each Workspace API's preferred version + endpoint from the live index. */
export async function resolveServices(): Promise<Array<[string, string, string]>> {
  const index = (await fetchJson('https://www.googleapis.com/discovery/v1/apis')) as {
    items?: DiscoveryIndexEntry[];
  };
  const items = index.items ?? [];
  const resolved: Array<[string, string, string]> = [];
  for (const name of WORKSPACE_APIS) {
    const api = items.find((entry) => entry.name === name && entry.preferred === true);
    const url = api?.discoveryRestUrl;
    if (!api || !url) {
      console.warn(`warning: ${name} not found in the discovery index (no preferred version); skipping`);
      continue;
    }
    resolved.push([name, url, api.version]);
  }
  return resolved;
}

const CHECK_ONLY = process.argv.includes('--check');

// Google's raw Discovery document shape — only the fields the distiller
// reads, and only as much structure as it relies on. The raw JSON is
// external input, but the fields below are spec-mandated on every document
// Google serves; fields that are genuinely absent in the wild (`flatPath`,
// `scopes`, `parameters`, ...) stay optional and every read keeps the exact
// `??` fallback the original .mjs used, so a degraded document distils to
// byte-identical output.
interface RawParameter {
  type?: string;
  repeated?: boolean;
  location?: string;
  description?: string;
  enum?: Array<string | number>;
  enumDescriptions?: string[];
  format?: string;
  default?: string;
  minimum?: number | string;
  maximum?: number | string;
  pattern?: string;
}

interface RawSchema {
  type?: string;
  properties?: Record<string, RawSchema>;
  items?: RawSchema;
  enum?: Array<string | number>;
  required?: string[];
  description?: string;
  $ref?: string;
  additionalProperties?: RawSchema | boolean;
}

interface RawMethod {
  // Spec-required on every Discovery method; read without a fallback,
  // exactly as before.
  httpMethod: string;
  path: string;
  flatPath?: string;
  description?: string;
  scopes?: string[];
  parameterOrder?: string[];
  parameters?: Record<string, RawParameter>;
  response?: { $ref?: string };
}

interface RawResource {
  methods?: Record<string, RawMethod>;
  resources?: Record<string, RawResource>;
}

interface RawDiscoveryDocument {
  resources?: Record<string, RawResource>;
  methods?: Record<string, RawMethod>;
  revision?: string;
  title?: string;
  rootUrl?: string;
  servicePath?: string;
  basePath?: string;
  batchPath?: string;
  schemas?: Record<string, RawSchema>;
}

/** Uniform `message` extraction — catch params are `unknown` under strict TS. */
function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/** Flatten a discovery `resources` tree into `a.b.c.methodId` entries. */
function collect(node: RawResource | undefined, path: string, out: DistilledMethod[]): void {
  if (!node || typeof node !== 'object') return;
  if (node.methods && typeof node.methods === 'object') {
    for (const [name, m] of Object.entries(node.methods)) {
      const id = path ? `${path}.${name}` : name;
      out.push({
        id,
        httpMethod: m.httpMethod,
        path: m.path,
        flatPath: m.flatPath ?? null,
        description: (m.description ?? '').replace(/\s+/g, ' ').trim().slice(0, 300),
        // Prefer least-privilege: discovery lists scopes best-first, and the
        // broad ones (https://mail.google.com/) come first. We keep the whole
        // list but record the narrowest for least-privilege defaults.
        scopes: m.scopes ?? [],
        // Discovery marks optional params by leaving them out of
        // parameterOrder; required ones appear there.
        required: m.parameterOrder ?? [],
        parameters: Object.entries(m.parameters ?? {})
          .map(([pname, p]) => ({
            name: pname,
            type: p.type ?? 'string',
            required: (m.parameterOrder ?? []).includes(pname),
            repeated: p.repeated === true,
            location: p.location ?? null,
            description: (p.description ?? '').replace(/\s+/g, ' ').trim().slice(0, 160),
            enum: p.enum ?? null,
            enumDescriptions: p.enumDescriptions ?? null,
            format: p.format ?? null,
            default: p.default ?? null,
            // Discovery serves bounds as JSON numbers in most services but
            // as strings ("1") in calendar; normalize to number | null.
            minimum: normalizeBound(p.minimum),
            maximum: normalizeBound(p.maximum),
            pattern: p.pattern ?? null,
          }))
          .sort((a, b) => a.name.localeCompare(b.name)),
        responseRef: m.response?.$ref ?? null,
      });
    }
  }
  if (node.resources && typeof node.resources === 'object') {
    for (const [rname, r] of Object.entries(node.resources)) {
      collect(r, path ? `${path}.${rname}` : rname, out);
    }
  }
}

/** Prune a raw Discovery schema to the shape an outputSchema needs: field
 * names, types, array/enum shapes, and required markers. Descriptions are
 * deliberately dropped — they are the bulk of the raw size (~1.2 MB across
 * 14 services) and the shape is what agents resolve against. `$ref` links
 * resolve one level (a property that is just a `$ref` becomes an object
 * with the referenced schema's shape inline). */
/** Discovery bounds arrive as numbers in most services but as strings
 * ("1") in calendar's document; coerce to number | null. */
function normalizeBound(v: number | string | undefined): number | null {
  if (v === undefined || v === null) return null;
  const n = typeof v === 'number' ? v : Number(v);
  return Number.isFinite(n) ? n : null;
}

function pruneSchema(s: RawSchema | undefined, schemas: Record<string, RawSchema>, depth: number): DistilledSchema | null {
  if (!s || typeof s !== 'object') return null;
  if (s.$ref) {
    if (depth > 2) return { type: 'object' }; // cycle guard; names stay visible in properties
    return pruneSchema(schemas[s.$ref], schemas, depth + 1);
  }
  const out: DistilledSchema = { type: s.type ?? 'object' };
  if (s.properties && typeof s.properties === 'object') {
    const props: Record<string, DistilledSchema> = {};
    for (const [k, v] of Object.entries(s.properties)) {
      const pruned = pruneSchema(v, schemas, depth);
      if (pruned) props[k] = pruned;
    }
    if (Object.keys(props).length > 0) out.properties = props;
  }
  if (s.items) {
    const items = pruneSchema(s.items, schemas, depth);
    if (items) out.items = items;
  }
  if (s.enum) out.enum = s.enum;
  if (s.required) out.required = s.required;
  return out;
}

async function fetchJson(url: string): Promise<RawDiscoveryDocument> {
  const res = await fetch(url, {
    headers: { Accept: 'application/json', 'User-Agent': 'grr-cli-discovery/1.0 (+https://github.com/debanjanbasu/grr-cli)' },
    signal: AbortSignal.timeout(30000),
  });
  if (!res.ok) throw new Error(`${url} -> HTTP ${res.status}`);
  // Trusted exactly as far as the interfaces above describe it: external
  // JSON, with every genuinely-optional field read through a `??` fallback.
  return (await res.json()) as RawDiscoveryDocument;
}

async function main(): Promise<void> {
  await mkdir(OUT_DIR, { recursive: true });

  const manifest: Manifest = { generatedAt: new Date().toISOString(), services: {} };
  let totalMethods = 0;
  let changed = false;

  const services = await resolveServices();
  for (const [name, url, version] of services) {
    const doc = await fetchJson(url);
    const methods: DistilledMethod[] = [];
    // Iterate entries, not values: the resource *name* is part of the method
    // id (gmail's `users.messages.list`, not `messages.list`).
    for (const [resourceName, resource] of Object.entries(doc.resources ?? {})) {
      collect(resource, resourceName, methods);
    }
    // Top-level methods (no resource path): id is just the method name.
    for (const [name, m] of Object.entries(doc.methods ?? {})) {
      collect({ methods: { [name]: m } }, '', methods);
    }
    methods.sort((a, b) => a.id.localeCompare(b.id));

    const schemas: Record<string, DistilledSchema> = {};
    for (const [name, schema] of Object.entries(doc.schemas ?? {})) {
      const pruned = pruneSchema(schema, doc.schemas ?? {}, 0);
      if (pruned) schemas[name] = pruned;
    }

    const scopes = [...new Set(methods.flatMap((m) => m.scopes))].sort();
    const index: DistilledIndex = {
      name,
      version,
      revision: doc.revision ?? null,
      title: doc.title ?? name,
      rootUrl: doc.rootUrl ?? null,
      servicePath: doc.servicePath ?? '',
      basePath: doc.basePath ?? '',
      batchPath: doc.batchPath ?? null,
      methods,
      schemas,
    };

    const file = resolve(OUT_DIR, `${name}.json`);
    const body = `${JSON.stringify(index)}\n`;
    let previous: string | null = null;
    try {
      previous = await readFile(file, 'utf8');
    } catch {
      /* first run */
    }
    if (previous !== body) changed = true;

    if (!CHECK_ONLY) await writeFile(file, body, 'utf8');

    const entry: ManifestServiceEntry = {
      methods: methods.length,
      scopes: scopes.length,
      revision: index.revision,
    };
    manifest.services[name] = entry;
    totalMethods += methods.length;
    console.log(
      `${name.padEnd(9)} ${String(methods.length).padStart(4)} methods  ` +
        `${String(scopes.length).padStart(3)} scopes  rev ${index.revision ?? '?'}`,
    );
  }

  if (!CHECK_ONLY && changed) {
    // Only rewrite the manifest when an index file actually changed. The
    // generatedAt stamp means an unconditional write produces a daily diff,
    // which makes the daily workflow open a PR even when Google's documents
    // did not move at all. The manifest is committed alongside the index, so
    // it cannot be missing unless the index changed too.
    await writeFile(resolve(OUT_DIR, 'manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
  }

  const files = (await readdir(OUT_DIR)).filter((f) => f.endsWith('.json'));
  let bytes = 0;
  // stat().size, not readFile().length: the count only needs the byte
  // length, and reading megabytes of JSON to measure them was pure waste.
  for (const f of files) bytes += (await stat(resolve(OUT_DIR, f))).size;

  console.log('---');
  console.log(`${services.length} services, ${totalMethods} methods total`);
  console.log(`index size: ${(bytes / 1024).toFixed(0)} KiB across ${files.length} files`);

  if (CHECK_ONLY && changed) {
    console.error('\nDiscovery index is stale. Run: node scripts/fetch-discovery.ts');
    process.exit(1);
  }
}

main().catch((error: unknown) => {
  console.error(`discovery refresh failed: ${errorMessage(error)}`);
  process.exit(1);
});
