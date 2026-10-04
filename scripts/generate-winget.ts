#!/usr/bin/env node
// Generate the winget submission for a release: the three manifest files
// winget-pkgs expects under manifests/d/debanjanbasu/grr/<version>/.
//
// Why generate: the installer manifest must name the executable that is
// INSIDE the zip and the zip's SHA-256 — both properties of the shipped
// artifact, not facts anyone should retype. The first hand-written
// submission promised `grr.exe` while the archive contained
// `windows-x86_64.exe`, so winget's validation could never find the nested
// installer and looped on "issue with installing the application correctly"
// (winget-pkgs PR #441361). This script downloads the real assets, reads the
// member list, and derives every field from what actually shipped.
//
// Usage: node scripts/generate-winget.ts v0.8.1 [--out <dir>]
//
// Deterministic: no wall-clock timestamps. ReleaseDate comes from the
// release's own publishedAt (the API is the only source allowed to supply
// it); if the API cannot be reached the line is omitted, never guessed.

import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';

const REPO = 'debanjanbasu/grr-cli';
const IDENTIFIER = 'debanjanbasu.grr';
const MANIFEST_VERSION = '1.6.0';

const WINDOWS_TARGETS = [
  { asset: 'windows-x86_64', architecture: 'x64' },
  { asset: 'windows-aarch64', architecture: 'arm64' },
];

function fail(message: string): never {
  console.error(`generate-winget: ${message}`);
  process.exit(1);
}

function parseArgs(argv: string[]): { tag: string; out: string } {
  const positional: string[] = [];
  let out = 'packaging/winget/winget-pkgs';
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === '--out') {
      out = argv[i + 1] ?? fail('--out needs a directory');
      i += 1;
    } else {
      positional.push(argv[i]!);
    }
  }
  const raw = positional[0] ?? fail('usage: generate-winget.ts <version-tag> [--out <dir>]');
  const tag = raw.startsWith('v') ? raw : `v${raw}`;
  if (!/^v[0-9]+\.[0-9]+\.[0-9]+$/.test(tag)) {
    fail(`'${raw}' is not a release tag (expected vX.Y.Z)`);
  }
  return { tag, out };
}

const { tag, out } = parseArgs(process.argv.slice(2));
const version = tag.slice(1);

async function fetchText(url: string): Promise<string | null> {
  const response = await fetch(url, { redirect: 'follow' });
  return response.ok ? await response.text() : null;
}

/** filename -> lowercase sha256, from the release's own SHA256SUMS. */
async function fetchSums(): Promise<Map<string, string>> {
  const text = await fetchText(`https://github.com/${REPO}/releases/download/${tag}/SHA256SUMS`);
  if (text === null) fail(`no SHA256SUMS on release ${tag} — is the release published?`);
  const sums = new Map<string, string>();
  for (const line of text.split('\n')) {
    const match = /^([0-9a-f]{64})\s+(\S+)$/.exec(line.trim());
    if (match) sums.set(match[2]!, match[1]!);
  }
  if (sums.size === 0) fail(`SHA256SUMS on ${tag} contains no entries`);
  return sums;
}

/** The release's publication date (YYYY-MM-DD), or null when unavailable. */
async function fetchReleaseDate(): Promise<string | null> {
  const body = await fetchText(`https://api.github.com/repos/${REPO}/releases/tags/${tag}`);
  if (body === null) return null;
  const parsed: unknown = JSON.parse(body);
  if (parsed && typeof parsed === 'object' && 'published_at' in parsed && typeof parsed.published_at === 'string') {
    return parsed.published_at.slice(0, 10);
  }
  return null;
}

const CACHE = 'target/winget-cache';

function sha256(path: string): string {
  return createHash('sha256').update(readFileSync(path)).digest('hex');
}

async function download(asset: string, sha: string): Promise<string> {
  const path = join(CACHE, asset);
  const url = `https://github.com/${REPO}/releases/download/${tag}/${asset}`;
  if (existsSync(path) && sha256(path) === sha) return path;
  mkdirSync(dirname(path), { recursive: true });
  const response = await fetch(url, { redirect: 'follow' });
  if (!response.ok) fail(`download failed (${response.status}) for ${url}`);
  writeFileSync(path, Buffer.from(await response.arrayBuffer()));
  if (sha256(path) !== sha) fail(`${asset}: sha256 does not match the release's ${sha}`);
  return path;
}

