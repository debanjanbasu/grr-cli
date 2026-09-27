#!/usr/bin/env node
// Fetch Google's Discovery documents and distil them into a compact index
// that grr embeds.
//
// Why distil instead of embedding the raw documents:
//   - The raw Gmail doc is ~1.5 MB. All ten services together would add
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

import { mkdir, readdir, writeFile, readFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { DistilledIndex, DistilledMethod, Manifest, ManifestServiceEntry } from './discovery-index.ts';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const OUT_DIR = resolve(ROOT, 'src', 'discovery');

// name -> [discovery url, api version]
export const SERVICES: Record<string, readonly [discoveryUrl: string, apiVersion: string]> = {
  gmail: ['https://gmail.googleapis.com/$discovery/rest?version=v1', 'v1'],
  calendar: ['https://www.googleapis.com/discovery/v1/apis/calendar/v3/rest', 'v3'],
  drive: ['https://www.googleapis.com/discovery/v1/apis/drive/v3/rest', 'v3'],
  people: ['https://people.googleapis.com/$discovery/rest?version=v1', 'v1'],
  chat: ['https://chat.googleapis.com/$discovery/rest?version=v1', 'v1'],
  forms: ['https://forms.googleapis.com/$discovery/rest?version=v1', 'v1'],
  tasks: ['https://tasks.googleapis.com/$discovery/rest?version=v1', 'v1'],
  docs: ['https://docs.googleapis.com/$discovery/rest?version=v1', 'v1'],
  sheets: ['https://sheets.googleapis.com/$discovery/rest?version=v4', 'v4'],
  slides: ['https://slides.googleapis.com/$discovery/rest?version=v1', 'v1'],
};

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
}

interface RawResource {
  methods?: Record<string, RawMethod>;
  resources?: Record<string, RawResource>;
}

interface RawDiscoveryDocument {
  resources?: Record<string, RawResource>;
  revision?: string;
  title?: string;
  rootUrl?: string;
  servicePath?: string;
  basePath?: string;
  batchPath?: string;
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
          }))
          .sort((a, b) => a.name.localeCompare(b.name)),
      });
    }
  }
  if (node.resources && typeof node.resources === 'object') {
    for (const [rname, r] of Object.entries(node.resources)) {
      collect(r, path ? `${path}.${rname}` : rname, out);
    }
  }
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

  for (const [name, [url, version]] of Object.entries(SERVICES)) {
    const doc = await fetchJson(url);
    const methods: DistilledMethod[] = [];
    // Iterate entries, not values: the resource *name* is part of the method
    // id (gmail's `users.messages.list`, not `messages.list`).
    for (const [resourceName, resource] of Object.entries(doc.resources ?? {})) {
      collect(resource, resourceName, methods);
    }
    methods.sort((a, b) => a.id.localeCompare(b.id));

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
  for (const f of files) bytes += (await readFile(resolve(OUT_DIR, f))).length;

  console.log('---');
  console.log(`${Object.keys(SERVICES).length} services, ${totalMethods} methods total`);
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
