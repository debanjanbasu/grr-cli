#!/usr/bin/env node
// Keep the GitHub repo's public metadata true without a human: the
// description carries the method/service counts that the Discovery index
// generates, and those counts move whenever Google ships an API method. A
// description with a stale count is the first thing a visitor reads.
//
// The counts come from the committed index (the same source the command
// tree, the skills, and the site's coverage table derive from), so this
// script needs no network beyond the two API calls it makes.
//
// Usage: node scripts/update-repo-metadata.ts [--check]
//
// Auth: uses the ambient gh credentials (locally) or GH_TOKEN (CI). In CI
// the default GITHUB_TOKEN CANNOT write repo metadata — the Administration
// permission is not available to workflow tokens — so the workflow passes a
// fine-grained PAT secret. When GH_TOKEN is unset the script no-ops with a
// notice; when it is set but the API refuses, the script fails loudly (a
// configured token that cannot do its job is a real error).
//
// --check: report drift without writing (exit 1 when the repo differs).

import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';

const REPO = 'debanjanbasu/grr-cli';
const HOMEPAGE = 'https://grr-cli.pages.dev';

// Hand-chosen topics; stable unless a deliberate change lands here.
const TOPICS = [
  'ai-agents',
  'calendar',
  'cli',
  'command-line-tool',
  'gmail',
  'gmail-api',
  'google-drive',
  'google-workspace',
  'http3',
  'mcp',
  'model-context-protocol',
  'oauth2',
  'quic',
  'rust',
];

const checkOnly = process.argv.includes('--check');

function gh(args: string[], tolerateFailure = false): { ok: boolean; out: string } {
  const result = spawnSync('gh', args, { encoding: 'utf8' });
  if (result.status !== 0 && !tolerateFailure) {
    console.error(`gh ${args.join(' ')} failed: ${result.stderr.trim()}`);
    process.exit(1);
  }
  return { ok: result.status === 0, out: (result.stdout ?? '').trim() };
}

interface ManifestService {
  methods: number;
}

const manifest = JSON.parse(readFileSync('src/discovery/manifest.json', 'utf8')) as unknown;
const services = (() => {
  if (manifest && typeof manifest === 'object' && 'services' in manifest) {
    const value = (manifest as Record<string, unknown>).services;
    if (value && typeof value === 'object') return Object.values(value) as ManifestService[];
  }
  return [];
})();
if (services.length === 0) {
  console.error('update-repo-metadata: no services in src/discovery/manifest.json');
  process.exit(1);
}
const methodTotal = services.reduce((sum, s) => sum + (typeof s.methods === 'number' ? s.methods : 0), 0);

const description =
  `Google tools from the terminal: ${methodTotal} methods across ${services.length} Google APIs in one ` +
  'generated command tree — zero-config, HTTP/3, offline-first, agent-native (ask/MCP/skills). MIT.';

const current = gh(['api', `repos/${REPO}`, '--jq', '{description: (.description // ""), homepage: (.homepage // ""), topics: (.topics // [])}'], true);
if (!current.ok) {
  if (!process.env.GH_TOKEN) {
    console.log('::notice::repo metadata not checked: gh is unauthenticated here and GH_TOKEN is unset.');
    process.exit(0);
  }
  console.error(`update-repo-metadata: cannot read ${REPO} with the configured token.`);
  process.exit(1);
}
const live = JSON.parse(current.out) as { description: string; homepage: string; topics: string[] };

const topicDrift = [...TOPICS].sort().join(',') !== [...live.topics].sort().join(',');
const fieldDrift = live.description !== description || live.homepage !== HOMEPAGE;
if (!topicDrift && !fieldDrift) {
  console.log(`repo metadata current: ${methodTotal} methods / ${services.length} APIs`);
  process.exit(0);
}

if (checkOnly) {
  console.error(`::error::repo metadata is stale (description drift=${live.description !== description}, homepage drift=${live.homepage !== HOMEPAGE}, topics drift=${topicDrift})`);
  process.exit(1);
}

if (fieldDrift) {
  gh(['api', '-X', 'PATCH', `repos/${REPO}`, '-f', `description=${description}`, '-f', `homepage=${HOMEPAGE}`]);
  console.log(`updated description + homepage (${methodTotal} methods / ${services.length} APIs)`);
}
if (topicDrift) {
  gh(['api', '-X', 'PUT', `repos/${REPO}/topics`, ...TOPICS.flatMap((t) => ['-f', `names[]=${t}`])]);
  console.log(`updated topics (${TOPICS.length})`);
}
