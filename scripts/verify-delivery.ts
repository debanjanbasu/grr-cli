#!/usr/bin/env node
// Verify the DELIVERED results, not just the repository: the crate on
// crates.io, the docs.rs build, the release asset set, the Homebrew formula,
// the live site's content — and the freshness of the generated data the site
// renders. Every check here corresponds to a real, silent failure this repo
// has already lived through: docs.rs failed 5/5 builds for weeks and nothing
// noticed; the winget submission looped forever; a release could exist with
// no crate, or a tag with no release, and only a human reading dashboards
// would know.
//
// Why a script and not inline YAML: the checks need JSON parsing and a
// grace-window policy (right after a release, the formula/site/crates docs
// legitimately lag for a while), and this file is testable and readable.
//
// Usage: node scripts/verify-delivery.ts [--json]
//
// Policy: a mismatch is a warning (not a failure) while the newest release is
// younger than GRACE_HOURS — deploy pipelines are asynchronous, and a check
// that flaps after every release gets ignored. Past the window, a mismatch is
// a failure: something is stuck.
//
// Exit: 1 when any check failed, 0 otherwise (warnings never fail).

import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';

const REPO = 'debanjanbasu/grr-cli';
const SITE = 'https://grr-cli.pages.dev';
const TAP_FORMULA = 'https://raw.githubusercontent.com/debanjanbasu/homebrew-tap/main/Formula/grr.rb';
const GRACE_HOURS = 12;
const STATS_MAX_AGE_DAYS = 7;

const EXPECTED_ASSETS = [
  'SHA256SUMS',
  'grr-demo.gif',
];
const ARCHIVE_SUFFIXES = ['linux-x86_64.tar.zst', 'linux-aarch64.tar.zst', 'macos-aarch64.tar.zst', 'windows-x86_64.zip', 'windows-aarch64.zip'];

interface Check {
  name: string;
  status: 'ok' | 'warn' | 'fail';
  detail: string;
}

const checks: Check[] = [];

function record(name: string, status: Check['status'], detail: string): void {
  checks.push({ name, status, detail });
}

async function fetchText(url: string): Promise<string | null> {
  try {
    const response = await fetch(url, {
      redirect: 'follow',
      // crates.io 403s requests without a User-Agent (verified: 403 bare, 200
      // with one); the other endpoints tolerate it, so one header serves all.
      headers: { 'cache-control': 'no-cache', 'user-agent': 'grr-verify-delivery (https://github.com/debanjanbasu/grr-cli)' },
    });
    return response.ok ? await response.text() : null;
  } catch {
    return null;
  }
}

function ghJson(args: string[]): unknown {
  const result = spawnSync('gh', args, { encoding: 'utf8' });
  if (result.status !== 0) return null;
  try {
    return JSON.parse(result.stdout);
  } catch {
    return null;
  }
}

/** A member of a parsed JSON object, or undefined — narrows instead of casting. */
function member(value: unknown, key: string): unknown {
  return value && typeof value === 'object' && key in value ? (value as Record<string, unknown>)[key] : undefined;
}

/** True while a fresh release may still be propagating through the pipes. */
function withinGrace(publishedAt: string | null): boolean {
  if (publishedAt === null) return false;
  const age = Date.now() - Date.parse(publishedAt);
  return age >= 0 && age < GRACE_HOURS * 3600 * 1000;
}

const version = /^version = "(.+)"$/m.exec(readFileSync('Cargo.toml', 'utf8'))?.[1];
if (version === undefined) {
  console.error('verify-delivery: no version in Cargo.toml');
  process.exit(1);
}
const tag = `v${version}`;

// The release, fetched first: its age drives the grace policy for the
// channels that necessarily lag behind it.
const release = ghJson(['api', `repos/${REPO}/releases/tags/${tag}`]);
const assets: string[] = (() => {
  const list = member(release, 'assets');
  return Array.isArray(list) ? list.flatMap((a) => (typeof member(a, 'name') === 'string' ? [member(a, 'name') as string] : [])) : [];
})();
const publishedAt = member(release, 'published_at');
const grace = withinGrace(typeof publishedAt === 'string' ? publishedAt : null);

if (release === null) {
  record(`release ${tag}`, 'fail', `no GitHub release for ${tag} (Cargo.toml is ahead of the release pipeline)`);
} else {
  const missing = [...ARCHIVE_SUFFIXES.map((s) => `grr-${tag}-${s}`), ...EXPECTED_ASSETS].filter((name) => !assets.includes(name));
  record(`release ${tag} assets`, missing.length === 0 ? 'ok' : 'fail', missing.length === 0 ? `${assets.length} assets present` : `missing: ${missing.join(', ')}`);
}

// crates.io: the published max stable version must be the manifest's.
const cratesBody = await fetchText('https://crates.io/api/v1/crates/grr-cli');
let published: string | null = null;
if (cratesBody !== null) {
  try {
    const crate = member(JSON.parse(cratesBody), 'crate');
    const versionField = member(crate, 'max_stable_version');
    published = typeof versionField === 'string' ? versionField : null;
  } catch {
    published = null;
  }
}
if (published === null) {
  record('crates.io', 'fail', 'crates.io API unreachable or the response shape changed');
} else if (published === version) {
  record('crates.io', 'ok', `${published} published`);
} else {
  record('crates.io', grace ? 'warn' : 'fail', `crates.io has ${published}, Cargo.toml is ${version}${grace ? ' (within the release grace window)' : ''}`);
}

