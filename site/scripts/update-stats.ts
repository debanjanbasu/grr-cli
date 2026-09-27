// Refresh site/src/data/stats.json from crates.io and GitHub Releases.
//
// The stats page renders committed JSON, never live API responses, so the
// numbers are reproducible and the site builds offline. Every field is read
// defensively: an unexpected API shape skips the refresh and keeps the
// existing file rather than writing a half-populated snapshot.

import { readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(scriptDirectory, '..', '..');
const cargoManifestPath = resolve(repositoryRoot, 'Cargo.toml');
const statsPath = resolve(repositoryRoot, 'site', 'src', 'data', 'stats.json');
const userAgentFor = (version: string): string => `grr-cli-site-stats/${version} (+https://github.com/debanjanbasu/grr-cli)`;

/** crates.io /payload, trimmed to the fields read below. External JSON. */
interface CratesIoCrate {
  downloads?: unknown;
  recent_downloads?: unknown;
}

interface CratesIoResponse {
  crate?: CratesIoCrate;
}

/** One GitHub release, defensively typed: only the fields stats read. */
interface GithubRelease {
  draft?: unknown;
  published_at?: string;
  created_at?: string;
  tag_name?: string;
  assets?: Array<ReleaseAsset>;
}

interface ReleaseAsset {
  name?: string;
  size?: unknown;
  download_count?: unknown;
}

interface SiteStats {
  version: string;
  crateDownloads: number;
  crateDownloads30d: number;
  githubDownloads: number;
  latestRelease: string;
  latestReleaseDate: string;
  binarySizes: Record<string, number> | null;
  updatedAt: string;
}

/** Hand-rolled guard: a JSON object (arrays excluded â€” a release row is one). */
const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value);

const readPackageVersion = async (): Promise<string> => {
  const manifest = await readFile(cargoManifestPath, 'utf8');
  const packageSection = manifest.match(/\[package\]([\s\S]*?)(?:\n\[|$)/)?.[1] ?? '';
  const version = packageSection.match(/^\s*version\s*=\s*"([^"]+)"/m)?.[1];
  if (!version) {
    throw new Error(`package version not found in ${cargoManifestPath}`);
  }
  return version;
};

// `T` documents the shape each call site expects; the runtime validation is
// the defensive narrowing at the call sites (isRecord / Array.isArray / the
// optional chaining below), exactly where the .mjs did its checking.
const fetchJson = async <T>(url: string, userAgent: string, headers: Record<string, string> = {}): Promise<T> => {
  const response = await fetch(url, {
    headers: {
      Accept: 'application/json',
      'User-Agent': userAgent,
      ...headers,
    },
    signal: AbortSignal.timeout(15000),
  });
  if (!response.ok) {
    throw new Error(`${url} returned HTTP ${response.status}`);
  }
  return (await response.json()) as T;
};

const asCount = (value: unknown): number => {
  const count = Number(value);
  return Number.isFinite(count) && count >= 0 ? Math.trunc(count) : 0;
};

// Release assets are named `grr-v<version>-<platform>.<ext>`, e.g.
// `grr-v0.4.0-macos-aarch64.tar.zst`. GitHub reports the download size in
// `asset.size` (bytes); `asset.size_in_bytes` does not exist on this endpoint.
// Any asset that is not a platform archive (SHA256SUMS, provenance, ...) is
// skipped, and a platform the release does not ship is simply absent.
const archiveExtensions = ['tar\\.zst', 'tar\\.gz', 'tar\\.xz', 'tar\\.bz2', 'tar', 'tgz', 'zip', 'gz', 'xz', 'bz2', '7z', 'exe', 'msi', 'pkg', 'dmg'];
const assetPattern = new RegExp(`^grr-v\\d+(?:\\.\\d+)*-(?<platform>[a-z0-9_]+(?:-[a-z0-9_]+)*)\\.(${archiveExtensions.join('|')})$`, 'i');

const readBinarySizes = (release: GithubRelease | undefined): Record<string, number> | null => {
  const sizes: Record<string, number> = {};
  const assets = release && Array.isArray(release.assets) ? release.assets : [];
  for (const asset of assets) {
    const platform = asset?.name?.match(assetPattern)?.groups?.platform?.toLowerCase();
    if (!platform || platform === 'source') continue;
    const bytes = asCount(asset?.size);
    if (bytes > 0) sizes[platform] = bytes;
  }
  return Object.keys(sizes).length > 0 ? sizes : null;
};

try {
  const version = await readPackageVersion();
  const userAgent = userAgentFor(version);
  const [cratePayload, releasePayload] = await Promise.all([
    fetchJson<CratesIoResponse>('https://crates.io/api/v1/crates/grr-cli', userAgent),
    fetchJson<unknown>('https://api.github.com/repos/debanjanbasu/grr-cli/releases', userAgent, {
      Accept: 'application/vnd.github+json',
      'X-GitHub-Api-Version': '2022-11-28',
    }),
  ]);

  const crate = cratePayload?.crate;
  if (!crate || !Array.isArray(releasePayload)) {
    throw new Error('unexpected API response shape');
  }

  const releases = (releasePayload as unknown[])
    .filter((release): release is GithubRelease => isRecord(release) && release.draft !== true)
    .sort((left, right) => {
      const leftDate = Date.parse(left.published_at ?? left.created_at ?? '') || 0;
      const rightDate = Date.parse(right.published_at ?? right.created_at ?? '') || 0;
      return rightDate - leftDate;
    });
  const latestRelease = releases[0];
  const githubDownloads = releases.reduce(
    (total, release) => total + (Array.isArray(release.assets) ? release.assets.reduce((sum, asset) => sum + asCount(asset?.download_count), 0) : 0),
    0,
  );

  const stats: SiteStats = {
    version,
    crateDownloads: asCount(crate.downloads),
    crateDownloads30d: asCount(crate.recent_downloads),
    githubDownloads,
    latestRelease: latestRelease?.tag_name ?? '',
    latestReleaseDate: latestRelease?.published_at ?? latestRelease?.created_at ?? '',
    binarySizes: readBinarySizes(latestRelease),
    updatedAt: new Date().toISOString(),
  };

  await writeFile(statsPath, `${JSON.stringify(stats, null, 2)}\n`, 'utf8');
  const sizeSummary = stats.binarySizes
    ? Object.entries(stats.binarySizes)
        .map(([platform, bytes]) => `${platform}=${(bytes / 1024 / 1024).toFixed(1)}MB`)
        .join(' ')
    : 'no platform assets found';
  console.log(`Updated ${statsPath}: ${stats.crateDownloads} crates.io downloads, ${stats.githubDownloads} GitHub asset downloads, ${sizeSummary}`);
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  console.warn(`Stats refresh skipped: ${message}`);
  console.warn(`Keeping the existing ${statsPath}`);
}
