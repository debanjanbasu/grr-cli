#!/usr/bin/env node
// Generate CHANGELOG.md from the git history.
//
// Why generated rather than hand-maintained: a hand-written changelog drifts.
// It forgets the fix that shipped in a patch release, it invents an entry for a
// change that was reverted before tagging, and nobody notices until a user asks
// "when did streaming send land?". Git already knows the answer, so we read it
// from git and render it in Keep a Changelog 1.1.0 form
// (https://keepachangelog.com/en/1.1.0/) with SemVer version headings, newest
// first.
//
// Inputs:
//   - tags matching v?MAJOR.MINOR.PATCH(-prerelease)
//   - the commits between consecutive tags
//   - the commits after the newest tag (the [Unreleased] section)
//
// The script never writes a version. It only *recommends* one on stdout, so a
// human decides the bump; see recommendVersion().
//
// Determinism: every run over the same history produces byte-identical output.
// Timestamps are never embedded (release dates come from tagged commits, not
// from "now"), groups are emitted in a fixed order, and entries inside a group
// are sorted by (commit date, sha) rather than by whatever order git happened
// to walk the graph in. The file is therefore idempotent: re-running with no
// new commits rewrites it byte-for-byte and `git status` stays clean.
//
// NOTE: the .mjs spelling inside HEADER below is pinned on purpose — it is
// part of the committed CHANGELOG.md's human preamble, and rewording it would
// make every run rewrite the file for a comment rename. Change it only
// together with a deliberate regeneration.
//
// Usage:
//   node scripts/generate-changelog.ts              # write CHANGELOG.md
//   node scripts/generate-changelog.ts --check      # exit 1 if stale (CI)
//   node scripts/generate-changelog.ts --stdout     # print, write nothing
//   node scripts/generate-changelog.ts --force      # replace a hand-written file

import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const OUT_FILE = resolve(ROOT, 'CHANGELOG.md');
// Baked into the website so /changelog/ renders from the same model that
// produces CHANGELOG.md, instead of parsing Markdown at build time.
const SITE_JSON = resolve(ROOT, 'site', 'src', 'data', 'changelog.json');

// Keep a Changelog has no machine-readable "this region is generated" marker,
// so we bracket the machine-owned part ourselves. Everything outside the
// brackets is treated as a human preamble and preserved verbatim, which lets
// someone add a "how to read this file" note without it being clobbered.
const START_MARKER = '<!-- generated-start -->';
const END_MARKER = '<!-- generated-end -->';

const argv = new Set(process.argv.slice(2));
const CHECK_ONLY = argv.has('--check');
const STDOUT_ONLY = argv.has('--stdout');
const FORCE = argv.has('--force');

/** Uniform `message` extraction — catch params are `unknown` under strict TS. */
function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/* ------------------------------------------------------------------ shapes */

/** A SemVer tag, plus the commit date resolved in listTags(). */
interface Tag {
  ref: string;
  version: string;
  nums: [number, number, number];
  pre: string | null;
  date?: string;
}

/** One commit, as parsed out of `git log`. */
interface Commit {
  sha: string;
  date: string;
  subject: string;
  body: string;
}

/** The Conventional-Commits reading of one subject line. */
interface ParsedSubject {
  type: string | null;
  scope: string | null;
  breaking: boolean;
  description: string;
  untyped: boolean;
}

/** A parsed commit, filed into a section. */
interface Entry extends ParsedSubject {
  sha: string;
  date: string;
  footer: string | null;
}

type SectionKey =
  | 'breaking' | 'added' | 'changed' | 'performance' | 'fixed'
  | 'documentation' | 'internal' | 'reverted' | 'other';

interface Section {
  key: SectionKey;
  heading: string;
  types: string[] | null;
  collapse?: boolean;
}

type BumpLevel = 'none' | 'major' | 'minor' | 'patch';

/** The SemVer bump implied by the unreleased commits. */
interface Recommendation {
  level: BumpLevel;
  version: string | null;
  reasons: string[];
  counts: Record<string, number>;
}

/** One entry in the site JSON's per-section item list. */
interface SiteSectionEntry {
  description: string;
  scope: string | null;
  type: string | null;
  note: string | null;
}

interface SiteSection {
  key: SectionKey;
  heading: string;
  items: SiteSectionEntry[];
}

interface SiteRelease {
  version: string;
  date: string | null;
  unreleased: boolean;
  sections: SiteSection[];
}

/* ------------------------------------------------------------------ git io */