/** The members of a zip, via Info-ZIP's quiet listing. */
function zipMembers(path: string): string[] {
  const result = spawnSync('unzip', ['-Z1', path], { encoding: 'utf8' });
  if (result.status !== 0) fail(`unzip -Z1 failed for ${path}: ${result.stderr}`);
  return (result.stdout ?? '').split('\n').filter((line) => line.length > 0);
}

interface Installer {
  architecture: string;
  sha: string;
  member: string;
  asset: string;
}

function buildManifests(installers: Installer[], releaseDate: string | null): Record<string, string> {
  const head = `PackageIdentifier: ${IDENTIFIER}\nPackageVersion: ${version}\n`;

  const entries = installers
    .slice()
    .sort((a, b) => (a.architecture < b.architecture ? -1 : a.architecture > b.architecture ? 1 : 0))
    .map(
      (installer) =>
        `  - Architecture: ${installer.architecture}\n` +
        `    InstallerType: zip\n` +
        `    InstallerUrl: https://github.com/${REPO}/releases/download/${tag}/grr-${tag}-${installer.asset}.zip\n` +
        `    InstallerSha256: ${installer.sha}\n` +
        `    NestedInstallerType: portable\n` +
        `    NestedInstallerFiles:\n` +
        `      - RelativeFilePath: ${installer.member}\n` +
        `        PortableCommandAlias: grr\n` +
        (releaseDate ? `    ReleaseDate: ${releaseDate}\n` : ''),
    )
    .join('');

  return {
    [`${IDENTIFIER}.yaml`]: `${head}DefaultLocale: en-US\nManifestType: version\nManifestVersion: ${MANIFEST_VERSION}\n`,
    [`${IDENTIFIER}.installer.yaml`]:
      `${head}InstallerType: zip\nInstallers:\n${entries}ManifestType: installer\nManifestVersion: ${MANIFEST_VERSION}\n`,
    [`${IDENTIFIER}.locale.en-US.yaml`]:
      `${head}PackageLocale: en-US\n` +
      `Publisher: Debanjan Basu\n` +
      `PublisherUrl: https://github.com/debanjanbasu\n` +
      `PublisherSupportUrl: https://github.com/${REPO}/issues\n` +
      `PackageName: grr\n` +
      `Moniker: grr\n` +
      `PackageUrl: https://github.com/${REPO}\n` +
      `License: MIT\n` +
      `LicenseUrl: https://github.com/${REPO}/blob/main/LICENSE\n` +
      `ShortDescription: Google tools from the terminal, at maximum performance\n` +
      `Description: The Google tools used every day — Gmail, Calendar, Drive, Docs, Sheets and ten more Google APIs — behind one OAuth login, in a single zero-config Rust binary.\n` +
      `Tags:\n` +
      `  - cli\n` +
      `  - google\n` +
      `  - gmail\n` +
      `  - calendar\n` +
      `  - terminal\n` +
      `ManifestType: defaultLocale\n` +
      `ManifestVersion: ${MANIFEST_VERSION}\n`,
  };
}

const sums = await fetchSums();
const releaseDate = await fetchReleaseDate();
const installers: Installer[] = [];

for (const { asset, architecture } of WINDOWS_TARGETS) {
  const filename = `grr-${tag}-${asset}.zip`;
  const sha = sums.get(filename);
  if (sha === undefined) fail(`${filename} is not in SHA256SUMS`);
  const zip = await download(filename, sha);
  const executables = zipMembers(zip).filter((member) => member.toLowerCase().endsWith('.exe'));
  if (executables.length !== 1) {
    fail(`${filename} contains ${executables.length} executables; expected exactly one nested installer`);
  }
  installers.push({ architecture, sha: sha.toUpperCase(), member: executables[0]!, asset });
  console.log(`  ${architecture}: ${filename} -> ${executables[0]} (sha ${sha.slice(0, 12)}…)`);
}

mkdirSync(out, { recursive: true });
for (const [name, body] of Object.entries(buildManifests(installers, releaseDate))) {
  writeFileSync(join(out, name), body);
  console.log(`wrote ${join(out, name)}`);
}

console.log(`\nwinget submission ready: copy ${out}/*.yaml into a fork of`);
console.log(`  microsoft/winget-pkgs under manifests/d/debanjanbasu/grr/${version}/`);