// docs.rs: the newest build of THIS version must not have failed. A missing
// or in-progress build is a warning inside the grace window, a failure after.
const buildsPage = await fetchText(`https://docs.rs/crate/grr-cli/${version}/builds`);
const buildId = buildsPage === null ? null : /builds\/(\d+)/.exec(buildsPage)?.[1] ?? null;
if (buildId === null) {
  record('docs.rs', grace ? 'warn' : 'fail', `no build found for ${version}${grace ? ' yet (within the release grace window)' : ''}`);
} else {
  const buildPage = await fetchText(`https://docs.rs/crate/grr-cli/${version}/builds/${buildId}`);
  if (buildPage === null) {
    record('docs.rs', grace ? 'warn' : 'fail', `build ${buildId} page unreadable`);
  } else if (/Build failed/.test(buildPage)) {
    record('docs.rs', 'fail', `build ${buildId} of ${version} FAILED — the docs.rs path is broken again`);
  } else if (/build log|Docs\.rs/.test(buildPage)) {
    record('docs.rs', 'ok', `build ${buildId} green`);
  } else {
    record('docs.rs', 'warn', `build ${buildId} has no verdict yet`);
  }
}

// Homebrew: the tap's formula must point at this version.
const formula = await fetchText(TAP_FORMULA);
const formulaVersion = formula === null ? null : /version "(.+)"/.exec(formula)?.[1] ?? null;
if (formulaVersion === null) {
  record('homebrew tap', 'warn', 'formula unreadable');
} else if (formulaVersion === version) {
  record('homebrew tap', 'ok', `formula ${formulaVersion}`);
} else {
  record('homebrew tap', grace ? 'warn' : 'fail', `formula is ${formulaVersion}, latest is ${version}${grace ? ' (within the release grace window)' : ' — tap updated within ~6h, still stuck is a failure'}`);
}

// The live site: the homepage must carry the current version (it is rendered
// from stats.json, which refresh.yml regenerates), the blog must exist (a
// deploy that dropped a page is silent otherwise), and the build-sha marker
// (BaseLayout, from Cloudflare's CF_PAGES_COMMIT_SHA) tells us whether the
// deploy tracks main. Sha drift is always a warning: deploys lag pushes.
const home = await fetchText(`${SITE}/`);
if (home === null) {
  record('live site', 'fail', `${SITE}/ unreachable`);
} else {
  if (home.includes(`v${version}`) || home.includes(version)) {
    record('live site version', 'ok', `homepage carries ${version}`);
  } else {
    const shown = /v\d+\.\d+\.\d+/.exec(home)?.[0] ?? 'none';
    record('live site version', grace ? 'warn' : 'fail', `homepage shows ${shown}, latest is ${version}${grace ? ' (within the release grace window)' : ''}`);
  }
  const liveSha = /<meta name="build-sha" content="([0-9a-f]{7,40})"/.exec(home)?.[1] ?? null;
  const headSha = spawnSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).stdout?.trim() ?? '';
  if (liveSha === null) {
    record('live site build', 'warn', 'no build-sha marker (deploy predates the marker, or is not from git)');
  } else if (liveSha.startsWith(headSha.slice(0, 7)) || headSha.startsWith(liveSha.slice(0, 7))) {
    record('live site build', 'ok', `deployed ${liveSha.slice(0, 7)} == HEAD`);
  } else {
    record('live site build', 'warn', `deployed ${liveSha.slice(0, 7)} != HEAD ${headSha.slice(0, 7)} (deploys lag pushes; check Cloudflare if this persists)`);
  }
  const blog = await fetchText(`${SITE}/blog/`);
  record('live site blog', blog === null ? 'fail' : 'ok', blog === null ? '/blog/ is not reachable' : 'present');
}

// Generated data the site renders: an unbounded-stale stats.json was a real
// gap (the fetch could fail silently and the workflow stayed green).
const stats = JSON.parse(readFileSync('site/src/data/stats.json', 'utf8')) as unknown;
const updatedAt = member(stats, 'updatedAt');
if (typeof updatedAt !== 'string') {
  record('stats.json age', 'fail', 'no updatedAt');
} else {
  const ageDays = (Date.now() - Date.parse(updatedAt)) / 86_400_000;
  record('stats.json age', ageDays <= STATS_MAX_AGE_DAYS ? 'ok' : 'fail', `updated ${ageDays.toFixed(1)} days ago (limit ${STATS_MAX_AGE_DAYS})`);
}

const failed = checks.filter((c) => c.status === 'fail');
if (process.argv.includes('--json')) {
  console.log(JSON.stringify({ version, checks }, null, 2));
} else {
  for (const c of checks) {
    const mark = c.status === 'ok' ? '✓' : c.status === 'warn' ? '·' : '✗';
    console.log(`${mark} ${c.name}: ${c.detail}`);
  }
  console.log(`\n${checks.length - failed.length}/${checks.length} ok${failed.length > 0 ? `, ${failed.length} FAILED` : ''}`);
}
for (const c of failed) {
  console.log(`::error::${c.name}: ${c.detail}`);
}
process.exit(failed.length > 0 ? 1 : 0);