function git(args: string[]): string {
  try {
    return execFileSync('git', args, {
      cwd: ROOT,
      encoding: 'utf8',
      // Default stdio forwards git's stderr to this process's stderr, which
      // turns an expected probe failure ("no remote 'origin'") into console
      // noise. Capture it and fold it into the thrown error instead.
      stdio: ['ignore', 'pipe', 'pipe'],
      maxBuffer: 64 * 1024 * 1024,
    });
  } catch (err) {
    // child_process failures carry their captured stderr; fall back to the
    // message for anything else a bad git invocation can throw.
    const stderr = err instanceof Error && 'stderr' in err ? (err as Error & { stderr?: unknown }).stderr : undefined;
    const detail = stderr ? String(stderr).trim() : '';
    throw new Error(`git ${args.join(' ')} failed: ${detail || errorMessage(err)}`);
  }
}

function tryGit(args: string[]): { ok: boolean; out: string } {
  try {
    return { ok: true, out: git(args) };
  } catch {
    return { ok: false, out: '' };
  }
}

/** GitHub-style slug, so the compare links at the bottom of the file work. */
function repoSlug(): string | null {
  const res = tryGit(['remote', 'get-url', 'origin']);
  if (!res.ok) return null;
  const url = res.out.trim();
  const m =
    /github\.com[:/]+([^/]+)\/([^/]+?)(?:\.git)?$/.exec(url) ??
    /git@[^:]+[:/]([^/]+)\/([^/]+?)(?:\.git)?$/.exec(url);
  return m ? `${m[1]}/${m[2]}` : null;
}

/* --------------------------------------------------------- semver helpers */

// Compare two parsed versions by SemVer precedence: numeric identifiers
// compare numerically and a prerelease sorts *before* its release
// (1.0.0-rc.1 < 1.0.0), with the usual "more fields wins" tiebreak.
function compareVersions(a: Tag, b: Tag): number {
  for (let i = 0; i < 3; i += 1) {
    if (a.nums[i] !== b.nums[i]) return a.nums[i] - b.nums[i];
  }
  const ap = a.pre;
  const bp = b.pre;
  if (ap === null && bp === null) return 0;
  if (ap === null) return 1;
  if (bp === null) return -1;
  const ai = ap.split('.');
  const bi = bp.split('.');
  for (let i = 0; i < Math.max(ai.length, bi.length); i += 1) {
    if (ai[i] === undefined) return -1;
    if (bi[i] === undefined) return 1;
    const an = /^\d+$/.test(ai[i]);
    const bn = /^\d+$/.test(bi[i]);
    if (an && bn) {
      const d = Number(ai[i]) - Number(bi[i]);
      if (d !== 0) return d;
    } else if (ai[i] !== bi[i]) {
      return ai[i] < bi[i] ? -1 : 1;
    }
  }
  return 0;
}

const SEMVER_TAG = /^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?(?:\+([0-9A-Za-z.-]+))?$/;

function parseTag(ref: string): Tag | null {
  const m = SEMVER_TAG.exec(ref.trim());
  if (!m) return null;
  return {
    ref: ref.trim(),
    version: `${Number(m[1])}.${Number(m[2])}.${Number(m[3])}${m[4] ? `-${m[4]}` : ''}`,
    nums: [Number(m[1]), Number(m[2]), Number(m[3])],
    pre: m[4] ?? null,
  };
}

function bumpVersion(v: Tag, level: BumpLevel): string {
  const [maj, min, pat] = v.nums;
  if (level === 'major') return `${maj + 1}.0.0`;
  if (level === 'minor') return `${maj}.${min + 1}.0`;
  return `${maj}.${min}.${pat + 1}`;
}

/**
 * Fully-qualified tag ref. Always use this rather than the bare name: the
 * shorthand `v0.3.0` is ambiguous when a branch or a remote-tracking ref
 * shares the name, and a wrong resolution silently yields an empty range.
 */
const tagRef = (tag: Tag): string => `refs/tags/${tag.ref}`;

