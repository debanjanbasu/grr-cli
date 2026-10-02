#!/usr/bin/env node
// Generate the service command tree (src/commands/generated.rs) from the
// committed Discovery index (src/discovery/*.json).
//
// Why generate at all: the index is the single source of truth for the
// CLI's command surface and is refreshed daily by
// .github/workflows/discovery.yml, so the tree must be derived from it —
// never hand-written. Derive macros cannot be generated (they are
// compile-time Rust), so the tree is emitted with clap's builder API.
//
// Every leaf carries its full dotted method id as its clap name (with a
// visible alias of the bare method name), one typed flag per Discovery
// parameter, and the same escape hatches as `grr api call` (--params,
// --body-file, --query, --dry-run, -f/--format). Dispatch lives in
// src/commands/gen_dispatch.rs and resolves the id via discovery::resolve.
//
// Output is deterministic: no timestamps from the wall clock (the
// generated-at marker is the index manifest's own timestamp), sorted
// everywhere, and a re-run over an unchanged index is byte-identical.
//
// NOTE: the .mjs spellings inside the emitted doc header below are pinned
// on purpose — the committed src/commands/generated.rs embeds them, and
// rewording the emitted text would churn the whole generated file for a
// comment rename. Change them only together with a deliberate regeneration.
//
// Usage:
//   node scripts/generate-commands.ts         # write src/commands/generated.rs
//   node scripts/generate-commands.ts --check # exit 1 if stale (CI gate)

import { readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { DistilledIndex, DistilledMethod, DistilledParameter, Manifest } from './discovery-index.ts';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const INDEX_DIR = resolve(ROOT, 'src', 'discovery');
const OUT_FILE = resolve(ROOT, 'src', 'commands', 'generated.rs');
const MANIFEST = resolve(INDEX_DIR, 'manifest.json');

const CHECK_ONLY = process.argv.includes('--check');

// The hand-written top-level commands; a service may never shadow them.
const STATIC_TOP_LEVEL = new Set(['auth', 'api', 'schema', 'transport']);

// Escape-hatch flag ids present on every generated leaf (plus clap's own
// `help`). A Discovery parameter whose kebab-case name lands here is
// renamed `param-<kebab>` — see gen_dispatch::flag_id, which must agree.
const RESERVED_FLAG_IDS = new Set(['params', 'body-file', 'query', 'dry-run', 'format', 'help']);

function assert(condition: unknown, message: string): void {
  if (!condition) {
    console.error(`generate-commands: ${message}`);
    process.exit(1);
  }
}

/** camelCase (or dotted, or $-prefixed) Discovery name -> kebab-case flag. */
function kebab(name: string): string {
  return name
    .replace(/([a-z0-9])([A-Z])/g, '$1-$2')
    .replace(/[^a-zA-Z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .toLowerCase();
}

function flagId(name: string): string {
  const s = kebab(name);
  return RESERVED_FLAG_IDS.has(s) ? `param-${s}` : s;
}

/** snake_case fn-name suffix for one id segment (getProfile -> get_profile). */
function snake(name: string): string {
  return kebab(name).replaceAll('-', '_');
}

/** VALUE_NAME for a flag id (user-id -> USER_ID). */
function valueName(flagId: string): string {
  return flagId.replaceAll('-', '_').toUpperCase();
}

/** Markdown links render as their text; help output is plain text. */
function stripMarkdown(text: string): string {
  return (text ?? '').replace(/\[([^\]]+)\]\([^)]*\)/g, '$1');
}

/** One line, collapsed whitespace, truncated at a word boundary. */
function oneLine(text: string, max = 100): string {
  const t = stripMarkdown(text).replace(/\s+/g, ' ').trim();
  if (t.length <= max) return t;
  const cut = t.slice(0, max);
  const boundary = cut.lastIndexOf(' ');
  const kept = boundary > max * 0.6 ? cut.slice(0, boundary) : cut;
  return kept.replace(/[\s,;:.]$/, '') + '...';
}

/** Collapse for long_about: full text, single line, links kept? No — same strip. */
function helpText(text: string): string {
  return stripMarkdown(text).replace(/\s+/g, ' ').trim();
}

/** Escape a Rust string literal body (no quotes added). */
function rs(s: string): string {
  return s
    .replace(/\\/g, '\\\\')
    .replace(/"/g, '\\"')
    .replace(/\r/g, '')
    .replace(/\n/g, '\\n');
}

// ── Load the index ────────────────────────────────────────────────────────

async function loadIndex(): Promise<DistilledIndex[]> {
  // The manifest is the single source of truth for WHICH services exist —
  // deriving the list here instead of keeping a second hardcoded copy is
  // what keeps the generator from silently drifting behind the index when a
  // new Workspace API is added to fetch-discovery.ts.
  const manifest = JSON.parse(await readFile(MANIFEST, 'utf8')) as Manifest;
  const serviceNames = Object.keys(manifest.services ?? {}).sort();
  assert(serviceNames.length > 0, 'manifest lists no services; run fetch-discovery.ts first');

  const services: DistilledIndex[] = [];
  for (const name of serviceNames) {
    const body = await readFile(resolve(INDEX_DIR, `${name}.json`), 'utf8');
    // The asserts below are the runtime validation of this cast: the file is
    // written by fetch-discovery.ts, and the shape both sides share is the
    // contract in ./discovery-index.ts.
    const index = JSON.parse(body) as DistilledIndex;
    assert(index.name === name, `${name}.json claims to be ${index.name}`);
    assert(
      !STATIC_TOP_LEVEL.has(name),
      `service name \`${name}\` would shadow a hand-written top-level command`,
    );
    assert(Array.isArray(index.methods) && index.methods.length > 0, `${name}.json has no methods`);
    for (const method of index.methods) {
      assert(method.id && method.httpMethod && method.path !== undefined,
        `${name}: malformed method entry ${JSON.stringify(method).slice(0, 80)}`);
      for (const segment of method.id.split('.')) {
        assert(/^[A-Za-z][A-Za-z0-9]*$/.test(segment),
          `${name}.${method.id}: id segment \`${segment}\` is not a usable subcommand name`);
        assert(segment !== 'help', `${name}.${method.id}: \`help\` collides with clap's own subcommand`);
      }
    }
    services.push(index);
  }
  return services;
}

// ── Build the resource tree ───────────────────────────────────────────────

/**
 * One node per resource path. `order` records the deterministic child
 * order: first-encounter while walking methods in the index's sorted id
 * order, so `grr <svc> --help` lists children alphabetically-by-id without
 * a separate sort. A leaf (method) and a child resource may never share a
 * name — clap would reject the duplicate subcommand.
 */
interface TreeNode {
  children: Map<string, TreeNode>;
  leaves: Map<string, DistilledMethod>;
  order: string[];
}

function buildTree(service: DistilledIndex): TreeNode {
  const root: TreeNode = { children: new Map(), leaves: new Map(), order: [] };
  const seen = new Set<string>();
  for (const method of service.methods) {
    assert(!seen.has(method.id), `${service.name}: duplicate method id ${method.id}`);
    seen.add(method.id);
    const segments = method.id.split('.');
    let node = root;
    for (const segment of segments.slice(0, -1)) {
      if (!node.children.has(segment)) {
        assert(!node.leaves.has(segment),
          `${service.name}.${method.id}: method name \`${segment}\` collides with a sibling resource of the same name`);
        node.children.set(segment, { children: new Map(), leaves: new Map(), order: [] });
        node.order.push(segment);
      }
      // The child exists by construction: created just above when missing.
      node = node.children.get(segment)!;
    }
    const leaf = segments[segments.length - 1];
    assert(!node.children.has(leaf),
      `${service.name}.${method.id}: resource name \`${leaf}\` collides with a sibling method of the same name`);
    if (!node.leaves.has(leaf)) {
      node.leaves.set(leaf, method);
      node.order.push(leaf);
    }
  }
  return root;
}

// ── Rust emission ─────────────────────────────────────────────────────────

const fnNames = new Set<string>();
function uniqueFn(base: string): string {
  assert(!fnNames.has(base), `fn name collision after snake-casing: ${base} (two ids collapse to the same fn name)`);
  fnNames.add(base);
  return base;
}

function leafFnName(service: DistilledIndex, method: DistilledMethod): string {
  return uniqueFn(`leaf_${snake(service.name)}_${method.id.split('.').map(snake).join('_')}`);
}

function groupFnName(service: DistilledIndex, path: string[]): string {
  return uniqueFn(`group_${snake(service.name)}_${path.map(snake).join('_')}`);
}

function serviceFnName(service: DistilledIndex): string {
  return uniqueFn(`service_${snake(service.name)}`);
}

/** One `.arg(...)` chain for a Discovery parameter. */
function emitParamArg(method: DistilledMethod, param: DistilledParameter, lines: string[]): void {
  const id = flagId(param.name);
  const renamed = id !== kebab(param.name);
  let arg = `Arg::new("${rs(id)}").long("${rs(id)}")`;

  if (param.type === 'boolean' && !param.repeated) {
    arg += '.action(ArgAction::SetTrue)';
  } else {
    arg += `.value_name("${rs(valueName(id))}")`;
    if (param.type === 'integer') {
      arg += '.value_parser(clap::value_parser!(i64))';
    }
    if (param.repeated) {
      arg += '.action(ArgAction::Append)';
    } else if (Array.isArray(param.enum) && param.enum.length > 0) {
      // Possible values only for single-valued params: a repeated
      // occurrence list and a validator fight over the same value.
      arg += `.value_parser([${param.enum.map((v) => `"${rs(String(v))}"`).join(', ')}])`;
    }
  }
  if (param.required) {
    arg += '.required(true)';
  }

  const description = helpText(param.description);
  const help = renamed
    ? `Discovery parameter \`${param.name}\`, renamed because --${kebab(param.name)} is reserved here. ${description}`
    : description;
  lines.push(`            .arg(${arg}${help ? `\n                .help("${rs(help)}")` : ''})`);
}

function emitLeaf(service: DistilledIndex, method: DistilledMethod, lines: string[]): void {
  const fn = leafFnName(service, method);
  // loadIndex asserts a truthy dotted id, so the last segment always exists.
  const leafName = method.id.split('.').at(-1)!;
  const dottedId = `${service.name}.${method.id}`;
  const about = oneLine(method.description);
  const longAbout = helpText(method.description);

  lines.push('');
  lines.push(`    // ${dottedId}`);
  lines.push(`    fn ${fn}() -> Command {`);
  lines.push(`        Command::new("${rs(dottedId)}")`);
  lines.push(`            .visible_alias("${rs(leafName)}")`);
  lines.push(`            .about("${rs(about)}")`);
  if (longAbout && longAbout !== about) {
    lines.push(`            .long_about("${rs(longAbout)}")`);
  }
  for (const param of method.parameters) {
    emitParamArg(method, param, lines);
  }
  lines.push('            .args(escape_hatch_args())');
  lines.push('    }');
}

function emitNode(service: DistilledIndex, node: TreeNode, path: string[], lines: string[]): void {
  // Depth-first: every child fn must exist before the parent references it.
  // The `.get(...)!` lookups are safe for the same reason: every key in
  // `order` was placed into `children` or `leaves` by buildTree.
  for (const key of node.order) {
    if (node.leaves.has(key)) {
      emitLeaf(service, node.leaves.get(key)!, lines);
    } else {
      emitNode(service, node.children.get(key)!, [...path, key], lines);
    }
  }
  const isRoot = path.length === 0;
  const fn = isRoot ? serviceFnName(service) : groupFnName(service, path);
  const name = isRoot ? service.name : path.at(-1)!;
  const dotted = `${service.name}${path.length ? `.${path.join('.')}` : ''}`;
  lines.push('');
  lines.push(`    // ${dotted}`);
  lines.push(`    fn ${fn}() -> Command {`);
  lines.push(`        Command::new("${rs(name)}")`);
  if (isRoot) {
    lines.push(`            .about("${rs(`${service.title} operations (${service.version}, ${service.methods.length} methods)`)}")`);
  } else {
    lines.push(`            .about("${rs(`Methods under ${dotted}`)}")`);
  }
  // Bare `grr gmail` (or any resource group) is a usage error, not a
  // silent no-op — the same behaviour the old derive surface had.
  lines.push('            .subcommand_required(true)');
  for (const key of node.order) {
    const childFn = node.leaves.has(key)
      ? `leaf_${[snake(service.name), ...path.map(snake), snake(key)].join('_')}`
      : `group_${[snake(service.name), ...path.map(snake), snake(key)].join('_')}`;
    lines.push(`            .subcommand(${childFn}())`);
  }
  lines.push('    }');
}

function countGroups(node: TreeNode): number {
  let n = node.children.size;
  for (const child of node.children.values()) n += countGroups(child);
  return n;
}

async function main(): Promise<void> {
  const services = await loadIndex();

  let generatedAt = 'unknown';
  try {
    const manifest = JSON.parse(await readFile(MANIFEST, 'utf8')) as Manifest;
    if (manifest.generatedAt) generatedAt = manifest.generatedAt;
  } catch {
    // No manifest yet (fresh checkout before the first fetch): "unknown"
    // keeps generation deterministic instead of freezing a wall-clock date.
  }

  const stats = {
    services: services.length,
    methods: 0,
    leaves: 0,
    groups: 0,
    flags: 0,
  };

  const lines: string[] = [];
  lines.push('//! GENERATED FILE — DO NOT EDIT BY HAND.');
  lines.push('//!');
  lines.push('//! The entire service command tree, compiled from the committed');
  lines.push('//! Discovery index (`src/discovery/*.json`) by');
  lines.push('//! `scripts/generate-commands.mjs`. The index is refreshed daily by');
  lines.push('//! `.github/workflows/discovery.yml`; after any index change,');
  lines.push('//! regenerate with `node scripts/generate-commands.mjs` (or run it');
  lines.push('//! with `--check`, which exits 1 when this file is stale).');
  lines.push('//!');
  lines.push(`//! Generated at: ${generatedAt} — the index manifest's own timestamp,`);
  lines.push('//! so regeneration is byte-identical until the index actually changes.');
  lines.push('//!');
  lines.push('//! Shape:');
  lines.push('//!   * one top-level subcommand per service;');
  lines.push('//!   * nested resources nest as subcommands (gmail -> users -> messages),');
  lines.push('//!     verbatim id segments, never flattened;');
  lines.push('//!   * one leaf per method. A leaf\'s clap name IS its full dotted');
  lines.push('//!     method id (`gmail.users.messages.list`), with a visible alias of');
  lines.push('//!     the bare method name, so dispatch resolves the id without');
  lines.push('//!     reconstructing paths and `grr schema` self-documents every');
  lines.push('//!     callable id;');
  lines.push('//!   * every leaf flag set = one typed flag per Discovery parameter');
  lines.push('//!     (camelCase -> kebab-case; integer -> i64; boolean -> presence');
  lines.push('//!     flag; repeated -> repeatable; enum -> possible values; required');
  lines.push('//!     -> required) plus the escape hatches `--params`, `--body-file`,');
  lines.push('//!     `--query`, `--dry-run`, `-f/--format`.');
  lines.push('//!');
  lines.push('//! Honest limitation: request bodies are NOT typed. Discovery\'s request');
  lines.push('//! schemas are not part of the distilled index, so POST/PATCH/PUT');
  lines.push('//! bodies pass through `--params` / `--body-file` verbatim.');

  const body: string[] = [];

  body.push(`    /// All ${services.length} service commands, in \`discovery::services()\` order.`);
  body.push('    pub fn commands() -> Vec<Command> {');
  body.push('        vec![');
  for (const service of services) {
    body.push(`            service_${snake(service.name)}(),`);
  }
  body.push('        ]');
  body.push('    }');

  for (const service of services) {
    const tree = buildTree(service);
    stats.methods += service.methods.length;
    stats.leaves += service.methods.length;
    stats.groups += countGroups(tree);
    stats.flags += service.methods.reduce((n, m) => n + m.parameters.length, 0);
    emitNode(service, tree, [], body);
  }

  lines.push('//!');
  lines.push(`//! Numbers: ${stats.services} services, ${stats.methods} methods, ${stats.leaves} leaves, ${stats.groups} resource groups, ${stats.flags} typed flags.`);
  lines.push('');
  lines.push('/// When the generator last ran, taken from the Discovery index');
  lines.push('/// manifest\'s own timestamp so it only moves when the index moves.');
  lines.push(`pub const GENERATED_AT: &str = "${rs(generatedAt)}";`);
  lines.push('');
  lines.push('#[rustfmt::skip] // mechanical output; formatting it would churn every diff');
  lines.push('pub mod tree {');
  lines.push('    use clap::{Arg, ArgAction, Command};');
  lines.push('    use crate::output::OutputFormat;');
  lines.push('');
  lines.push('    /// The escape hatches every generated leaf shares with `grr api call`.');
  lines.push('    /// Dispatch in gen_dispatch.rs reads them by these exact ids.');
  lines.push('    fn escape_hatch_args() -> Vec<Arg> {');
  lines.push('        vec![');
  lines.push('            Arg::new("params").long("params").value_name("JSON")');
  lines.push('                .help("Method parameters as a JSON object. Merged before the typed flags, so a typed flag always wins on conflict"),');
  lines.push('            Arg::new("body-file").long("body-file").value_name("PATH|-")');
  lines.push('                .help("Read the request body from a file (\'-\' = stdin) and send it verbatim (POST/PATCH/PUT)"),');
  lines.push('            Arg::new("query").long("query").value_name("KEY=VALUE").action(ArgAction::Append)');
  lines.push('                .help("Extra query pair for parameters Discovery does not document (e.g. alt=json); repeatable"),');
  lines.push('            Arg::new("dry-run").long("dry-run").action(ArgAction::SetTrue)');
  lines.push('                .help("Print the request that would be sent, without sending it"),');
  lines.push('            Arg::new("format").short(\'f\').long("format").value_name("FORMAT")');
  lines.push('                .value_parser(clap::value_parser!(OutputFormat)).default_value("json")');
  lines.push('                .help("Output format"),');
  lines.push('        ]');
  lines.push('    }');
  lines.push(...body);
  lines.push('}');
  lines.push('');

  const content = lines.join('\n');
  // Compare line-ending-independently: a Windows CI runner checks this file
  // out with CRLF (autocrlf), while the generator always writes LF. A raw
  // byte compare would cry "stale" on every fresh Windows checkout.
  const previous = (await readFile(OUT_FILE, 'utf8').catch(() => null))?.replace(/\r\n/g, '\n') ?? null;

  if (CHECK_ONLY) {
    if (previous !== content) {
      console.error('\nGenerated command tree is stale. Run: node scripts/generate-commands.ts');
      process.exit(1);
    }
    console.log(`generated tree current: ${stats.services} services, ${stats.methods} methods, ${stats.flags} typed flags`);
    return;
  }

  if (previous !== content) {
    await writeFile(OUT_FILE, content, 'utf8');
    console.log('wrote src/commands/generated.rs');
  } else {
    console.log('src/commands/generated.rs already up to date');
  }
  console.log(
    `${stats.services} services, ${stats.methods} methods, ${stats.leaves} leaves, ` +
      `${stats.groups} resource groups, ${stats.flags} typed flags, ` +
      `${(content.length / 1024).toFixed(0)} KiB`,
  );
}

main().catch((error: unknown) => {
  console.error(`generate-commands failed: ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
});