/** SemVer tags, oldest first. Non-SemVer tags (e.g. "nightly") are ignored. */
function listTags(): Tag[] {
  const parsed = git(['tag', '--list'])
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean)
    .map(parseTag)
    .filter((t): t is Tag => t !== null)
    .sort(compareVersions);

  // A repository that has ever tagged both `v1.0.0` and `1.0.0` would otherwise
  // get two sections with the same heading, which breaks the link references
  // and reads as a bug. Keep one tag per version, preferring the `v`-prefixed
  // spelling because that is what release.yml pushes.
  const byVersion = new Map<string, Tag>();
  for (const t of parsed) {
    const existing = byVersion.get(t.version);
    if (!existing || (!existing.ref.startsWith('v') && t.ref.startsWith('v'))) {
      if (existing) {
        console.warn(
          `warning: tags ${existing.ref} and ${t.ref} are both version ${t.version}; using ${t.ref}.`,
        );
      }
      byVersion.set(t.version, t);
    } else {
      console.warn(`warning: tags ${existing.ref} and ${t.ref} are both version ${t.version}; using ${existing.ref}.`);
    }
  }
  const tags = [...byVersion.values()].sort(compareVersions);

  for (const t of tags) {
    // `git log -1 <tagref>` peels an annotated tag to the commit it points at,
    // so the release date resolves the same for annotated and lightweight tags.
    t.date = git(['log', '-1', '--format=%cI', tagRef(t)]).trim().slice(0, 10);
  }
  return tags;
}

/* ------------------------------------------------------- commit collection */

const FIELD = '\x1f';
const RECORD = '\x1e';

/**
 * Commits in `from..to` (or every ancestor of `to` when `from` is null),
 * oldest first. Merge commits are excluded: a squash-merge repository has none,
 * and in a merge-commit repository the merge itself would duplicate every
 * change its parents already report.
 */
function commitsBetween(from: string | null, to: string): Commit[] {
  // Cheap guard: a non-string here stringifies to "[object Object]" and git
  // then reports an empty range, which looks like "this release changed
  // nothing" instead of the bug it is. (Kept at runtime on purpose — the
  // same callers can be reached from untyped entry points.)
  if (from !== null && typeof from !== 'string') {
    throw new TypeError(`commitsBetween: \`from\` must be a ref string or null, got ${typeof from}`);
  }
  if (typeof to !== 'string') {
    throw new TypeError(`commitsBetween: \`to\` must be a ref string, got ${typeof to}`);
  }
  const range = from ? `${from}..${to}` : to;
  const out = git([
    'log',
    '--reverse',
    '--no-merges',
    `--format=%H${FIELD}%cI${FIELD}%s${FIELD}%b${RECORD}`,
    range,
  ]);

  const commits: Commit[] = [];
  for (const record of out.split(RECORD)) {
    const trimmed = record.replace(/^[\r\n]+/, '');
    if (!trimmed.trim()) continue;
    const [sha, date, subject, body = ''] = trimmed.split(FIELD);
    if (!sha || !subject) continue;
    commits.push({ sha, date, subject, body });
  }

  // `git log` is deterministic for a given history, but its walk order depends
  // on the commit-graph and user config. Sorting on (date, sha) pins the order
  // to the data itself, so the output cannot depend on the machine.
  return commits.sort((a, b) => a.date.localeCompare(b.date) || a.sha.localeCompare(b.sha));
}

/* ------------------------------------------------ conventional-commit parse */

const CONVENTIONAL =
  /^(?<type>[a-zA-Z]+)(?:\((?<scope>[^)]*)\))?(?<bang>!)?:\s*(?<description>.+)$/;

// GitHub squash-merge titles get the PR number appended: "... (#42)".
const PR_SUFFIX = /\s*\(#\d+\)\s*$/;
// `revert: "feat: x" (#7)` — conventional-changelog unwraps the quoted title.
const REVERT_OF = /^["'“”](.+)["'“”]$/;

function parseSubject(subject: string): ParsedSubject {
  const line = subject.trim();
  // The PR number is metadata, not a change description.
  const withoutPr = line.replace(PR_SUFFIX, '').trim();
  const m = CONVENTIONAL.exec(withoutPr);

  if (!m?.groups) {
    // An untyped subject: still a real change, so it is reported under "Other"
    // rather than dropped.
    return {
      type: null,
      scope: null,
      breaking: false,
      description: cleanText(withoutPr),
      untyped: true,
    };
  }
  const groups = m.groups;

  const type = groups.type.toLowerCase();
  const description = cleanText(groups.description);
  const revertOf = type === 'revert' ? REVERT_OF.exec(description) : null;

  return {
    type,
    scope: groups.scope ? cleanText(groups.scope) : null,
    // `feat!:` / `refactor!:` — the exclamation mark marks a breaking change
    // under the Conventional Commits spec.
    breaking: groups.bang === '!',
    description: revertOf ? cleanText(revertOf[1]) : description,
    untyped: false,
  };
}

/** `BREAKING CHANGE:` / `BREAKING-CHANGE:` footer, per the Conventional Commits spec. */
function breakingFooter(body: string): string | null {
  const m = /^BREAKING[ -]CHANGE:\s*(.+?)(?:\r?\n(?![ \t])\S.*)*$/ms.exec(body);
  return m ? cleanText(m[1]) : null;
}

/**
 * Flatten a commit subject into one safe Markdown line: no newlines, no leading
 * list marker, no control characters. The inline-significant characters are
 * backslash-escaped — `_` in particular, because subjects here are full of
 * identifiers (`fs_io`, `io_uring`, `x86_64-linux`) and two of them on one line
 * would otherwise italicise the prose between them.
 */
function cleanText(text: string): string {
  return String(text)
    .replace(/\r\n/g, '\n')
    .split('\n')
    .map((l) => l.trim())
    .filter(Boolean)
    .join(' ')
    .replace(/\s+/g, ' ')
    // eslint-disable-next-line no-control-regex
    .replace(/[\u0000-\u001f\u007f]/g, '')
    .replace(/^([-*+>#]|\d+[.)])\s+/, '')
    .replace(/([\\`*_[\]<>|])/g, '\\$1')
    .trim();
}

/* --------------------------------------------------------------- grouping */

// Keep a Changelog's own categories, plus the ones this repository actually
// produces. Order is the order the sections are emitted in: user-visible impact
// first, repo hygiene last.
const SECTIONS: Section[] = [
  { key: 'breaking', heading: 'Breaking changes', types: null },
  { key: 'added', heading: 'Added', types: ['feat'] },
  { key: 'changed', heading: 'Changed', types: ['refactor', 'build'] },
  { key: 'performance', heading: 'Performance', types: ['perf'] },
  { key: 'fixed', heading: 'Fixed', types: ['fix'] },
  { key: 'documentation', heading: 'Documentation', types: ['docs'] },
  // chore/ci/test/style are noise in a user-facing changelog. They are counted,
  // not listed: the count is the signal ("this release touched CI"), the
  // individual commits are noise.
  { key: 'internal', heading: 'Internal', types: ['chore', 'ci', 'test', 'style'], collapse: true },
  { key: 'reverted', heading: 'Reverted', types: ['revert'] },
  { key: 'other', heading: 'Other', types: [] },
];

// Every section key comes from SECTIONS itself, so these lookups cannot miss.
const SECTION_BY_KEY: Record<SectionKey, Section> = Object.fromEntries(
  SECTIONS.map((s) => [s.key, s] as const),
) as Record<SectionKey, Section>;

function sectionForType(type: string | null): Section {
  if (type === null) return SECTION_BY_KEY.other;
  return SECTIONS.find((s) => s.types !== null && s.types.includes(type)) ?? SECTION_BY_KEY.other;
}

function groupCommits(commits: Commit[]): { groups: Map<string, Entry[]>; breaking: Entry[] } {
  const groups = new Map<string, Entry[]>(SECTIONS.map((s) => [s.key, [] as Entry[]] as const));
  const breaking: Entry[] = [];

  for (const commit of commits) {
    const parsed = parseSubject(commit.subject);
    // A breaking change can be declared two ways: the `!` marker in the header
    // or a BREAKING CHANGE: footer in the body. Either one counts.
    const footer = breakingFooter(commit.body);
    const isBreaking = parsed.breaking || footer !== null;
    // A breaking entry is *moved* to the Breaking changes section, not copied
    // into its type section as well: one line per commit means the file cannot
    // show the same change twice, and the two lists can never drift apart.
    const section = isBreaking ? SECTION_BY_KEY.breaking : sectionForType(parsed.type);

    const entry: Entry = { ...parsed, sha: commit.sha, date: commit.date, footer };
    // The key provably exists: groups is built from the same SECTIONS list
    // that defines every possible section.key.
    groups.get(section.key)!.push(entry);
    if (isBreaking) breaking.push(entry);
  }

  return { groups, breaking };
}

/* ------------------------------------------------------------- rendering */

function bullet(entry: Entry): string {
  // Keep the scope: "crates.io allows at most 5 keywords" says less than
  // "**packaging:** crates.io allows at most 5 keywords" on the first read.
  const scope = entry.scope ? `**${entry.scope}:** ` : '';
  return `- ${scope}${entry.description}`;
}

function renderSection(section: Section, entries: Entry[]): string[] {
  if (entries.length === 0) return [];
  const out = [`### ${section.heading}`, ''];

  if (section.collapse) {
    // One line instead of N: the types that landed here are listed so the count
    // is auditable without the individual subjects.
    const counts = new Map<string, number>();
    for (const e of entries) {
      const type = e.type ?? 'untyped';
      counts.set(type, (counts.get(type) ?? 0) + 1);
    }
    const breakdown = [...counts.entries()]
      .sort((a, b) => a[0].localeCompare(b[0]))
      .map(([type, n]) => `${type} ${n}`)
      .join(', ');
    out.push(`- ${entries.length} internal commit${entries.length === 1 ? '' : 's'} (${breakdown})`);
    out.push('');
    return out;
  }

  for (const entry of entries) {
    out.push(bullet(entry));
    // A BREAKING CHANGE: footer carries migration detail the title does not,
    // so it becomes a nested bullet rather than being folded into the title.
    if (entry.footer) out.push(`  - BREAKING CHANGE: ${entry.footer}`);
  }
  out.push('');
  return out;
}

function renderRelease({ heading, commits, isUnreleased }: { heading: string; commits: Commit[]; isUnreleased?: boolean }): string[] {
  const { groups, breaking } = groupCommits(commits);
  const out = [`## ${heading}`, ''];

  const nonEmpty = SECTIONS.filter((s) => (groups.get(s.key) ?? []).length > 0);
  if (nonEmpty.length === 0) {
    out.push(isUnreleased ? '_No changes yet._' : '_No user-visible changes._');
    out.push('');
    return out;
  }

  for (const section of nonEmpty) {
    // The breaking section is rendered from the breaking entries rather than
    // from groups.get('breaking'), which stays empty by construction.
    if (section.key === 'breaking') {
      out.push(`### ${section.heading}`, '');
      for (const entry of breaking) {
        out.push(bullet(entry));
        if (entry.footer) out.push(`  - BREAKING CHANGE: ${entry.footer}`);
      }
      out.push('');
      continue;
    }
    out.push(...renderSection(section, groups.get(section.key)!));
  }
  return out;
}

const HEADER = `# Changelog

All notable changes to \`grr\` are recorded in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!--
  This file is generated by scripts/generate-changelog.mjs from the commit
  history and the git tags. Do not edit it by hand - the next run overwrites
  everything between the generated-start and generated-end markers. Text outside
  those markers is preserved.

  Commits are grouped by Conventional Commits type (feat, fix, perf, docs,
  chore, ...). A commit with no conventional type is still listed, under
  "Other", because a real change should never vanish from the changelog. A
  release with a breaking change (feat!: or a BREAKING CHANGE: footer) is called
  out in a Breaking changes section and should ship as a MAJOR bump.
-->
`;

/**
 * Assemble the machine-owned region: releases newest first, then the link
 * reference definitions, then the closing marker. The region is exactly
 * START_MARKER .. END_MARKER with no trailing newline, so splicing it into an
 * existing file cannot accumulate blank lines between runs.
 */
function renderBody(tags: Tag[], slug: string | null): string {
  const lines = [START_MARKER, ''];
  const links: string[] = [];

  const newest: Tag | undefined = tags[tags.length - 1];
  const unreleased = commitsBetween(newest ? tagRef(newest) : null, 'HEAD');
  lines.push(
    ...renderRelease({
      heading: '[Unreleased]',
      commits: unreleased,
      isUnreleased: true,
    }),
  );
  if (slug) {
    links.push(
      newest
        ? `[unreleased]: ${compareUrl(slug, newest.ref, 'HEAD')}`
        : `[unreleased]: ${repoUrl(slug)}`,
    );
  }

  for (let i = tags.length - 1; i >= 0; i -= 1) {
    const tag = tags[i];
    const prev = tags[i - 1];
    const heading = `[${tag.version}] - ${tag.date}`;
    lines.push(
      ...renderRelease({ heading, commits: commitsBetween(prev ? tagRef(prev) : null, tagRef(tag)) }),
    );
    if (slug) {
      links.push(
        prev
          ? `[${tag.version}]: ${compareUrl(slug, prev.ref, tag.ref)}`
          : `[${tag.version}]: ${repoUrl(slug)}/releases/tag/${tag.ref}`,
      );
    }
  }

  // Link definitions live *inside* the region, immediately above the closing
  // marker: they are derived from the tag list, so they are machine-owned.
  if (links.length > 0) {
    lines.push(...dedupe(links), '');
  }
  lines.push(END_MARKER);
  return lines.join('\n');
}

function dedupe(list: string[]): string[] {
  return [...new Set(list)];
}

const repoUrl = (slug: string): string => `https://github.com/${slug}`;
const compareUrl = (slug: string, from: string, to: string): string => `${repoUrl(slug)}/compare/${from}...${to}`;

/* -------------------------------------------------- version recommendation */

/**
 * SemVer bump implied by the unreleased commits. Conventional Commits maps
 * types to impact: a breaking change is MAJOR, a feature is MINOR, a fix or a
 * performance change is PATCH. Nothing recognisable means no release needed.
 */
function recommendVersion(commits: Commit[], currentTag: Tag | null): Recommendation {
  if (commits.length === 0) {
    return { level: 'none', version: null, reasons: [], counts: {} };
  }

  // Parse once: the classification, the counts and the breaking-change test
  // all need the same parse of the same subject.
  const parsed = commits.map((c) => {
    const p = parseSubject(c.subject);
    return {
      type: p.type,
      untyped: p.untyped,
      breaking: p.breaking || breakingFooter(c.body) !== null,
    };
  });

  const TYPES = ['feat', 'fix', 'perf', 'refactor', 'build', 'docs', 'chore', 'ci', 'test', 'revert'];
  const counts: Record<string, number> = {};
  for (const type of TYPES) {
    const n = parsed.filter((p) => p.type === type).length;
    if (n > 0) counts[type] = n;
  }
  const untyped = parsed.filter((p) => p.untyped).length;
  if (untyped > 0) counts.untyped = untyped;

  const breaking = parsed.filter((p) => p.breaking).length;
  const plural = (n: number, noun: string): string => `${n} ${noun}${n === 1 ? '' : 's'}`;
  // `counts` only holds types that actually occurred, so every read needs a
  // default: `undefined + 1` is NaN, and `NaN > 0` is false.
  const count = (type: string): number => counts[type] ?? 0;

  const reasons: string[] = [];
  let level: BumpLevel = 'none';
  if (breaking > 0) {
    level = 'major';
    reasons.push(plural(breaking, 'breaking change'));
  } else if (count('feat') > 0) {
    level = 'minor';
  } else if (count('fix') + count('perf') > 0) {
    level = 'patch';
  }

  if (level === 'minor') reasons.push(plural(count('feat'), 'feat commit'));
  if (level === 'patch') {
    const kinds = [count('fix') ? `${count('fix')} fix` : null, count('perf') ? `${count('perf')} perf` : null]
      .filter(Boolean)
      .join(' + ');
    reasons.push(`${kinds} (${plural(count('fix') + count('perf'), 'commit')})`);
  }
  if (level === 'none') reasons.push('no feat, fix, perf or breaking change');

  return {
    level,
    version: currentTag && level !== 'none' ? bumpVersion(currentTag, level) : null,
    reasons,
    counts,
  };
}

function printRecommendation(tags: Tag[], unreleased: Commit[]): void {
  const newest: Tag | null = tags[tags.length - 1] ?? null;
  const rec = recommendVersion(unreleased, newest);

  console.log('');
  console.log('Next version recommendation');
  console.log(`  newest tag : ${newest ? `${newest.ref} (${newest.date})` : 'none'}`);
  console.log(`  unreleased : ${unreleased.length} commit${unreleased.length === 1 ? '' : 's'}`);
  if (rec.level === 'none') {
    console.log('  suggested  : no release needed (nothing user-visible is pending)');
  } else {
    const from = newest ? `v${newest.version}` : '0.0.0';
    console.log(`  suggested  : ${from} -> v${rec.version}  (${rec.level.toUpperCase()})`);
    console.log(`  because    : ${rec.reasons.join('; ')}`);
  }
  // Pre-1.0 caveat, so nobody is surprised when a breaking change on 0.x yields
  // 1.0.0 and the lockstep consumers have to move.
  if (rec.level === 'major' && newest && newest.nums[0] === 0) {
    console.log('  note       : 0.x is pre-1.0, so a MAJOR bump here lands on 1.0.0 -');
    console.log('               confirm that is intended before tagging.');
  }

  // Stable key=value lines for the workflow step summary / future automation.
  console.log('');
  console.log(`changelog:recommended-version=${rec.version ? `v${rec.version}` : ''}`);
  console.log(`changelog:recommended-bump=${rec.level}`);
  console.log(`changelog:newest-tag=${newest ? newest.ref : ''}`);
  console.log(`changelog:unreleased-count=${unreleased.length}`);
}

/* ----------------------------------------------------------- file writing */

/**
 * Which line ending a file predominantly uses. A Windows checkout with
 * core.autocrlf=true materialises CRLF even though the blob is LF, so the
 * generated content has to be written back in whatever the file on disk
 * already uses. Without this, every run on Windows would rewrite all 120 lines
 * and the idempotency guarantee would only hold on Linux CI.
 */
function detectEol(text: string): string {
  const crlf = (text.match(/\r\n/g) ?? []).length;
  const lf = (text.match(/\n/g) ?? []).length - crlf;
  return crlf > lf ? '\r\n' : '\n';
}

function writeOrCheck(content: string): void {
  const onDisk = existsSync(OUT_FILE) ? readFileSync(OUT_FILE, 'utf8') : null;
  const eol = onDisk ? detectEol(onDisk) : '\n';
  // Compare and splice in LF only; the file's own ending is restored on write.
  const current = onDisk === null ? null : onDisk.replace(/\r\n/g, '\n');
  const hasMarkers = current !== null && current.includes(START_MARKER) && current.includes(END_MARKER);

  if (current !== null && !hasMarkers && !FORCE && !STDOUT_ONLY) {
    // Refuse to destroy a hand-written changelog: it may contain unreleased
    // notes no commit message ever recorded.
    console.error(
      `refusing to overwrite ${OUT_FILE}: it has no ${START_MARKER} / ${END_MARKER} markers.`,
    );
    console.error('Move it aside, or re-run with --force to replace it.');
    process.exitCode = 1;
    return;
  }

  // Preserve the hand-written parts: keep everything up to and including the
  // start marker, replace the region, and keep anything below the end marker.
  let next: string;
  if (hasMarkers) {
    // `content` always *starts* with the marker, so its body begins at a fixed
    // offset. Deriving this from the existing file's offsets instead would
    // silently truncate the region by the length of the header.
    const body = content.slice(START_MARKER.length);
    const before = current!.slice(0, current!.indexOf(START_MARKER) + START_MARKER.length);
    const after = current!.slice(current!.indexOf(END_MARKER) + END_MARKER.length);
    next = `${before}${body}${after}`;
    // Normalise the tail so re-running cannot accumulate trailing newlines.
    if (after.trim() === '') {
      next = `${next.replace(/\s+$/, '')}\n`;
    } else if (!after.startsWith('\n')) {
      next = `${next}\n${after}`;
    }
  } else {
    next = `${HEADER}\n${content}\n`;
  }

  // Guard the splice: a region that is not marker-delimited would be written
  // into the file and then mis-spliced on every later run.
  if (!content.startsWith(START_MARKER) || !content.endsWith(END_MARKER)) {
    throw new Error('internal error: generated region is not marker-delimited');
  }

  if (STDOUT_ONLY) {
    // Always LF on stdout: it is consumed by a pipe or a diff, not edited.
    process.stdout.write(next);
    return;
  }

  if (current === next) {
    console.log(`${OUT_FILE} is already up to date.`);
    return;
  }

  if (CHECK_ONLY) {
    console.error(`${OUT_FILE} is out of date. Run: node scripts/generate-changelog.ts`);
    console.error('');
    console.error(printableDiff(current ?? '', next));
    process.exitCode = 1;
    return;
  }

  const bytes = eol === '\r\n' ? next.replace(/\n/g, '\r\n') : next;
  writeFileSync(OUT_FILE, bytes, 'utf8');
  console.log(`wrote ${OUT_FILE}${eol === '\r\n' ? ' (CRLF, matching the existing file)' : ''}`);
}

/**
 * Minimal line diff: trim the shared prefix and suffix, then show what is left.
 * Enough to see why the check failed without pulling in a dependency or
 * shelling out to `git diff --no-index` (which is not available everywhere).
 */
function printableDiff(before: string, after: string): string {
  const a = before.split('\n');
  const b = after.split('\n');
  let start = 0;
  while (start < a.length && start < b.length && a[start] === b[start]) start += 1;
  let end = 0;
  while (
    end < a.length - start &&
    end < b.length - start &&
    a[a.length - 1 - end] === b[b.length - 1 - end]
  ) {
    end += 1;
  }
  const removed = a.slice(start, a.length - end);
  const added = b.slice(start, b.length - end);
  const fmt = (prefix: string, lines: string[]): string[] => lines.map((l) => `${prefix}${l}`);
  const body = [
    ...fmt('-', removed.slice(0, 40)),
    ...(removed.length > 40 ? [`- ... ${removed.length - 40} more`] : []),
    ...fmt('+', added.slice(0, 40)),
    ...(added.length > 40 ? [`+ ... ${added.length - 40} more`] : []),
  ];
  return body.join('\n');
}

/* -------------------------------------------------------------------- main */

// Exported so the parsing rules can be exercised directly (a squash-merge title
// with a (#NN) suffix, a BREAKING CHANGE: footer, a prerelease tag) without
// standing up a git repository for each case.
export {
  HEADER,
  SECTIONS,
  START_MARKER,
  END_MARKER,
  breakingFooter,
  bumpVersion,
  cleanText,
  compareVersions,
  groupCommits,
  parseSubject,
  parseTag,
  recommendVersion,
  renderBody,
};

function main(): void {
  const tags = listTags();

  if (tags.length === 0) {
    // A brand-new repository, or one that has not cut a release yet. Not an
    // error: exit 0 and leave whatever CHANGELOG.md exists alone.
    console.log('No SemVer tags found (looked for v?MAJOR.MINOR.PATCH), so there is nothing to group yet.');
    console.log('Create and push a v0.1.0 tag, then re-run this script.');
    return;
  }

  const newest = tags[tags.length - 1];
  const unreleased = commitsBetween(tagRef(newest), 'HEAD');
  const slug = repoSlug();
  if (!slug) {
    console.warn('warning: no GitHub remote found; compare links will be omitted.');
  }

  const content = renderBody(tags, slug);
  writeOrCheck(content);
  writeSiteJson(tags, slug);
  printRecommendation(tags, unreleased);
}

/**
 * Emit the same history as structured JSON for the website.
 *
 * The changelog page renders this instead of parsing Markdown at build time:
 * one generator, one model, and the site can never disagree with
 * CHANGELOG.md. Written next to the other baked site data so `npm run build`
 * picks it up with no extra fetch.
 */
function writeSiteJson(tags: Tag[], slug: string | null): void {
  const newest: Tag | undefined = tags[tags.length - 1];
  const releases: SiteRelease[] = [];

  const push = (version: string, date: string | null, commits: Commit[], unreleased: boolean): void => {
    const { groups, breaking } = groupCommits(commits);
    const sections: SiteSection[] = [];
    for (const section of SECTIONS) {
      const entries = section.key === 'breaking' ? breaking : groups.get(section.key);
      if (!entries || entries.length === 0) continue;
      sections.push({
        key: section.key,
        heading: section.heading,
        items: entries.map((entry) => ({
          description: entry.description,
          scope: entry.scope ?? null,
          type: entry.type,
          // Only carry a footer through when it adds something the title
          // does not already say.
          note: entry.footer ?? null,
        })),
      });
    }
    releases.push({ version, date, unreleased, sections });
  };

  push('Unreleased', null, commitsBetween(newest ? tagRef(newest) : null, 'HEAD'), true);
  for (let i = tags.length - 1; i >= 0; i -= 1) {
    const tag = tags[i];
    const prev = tags[i - 1];
    push(
      tag.version,
      tag.date ?? null,
      commitsBetween(prev ? tagRef(prev) : null, tagRef(tag)),
      false,
    );
  }

  const payload = {
    generatedAt: new Date().toISOString(),
    repository: slug ?? null,
    releases,
  };
  const json = `${JSON.stringify(payload, null, 2)}\n`;

  if (CHECK_ONLY) {
    // `generatedAt` is a wall-clock stamp, so it can never byte-match a
    // committed file. Compare the CONTENT: strip the stamp from both sides
    // and require everything else to be identical. Without this the
    // release-time verify job would be permanently red.
    if (existsSync(SITE_JSON)) {
      const stripStamp = (value: unknown): string => {
        const record = { ...(value as Record<string, unknown>) };
        delete record.generatedAt;
        return JSON.stringify(record);
      };
      if (
        stripStamp(JSON.parse(readFileSync(SITE_JSON, 'utf8'))) ===
        stripStamp(JSON.parse(json))
      ) {
        return;
      }
    }
    console.error('site changelog JSON is stale; run: node scripts/generate-changelog.ts');
    process.exitCode = 1;
    return;
  }
  // Same stamp rule: rewriting unconditionally would change the file on
  // every run and open a PR that carries nothing but a fresh timestamp.
  if (existsSync(SITE_JSON)) {
    const stripStamp = (value: unknown): string => {
      const record = { ...(value as Record<string, unknown>) };
      delete record.generatedAt;
      return JSON.stringify(record);
    };
    if (
      stripStamp(JSON.parse(readFileSync(SITE_JSON, 'utf8'))) ===
      stripStamp(JSON.parse(json))
    ) {
      return;
    }
  }
  mkdirSync(dirname(SITE_JSON), { recursive: true });
  writeFileSync(SITE_JSON, json, 'utf8');
}

// Only generate when executed directly. `import`ing this module (to test the
// parsing rules) must not rewrite CHANGELOG.md as a side effect.
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main();
}
