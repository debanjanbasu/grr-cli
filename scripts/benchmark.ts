#!/usr/bin/env node
// Daily, credential-free benchmark of grr against the other Google Workspace
// CLIs, baked into the site as site/src/data/benchmarks.json.
//
// Why this exists: a comparison page is only worth anything if its numbers
// are measured, dated, and reproducible. Everything here runs without Google
// credentials — startup, the `grr schema` contract dump, the `grr api list`
// index and binary size are all properties of the binaries themselves, so a
// CI run can collect them and open a PR with the fresh snapshot.
//
// Startup is the headline metric (median of 15 timed runs); the schema dump,
// the api-index listing, binary size and peak RSS are measured alongside it.
//
// Size is recorded twice for grr, because the two numbers are different
// artifacts and the compare page must not confuse them:
//   - `binary.bytes` is the locally built (unpacked) executable that the
//     timings run, recorded with its build kind;
//   - `binary.shipped` is the binary inside the latest release archive — what
//     a user actually downloads. Since v0.7.0 the release workflow UPX-packs
//     that binary (--best --lzma), so it is materially smaller than the local
//     build. UPX is detected from the packer's own magic, never assumed, and
//     a release asset that cannot be fetched records nothing (the page then
//     falls back to the unpacked figure with its label).
// Competitors are only ever measured as shipped (their official release
// builds), so the page compares like with like.
//
// Peak RSS is measured with the platform's `time` binary around each tool's
// schema-dump command — the closest equivalent workload every tool has, and a
// real one; `--version` is too trivial to separate the tools. It is the
// number that competes with local models and agents sharing unified memory.
//
// Honesty rules, enforced structurally:
//   - A tool that cannot be obtained is recorded as `available: false` with
//     the reason. No number is ever invented, interpolated or remembered from
//     a previous run.
//   - A command that fails is recorded as unavailable with its exit status,
//     not skipped silently.
//   - A competitor metric with no equivalent command is recorded as
//     `available: false` with the reason, never omitted — so "not measured"
//     stays distinguishable from "measured and absent".
//
// Protocol: 1 warm-up run is discarded per metric (cold filesystem/cache
// effects are real but are not the steady-state cost agents pay), then 15
// timed runs; median/p95/min are reported. Timing uses
// process.hrtime.bigint() around spawnSync, so it is wall-clock monotonic
// per run and never depends on Date.now() jitter.
//
// Idempotency: raw timings drift a few ms between runs, so byte-identity is
// not defined as "identical raw numbers". A fresh run whose medians land
// within the noise band (max(15 ms, 6%) per metric) of the committed
// snapshot — with an identical environment, tool availability and versions —
// rewrites NOTHING, keeping `git status` quiet and the daily workflow
// PR-free until something actually moved. Outside the band the file is
// rewritten with a new generatedAt.
//
// Version tracking: the snapshot records, for grr and every competitor, the
// version string that actually ran (`local`) and the project's latest GitHub
// release tag (`latestTag`). The daily workflow compares these against
// GitHub BEFORE building anything, so a re-measurement only happens when a
// new version of grr, gog or gws actually shipped.
//
// Usage:
//   node scripts/benchmark.ts            # write site/src/data/benchmarks.json
//   node scripts/benchmark.ts --check    # exit 1 if the committed file is stale
//   node scripts/benchmark.ts --gate     # print `skip=`/`reason=` for the workflow's version gate

import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';

const ROOT = resolve(import.meta.dirname, '..');
const OUT_FILE = resolve(ROOT, 'site', 'src', 'data', 'benchmarks.json');
const CACHE_DIR = resolve(ROOT, 'target', 'benchmark-cache');

const GRR_PROJECT = 'debanjanbasu/grr-cli';

const RUNS = 15;
const WARMUP_RUNS = 1;
const COMMAND_TIMEOUT_MS = 60_000;

const CHECK_ONLY = process.argv.includes('--check');
const GATE_ONLY = process.argv.includes('--gate');

/** Uniform `message` extraction — catch params are `unknown` under strict TS. */
function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

// The noise band that separates "same measurement, different morning" from
// "the number actually changed". Absolute floor first because sub-20ms
// startups can jitter by more than 6% of almost nothing.
const NOISE_ABS_MS = 15;
const NOISE_REL = 0.06;

// Peak RSS is noisier than a median: one run's high-water mark moves with the
// allocator, page cache and scheduler. The band that separates "same
// measurement" from "the footprint actually moved" is therefore wider and
// absolute-first (2 MB floor, then 10%).
const MEMORY_ABS_KB = 2048;
const MEMORY_REL = 0.1;

// The measurement surface for peak RSS. GNU time (Linux) reports `-v` in
// kbytes; BSD time (macOS) reports `-l` in bytes; neither exists on Windows,
// where the metric records unavailable rather than an estimate.
const TIME_BIN = '/usr/bin/time';
const TIME_FLAG = process.platform === 'darwin' ? '-l' : '-v';
const TIME_UNITS = process.platform === 'darwin' ? 'bytes' : 'kbytes';
const MEMORY_WORKLOAD =
  "each tool's schema-dump command (grr schema / gog schema / gws schema drive.files.list --resolve-refs) — not --version, which is too trivial a workload to separate the tools";

const withinBand = (a: number, b: number, abs: number, rel: number): boolean =>
  Math.abs(a - b) <= Math.max(abs, rel * Math.max(a, b));
const withinNoise = (a: number, b: number): boolean => withinBand(a, b, NOISE_ABS_MS, NOISE_REL);

const round1 = (value: number): number => Math.round(value * 10) / 10;
const firstLine = (text: string): string => text.split('\n')[0].trim();

/* ------------------------------------------------------------- snapshot shapes */

/**
 * One timed metric. Either measured (`available: true` with the three
 * statistics) or honestly unavailable (`available: false` with the reason).
 */
type Metric =
  | { available: true; medianMs: number; p95Ms: number; minMs: number }
  | { available: false; reason: string };

/**
 * Peak resident set size for one tool's schema-dump command, read from the
 * platform's `time` binary (GNU `-v` on Linux, BSD `-l` on macOS). Either
 * measured (`available: true` with the high-water mark across the runs) or
 * honestly unavailable with the reason — never estimated. `command` records
 * exactly which equivalent workload was measured.
 */
interface MemoryMetric {
  available: boolean;
  maxRssKb?: number;
  runs?: number;
  command?: string;
  reason?: string;
}

/**
 * The artifact a user actually installs: the binary inside grr's latest
 * release archive. `packed` is a measurement of the packer's magic, not an
 * assumption about the tag — the release workflow ships unpacked when UPX
 * fails or the packed binary does not run.
 */
interface ShippedBinary {
  bytes: number;
  source: string;
  form: string;
  packed: boolean;
  releaseTag?: string;
  asset?: string;
}

/**
 * Binary size entry. Success shape: `{ bytes, source, form }`. Failure
 * shape: `{ available: false, reason }` — recorded, never estimated. The
 * optional-flat form mirrors the JSON exactly, where only one of the two
 * shapes is ever present. `shipped` is the release artifact, present only
 * when it could actually be downloaded and measured.
 */
interface BinarySize {
  bytes?: number;
  source?: string;
  form?: string;
  available?: false;
  reason?: string;
  shipped?: ShippedBinary;
}

/** A competitor row in the snapshot — available, or unavailable with why. */
interface CompetitorRow {
  tool: string;
  project: string;
  available: boolean;
  reason?: string;        // when available === false
  obtainedVia?: string;   // when available === true
  version?: string;       // when available === true
  // Per-metric records. Every metric grr measures has a key here for every
  // competitor: measured, or `available: false` with the reason (no
  // equivalent command / tool unreachable) — never omitted.
  startup?: Metric;
  schemaDump?: Metric;
  apiList?: Metric;
  memory?: MemoryMetric; // peak RSS for the schema-dump command
  binary?: BinarySize;   // when available === true
}

/** Version tracking block: what ran, and what was latest at measurement time. */
interface VersionEntry {
  local: string | null;
  latestTag?: string | null;
  latestTagReason?: string;
}

type VersionsBlock = Record<string, VersionEntry>;

interface BenchmarkEnvironment {
  platform: string;
  arch: string;
  nodeVersion: string;
  binaryKind?: string;
  binaryPath?: string;
  grrVersion?: string;
}

interface BenchmarkSnapshot {
  generatedAt: string;
  environment: BenchmarkEnvironment;
  versions: VersionsBlock;
  protocol: {
    runs: number;
    warmupRunsDiscarded: number;
    timer: string;
    stability: string;
    memoryTimer: string;
    memoryWorkload: string;
  };
  grr: {
    startup: Metric | null;
    schemaDump: Metric | null;
    apiList: Metric | null;
    memory: MemoryMetric | null;
    binary: BinarySize | null;
  };
  competitors: CompetitorRow[];
}

/** Latest-release payload from the GitHub REST API, trimmed to what is read. */
interface GithubRelease {
  tag_name?: string;
  assets?: GithubReleaseAsset[];
}

interface GithubReleaseAsset {
  name: string;
  size: number;
  browser_download_url: string;
}

// ---------------------------------------------------------------------------
// Timing
// ---------------------------------------------------------------------------

function percentile(sorted: number[], q: number): number {
  const idx = Math.min(sorted.length - 1, Math.ceil(q * sorted.length) - 1);
  return sorted[idx];
}

/**
 * Run `exe args...` WARMUP_RUNS + RUNS times, report median/p95/min in ms.
 * Returns `{ available: false, reason }` if any run fails or times out — the
 * failure of one sample must invalidate the metric, not be averaged away.
 */
// `label` is what a failure reason cites — a repo-relative command, never the
// absolute path of the machine that happened to run it (the snapshot is
// published on the website).
function measure(exe: string, args: string[], label?: string): Metric {
  const cited = label ?? [exe, ...args].join(' ');
  const samples: number[] = [];
  for (let i = 0; i < WARMUP_RUNS + RUNS; i++) {
    const t0 = process.hrtime.bigint();
    const res = spawnSync(exe, args, { stdio: 'ignore', timeout: COMMAND_TIMEOUT_MS, windowsHide: true });
    const t1 = process.hrtime.bigint();
    if (res.error) {
      return { available: false, reason: `spawn failed: ${res.error.message}` };
    }
    if (res.status !== 0) {
      const how = res.signal ? `signal ${res.signal}` : `exit status ${res.status}`;
      return { available: false, reason: `${how} for \`${cited}\`` };
    }
    if (i < WARMUP_RUNS) continue; // warm-up, discarded on purpose
    samples.push(Number(t1 - t0) / 1e6);
  }
  samples.sort((a, b) => a - b);
  return {
    available: true,
    medianMs: round1(samples[Math.floor(samples.length / 2)]),
    p95Ms: round1(percentile(samples, 0.95)),
    minMs: round1(samples[0]),
  };
}

// ---------------------------------------------------------------------------
// Peak memory
// ---------------------------------------------------------------------------

/**
 * Pull the high-water mark out of one `time` report. GNU time -v prints
 * `Maximum resident set size (kbytes): 12345`; BSD time -l prints
 * `\t 12345678  maximum resident set size` with the value in bytes. The
 * report goes to stderr in both cases.
 */
function parseMaxRss(platform: NodeJS.Platform, report: string): number | null {
  const match = platform === 'darwin'
    ? /^\s*(\d+)\s+maximum resident set size\s*$/im.exec(report)
    : /Maximum resident set size \(kbytes\):\s*(\d+)/i.exec(report);
  if (!match) return null;
  const value = Number(match[1]);
  if (!Number.isFinite(value) || value <= 0) return null;
  return platform === 'darwin' ? Math.round(value / 1024) : value;
}

/**
 * Peak RSS for one command: the same warm-up + timed-runs protocol as a
 * timing, but reporting the **max of the per-run peaks** (RSS is a single
 * high-water mark per run, so a median would describe an average run rather
 * than the worst-case footprint a user should expect). Measured under the
 * platform's `time` binary; if that binary is missing (Windows), the command
 * fails, or the report cannot be parsed, the metric is recorded unavailable
 * with the reason — never estimated.
 *
 * This is a second pass over the schema-dump workload rather than a byproduct
 * of `measure()`: `measure()` discards all output for timing fidelity, and
 * timing and memory want different statistics (median vs max), so folding
 * them together would conflate two protocols to save a few seconds.
 */
function measurePeakRss(exe: string, args: string[], label: string): MemoryMetric {
  if (process.platform === 'win32') {
    return {
      available: false,
      command: label,
      reason: `peak RSS is measured with ${TIME_BIN}, which does not exist on Windows — not estimated`,
    };
  }
  let maxRssKb = 0;
  for (let i = 0; i < WARMUP_RUNS + RUNS; i++) {
    // Only stderr is piped: `time` writes its report there, while the tool's
    // own stdout (a full schema dump) is discarded so it can never trip the
    // spawn buffer.
    const res = spawnSync(TIME_BIN, [TIME_FLAG, exe, ...args], {
      encoding: 'utf8',
      stdio: ['ignore', 'ignore', 'pipe'],
      timeout: COMMAND_TIMEOUT_MS,
      windowsHide: true,
    });
    if (res.error) return { available: false, command: label, reason: `spawn failed: ${res.error.message}` };
    if (res.status !== 0) {
      const how = res.signal ? `signal ${res.signal}` : `exit status ${res.status}`;
      return { available: false, command: label, reason: `${how} for \`${TIME_BIN} ${TIME_FLAG} ${label}\`` };
    }
    const rss = parseMaxRss(process.platform, `${res.stderr ?? ''}`);
    if (rss === null) {
      return {
        available: false,
        command: label,
        reason: `could not parse "maximum resident set size" from \`${TIME_BIN} ${TIME_FLAG}\` report`,
      };
    }
    if (i < WARMUP_RUNS) continue; // warm-up, discarded on purpose
    maxRssKb = Math.max(maxRssKb, rss);
  }
  return { available: true, maxRssKb, runs: RUNS, command: label };
}

// ---------------------------------------------------------------------------
// Running a tool and capturing its self-reported output
// ---------------------------------------------------------------------------

type CaptureResult = { ok: false; reason: string } | { ok: true; text: string };

const runCapture = (exe: string, args: string[]): CaptureResult => {
  const res = spawnSync(exe, args, { encoding: 'utf8', timeout: COMMAND_TIMEOUT_MS, windowsHide: true });
  if (res.error) return { ok: false, reason: `spawn failed: ${res.error.message}` };
  if (res.status !== 0) {
    const how = res.signal ? `signal ${res.signal}` : `exit status ${res.status}`;
    return { ok: false, reason: how };
  }
  return { ok: true, text: `${res.stdout ?? ''}`.trim() };
};

// ---------------------------------------------------------------------------
// Obtaining competitor binaries (PATH first, then GitHub release download)
// ---------------------------------------------------------------------------

async function githubJson(url: string): Promise<GithubRelease> {
  const res = await fetch(url, {
    headers: { 'user-agent': 'grr-cli-benchmark', accept: 'application/vnd.github+json' },
    redirect: 'follow',
  });
  if (!res.ok) throw new Error(`GET ${url} -> HTTP ${res.status}`);
  // External JSON; the readers below only touch the spec-mandated fields,
  // with `?? []` / typeof fallbacks exactly where the .mjs had them.
  return (await res.json()) as GithubRelease;
}

// Latest published release tag of a project, or the reason it could not be
// fetched. Recorded for grr and every tracked competitor so the daily
// workflow can decide — before building anything — whether a re-measurement
// is due (`--gate`).
async function latestReleaseTag(project: string): Promise<Pick<VersionEntry, 'latestTag' | 'latestTagReason'>> {
  try {
    const release = await githubJson(`https://api.github.com/repos/${project}/releases/latest`);
    return { latestTag: typeof release.tag_name === 'string' ? release.tag_name : null };
  } catch (error) {
    return { latestTag: null, latestTagReason: errorMessage(error) };
  }
}

function walkFiles(dir: string): string[] {
  const out: string[] = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) out.push(...walkFiles(path));
    else out.push(path);
  }
  return out;
}

// `which`/`where` rather than a bare spawn: Windows spawn does not resolve
// PATH + the .exe extension for a bare name, and a resolved absolute path
// also lets us stat the binary for its on-disk size.
function resolveOnPath(name: string): string | null {
  const probe = process.platform === 'win32'
    ? spawnSync('where.exe', [name], { encoding: 'utf8', windowsHide: true })
    : spawnSync('which', [name], { encoding: 'utf8' });
  if (probe.status !== 0 || !probe.stdout) return null;
  const path = firstLine(probe.stdout);
  return path && existsSync(path) ? path : null;
}

function extractArchive(archive: string, dir: string): void {
  // System tar, not a Node dependency: bsdtar ships with Windows 10+ and
  // handles .zip as well as .tar.gz; Linux runners have GNU tar.
  mkdirSync(dir, { recursive: true });
  const res = spawnSync('tar', ['-xf', archive, '-C', dir], { encoding: 'utf8', windowsHide: true });
  if (res.status === 0) return;
  // grr's release archives are .tar.zst. GNU tar shells out to `zstd` for
  // those, so a tar built without zstd support (or a machine without the
  // zstd binary) fails where gzip/bzip2 would not. Decompress through an
  // explicit pipe before giving up — .zst never appears on Windows, whose
  // grr assets are .zip.
  if (archive.endsWith('.zst') && process.platform !== 'win32') {
    const piped = spawnSync('sh', ['-c', 'zstd -dc "$1" | tar -xf - -C "$2"', 'sh', archive, dir], {
      encoding: 'utf8',
    });
    if (piped.status === 0) return;
    throw new Error(`zstd | tar -xf failed: ${(piped.stderr ?? '').trim() || `exit ${piped.status}`}`);
  }
  throw new Error(`tar -xf failed: ${(res.stderr ?? '').trim() || `exit ${res.status}`}`);
}

function findExecutable(dir: string, name: string): string | null {
  const lower = name.toLowerCase();
  const suffix = process.platform === 'win32' ? '.exe' : '';
  const hits = walkFiles(dir).filter((p) => p.toLowerCase().endsWith(`${lower}${suffix}`));
  return hits.length > 0 ? hits[0] : null;
}

/** A competitor binary that could be located. */
interface ObtainSuccess {
  exe: string;
  via: 'path' | 'github-release-download';
  releaseTag?: string;
  asset?: string;
  /**
   * The real binary behind a launcher shim, when one was resolved. `exe`
   * stays the launcher for timing (that is what a user invoking the command
   * pays), but the binary-size metric follows this path.
   */
  payload?: string;
}

type Obtained = ObtainSuccess | { error: string };

/**
 * The npm-installed `gws` on PATH is an ~850-byte launcher: it downloads the
 * platform binary on first run and then execs it. Timing the launcher is
 * honest (that is what a user invoking `gws` pays), but its stat size is the
 * shim's — the compare page once showed gws at "0.0 MB" because of exactly
 * this. Call this only AFTER the launcher has run once (the smoke test does),
 * so the payload exists.
 */
function resolveGwsPayload(): string | null {
  if (process.platform === 'win32' || !gwsTriple) return null;
  const root = spawnSync('npm', ['root', '-g'], { encoding: 'utf8', timeout: COMMAND_TIMEOUT_MS, windowsHide: true });
  const globalRoot = root.status === 0 ? (root.stdout ?? '').trim() : '';
  if (!globalRoot) return null;
  // The package layout, after the launcher's first run: bin/<triple>/gws.
  const candidate = join(globalRoot, '@googleworkspace/cli', 'bin', gwsTriple, 'gws');
  return existsSync(candidate) ? candidate : null;
}

/**
 * Download one release asset into the cache and return its path. The cached
 * copy is validated by size, so a truncated download from a previous run is
 * refetched rather than extracted. Throws on failure; callers decide whether
 * that is fatal (competitors: recorded unavailable) or a skip (grr's shipped
 * size: recorded not at all).
 */
async function cachedArchive(tool: string, tag: string | null | undefined, asset: GithubReleaseAsset): Promise<string> {
  const archive = join(CACHE_DIR, `${tool}-${tag ?? 'unknown'}-${asset.name}`);
  if (!existsSync(archive) || statSync(archive).size !== asset.size) {
    mkdirSync(CACHE_DIR, { recursive: true });
    const res = await fetch(asset.browser_download_url, { redirect: 'follow' });
    if (!res.ok) throw new Error(`download ${asset.browser_download_url} -> HTTP ${res.status}`);
    writeFileSync(archive, Buffer.from(await res.arrayBuffer()));
  }
  return archive;
}

/**
 * Locate a competitor binary: PATH first (the benchmark workflow pre-installs
 * the tools), then the latest GitHub release asset for this platform.
 * Archives are cached under target/ (gitignored) so re-runs are cheap and
 * identical.
 *
 * A PATH hit is not trusted blindly: the daily workflow has pre-installed a
 * tool with the wrong architecture before (an amd64 ELF placed on an arm64
 * runner), and spawnSync reported the fallout only later, as every metric's
 * `--version` failing with exit status 2. The PATH branch therefore
 * smoke-tests the binary with `--version` and falls back to the
 * arch-matching release download when it cannot run.
 */
async function obtainCompetitor({ tool, project, exeName, assetTokens }: CompetitorSpec): Promise<Obtained> {
  const onPath = resolveOnPath(exeName);
  if (onPath) {
    const smoke = spawnSync(onPath, ['--version'], { encoding: 'utf8', timeout: COMMAND_TIMEOUT_MS, windowsHide: true });
    if (!smoke.error && smoke.status === 0) {
      // gws ships as an npm launcher; the smoke test just ran it, which is
      // what triggers its one-time payload download — so the real binary is
      // resolvable now (see resolveGwsPayload for why size follows it).
      const payload = tool === 'gws' ? resolveGwsPayload() : null;
      return { exe: onPath, via: 'path', ...(payload ? { payload } : {}) };
    }
    // Wrong-arch or otherwise unrunnable: on Linux, libuv falls back to
    // /bin/sh for a spawn that returns ENOEXEC, and the shell exits 2
    // mis-parsing the ELF — so record the smoke failure but keep going:
    // the release download below is the recovery path.
    const how = smoke.error
      ? `spawn failed: ${smoke.error.message}`
      : smoke.signal
        ? `signal ${smoke.signal}`
        : `exit status ${smoke.status}`;
    process.stderr.write(
      `bench: ${exeName} on PATH at ${onPath} did not pass the --version smoke test (${how}); trying the release download instead\n`,
    );
  }

  try {
    const release = await githubJson(`https://api.github.com/repos/${project}/releases/latest`);
    const asset = (release.assets ?? []).find(
      (a) => assetTokens.every((t) => a.name.includes(t)) && /\.(zip|tar\.gz|tgz)$/.test(a.name),
    );
    if (!asset) {
      throw new Error(
        `no release asset matching [${assetTokens.join(', ')}]; assets: ${(release.assets ?? []).map((a) => a.name).join(', ')}`,
      );
    }

    const archive = await cachedArchive(tool, release.tag_name, asset);

    const extractDir = join(CACHE_DIR, tool, release.tag_name ?? 'unknown');
    extractArchive(archive, extractDir);
    const exe = findExecutable(extractDir, exeName);
    if (!exe) throw new Error(`no ${exeName} binary inside ${asset.name}`);

    return { exe, via: 'github-release-download', releaseTag: release.tag_name, asset: asset.name };
  } catch (error) {
    return { error: errorMessage(error) };
  }
}

// ---------------------------------------------------------------------------
// Platform mapping for release assets
// ---------------------------------------------------------------------------

const PLATFORM_OS_TOKENS: Record<string, string> = { win32: 'windows', linux: 'linux', darwin: 'darwin' };
const OS_TOKEN = PLATFORM_OS_TOKENS[process.platform] ?? process.platform;

const GOG_ARCH_TOKENS: Record<string, string> = { x64: 'amd64', arm64: 'arm64' };
const gogArchToken = GOG_ARCH_TOKENS[process.arch] ?? process.arch;

const GWS_TRIPLES: Record<string, Record<string, string>> = {
  win32: { x64: 'x86_64-pc-windows-msvc', arm64: 'aarch64-pc-windows-msvc' },
  linux: { x64: 'x86_64-unknown-linux-gnu', arm64: 'aarch64-unknown-linux-gnu' },
  darwin: { x64: 'x86_64-apple-darwin', arm64: 'aarch64-apple-darwin' },
};
const gwsTriple = GWS_TRIPLES[process.platform]?.[process.arch] ?? '';

// grr's own release archives (packaging/release.yml): used only when neither
// a release nor a debug build exists locally.
const GRR_ASSET_TOKENS: Record<string, string[]> = {
  win32: ['windows', process.arch === 'arm64' ? 'aarch64' : 'x86_64'],
  linux: ['linux', process.arch === 'arm64' ? 'aarch64' : 'x86_64'],
  darwin: ['macos', process.arch === 'arm64' ? 'aarch64' : 'x86_64'],
};
const grrAssetTokens = GRR_ASSET_TOKENS[process.platform] ?? ['linux', 'x86_64'];

interface CompetitorSpec {
  tool: string;
  project: string;
  url: string;
  exeName: string;
  assetTokens: string[];
  /**
   * The credential-free command whose timing is the equivalent of a grr
   * metric, or the reason no equivalent exists. Verified against each
   * tool's release binary: gog's `schema` and `api list` both run
   * credential-free, but `api list` fetches Google's live Discovery
   * directory over the network, so it is not comparable to `grr api list`'s
   * offline index. gws's `schema <service.resource.method>` answers one
   * method's contract offline, but it has no index command at all.
   */
  schemaCommand?: string[] | null;
  apiListCommand?: string[] | null;
  apiListReason?: string;
}

const COMPETITORS: CompetitorSpec[] = [
  {
    tool: 'gog',
    project: 'openclaw/gogcli',
    url: 'https://github.com/openclaw/gogcli',
    exeName: 'gog',
    assetTokens: [OS_TOKEN, gogArchToken],
    schemaCommand: ['schema'],
    apiListCommand: null,
    apiListReason: 'has an `api list` command, but it fetches Google\'s live Discovery directory over the network — not comparable to grr\'s offline index',
  },
  {
    tool: 'gws',
    project: 'googleworkspace/cli',
    url: 'https://github.com/googleworkspace/cli',
    exeName: 'gws',
    assetTokens: gwsTriple ? [gwsTriple] : [],
    schemaCommand: ['schema', 'drive.files.list', '--resolve-refs'],
    apiListCommand: null,
    apiListReason: 'no api-index equivalent command (`gws api` is unknown-service only)',
  },
];

// ---------------------------------------------------------------------------
// grr itself
// ---------------------------------------------------------------------------

function locateGrr(): { path: string; kind: 'release' | 'debug' } | null {
  const exe = process.platform === 'win32' ? 'grr.exe' : 'grr';
  // Release before debug: the release profile is what ships and what the
  // daily workflow builds; a debug binary is a developer's fallback.
  for (const kind of ['release', 'debug'] as const) {
    const path = resolve(ROOT, 'target', kind, exe);
    if (existsSync(path)) {
      return { path, kind };
    }
  }
  return null;
}

async function grrReleaseAssetBytes(): Promise<BinarySize> {
  // Fallback when no local build exists: the published release archive.
  // GitHub's REST API exposes the size on asset.size — size_in_bytes is null
  // for release assets and has caused real bugs in this repo before.
  const release = await githubJson('https://api.github.com/repos/debanjanbasu/grr-cli/releases/latest');
  const asset = (release.assets ?? []).find(
    (a) => grrAssetTokens.every((t) => a.name.includes(t)) && /\.(zip|tar\.gz|tar\.zst)$/.test(a.name),
  );
  if (!asset) {
    throw new Error(
      `no release asset matching [${grrAssetTokens.join(', ')}]; assets: ${(release.assets ?? []).map((a) => a.name).join(', ')}`,
    );
  }
  return { bytes: asset.size, source: `github-release-asset: ${asset.name}`, form: 'release-archive' };
}

const GRR_ARCHIVE_RE = /\.(zip|tar\.gz|tar\.zst)$/;

/**
 * Locate grr's binary in an extracted release archive. Releases up to
 * v0.8.0 tars the binary under its target name (`linux-aarch64`,
 * `macos-aarch64`, `windows-x86_64.exe`); v0.8.1+ ships it as `grr` /
 * `grr.exe` — the target-named member broke winget's nested-installer
 * lookup and every human had to rename on extraction. Both layouts are
 * accepted here, so the collector reads old and new releases alike.
 */
function findGrrStagedBinary(dir: string): string | null {
  const [os] = grrAssetTokens;
  const named = join(dir, `${os}-${process.arch === 'arm64' ? 'aarch64' : 'x86_64'}${process.platform === 'win32' ? '.exe' : ''}`);
  if (existsSync(named) && statSync(named).isFile()) return named;
  const byName = findExecutable(dir, 'grr');
  if (byName) return byName;
  const files = walkFiles(dir).filter((path) => statSync(path).isFile());
  return files.length === 1 ? files[0] : null;
}

/**
 * The binary a user actually installs: the release artifact for this platform,
 * downloaded and unpacked from the cached archive. Recorded alongside the
 * locally built size so the compare page can lead with what ships.
 *
 * Best-effort by design: no released tag yet, offline, an archive tar cannot
 * read — each records nothing at all, and the page falls back to the unpacked
 * build under its own explicit label. A failed download must never be able to
 * look like a measured shipped size.
 */
async function grrShippedBinary(): Promise<ShippedBinary | null> {
  try {
    const release = await githubJson(`https://api.github.com/repos/${GRR_PROJECT}/releases/latest`);
    const asset = (release.assets ?? []).find(
      (a) => grrAssetTokens.every((t) => a.name.includes(t)) && GRR_ARCHIVE_RE.test(a.name),
    );
    if (!asset) return null;

    const archive = await cachedArchive('grr', release.tag_name, asset);

    const extractDir = join(CACHE_DIR, 'grr', release.tag_name ?? 'unknown');
    extractArchive(archive, extractDir);
    const exe = findGrrStagedBinary(extractDir);
    if (!exe) return null;

    // UPX's own magic decides the label. release-binaries.yml falls back to
    // shipping unpacked when packing fails (a win64 packer segfault, a packed
    // binary that will not run), so "packed" is a property of this artifact,
    // not of the tag.
    const packed = readFileSync(exe).includes('UPX!');
    return {
      bytes: statSync(exe).size,
      source: packed ? 'release archive (UPX-packed)' : 'release archive',
      form: 'binary',
      packed,
      ...(typeof release.tag_name === 'string' ? { releaseTag: release.tag_name } : {}),
      asset: asset.name,
    };
  } catch {
    return null;
  }
}

// ---------------------------------------------------------------------------
// Snapshot assembly
// ---------------------------------------------------------------------------

const toPosixPath = (path: string): string => relative(ROOT, path).split('\\').join('/');

async function buildSnapshot(): Promise<BenchmarkSnapshot> {
  const environment: BenchmarkEnvironment = {
    platform: process.platform,
    arch: process.arch,
    nodeVersion: process.version,
  };

  const grr: BenchmarkSnapshot['grr'] = { startup: null, schemaDump: null, apiList: null, memory: null, binary: null };
  const binary = locateGrr();

  if (binary) {
    const binaryPath = toPosixPath(binary.path);
    environment.binaryKind = binary.kind;
    environment.binaryPath = binaryPath;
    const version = runCapture(binary.path, ['--version']);
    if (version.ok) environment.grrVersion = firstLine(version.text);

    grr.startup = measure(binary.path, ['--version'], 'grr --version');
    grr.schemaDump = measure(binary.path, ['schema'], 'grr schema');
    grr.apiList = measure(binary.path, ['api', 'list'], 'grr api list');
    // Peak RSS on the schema dump, not --version: the dump is the closest
    // equivalent of the workload every tool is asked to run below.
    grr.memory = measurePeakRss(binary.path, ['schema'], 'grr schema');
    // The timings/peak RSS above run the locally built (unpacked) binary; the
    // size below additionally records the release artifact users download.
    const shipped = await grrShippedBinary();
    grr.binary = {
      bytes: statSync(binary.path).size,
      source: binaryPath,
      form: 'binary',
      ...(shipped ? { shipped } : {}),
    };
  } else {
    environment.binaryKind = 'none';
    const why = 'no local grr binary (looked for target/release and target/debug)';
    grr.startup = { available: false, reason: why };
    grr.schemaDump = { available: false, reason: why };
    grr.apiList = { available: false, reason: why };
    grr.memory = { available: false, command: 'grr schema', reason: why };
    const shipped = await grrShippedBinary();
    try {
      const archive = await grrReleaseAssetBytes();
      grr.binary = shipped ? { ...archive, shipped } : archive;
    } catch (error) {
      grr.binary = shipped
        ? { bytes: shipped.bytes, source: shipped.source, form: 'release-archive', shipped }
        : { available: false, reason: errorMessage(error) };
    }
  }

  // Version tracking: `local` is the version string the measurement actually
  // ran (null when a tool could not run at all); `latestTag` is the project's
  // latest GitHub release tag at measurement time. The workflow's gate reads
  // this block back and compares it with a fresh tag fetch.
  const versions: VersionsBlock = {
    grr: {
      local: environment.grrVersion ?? null,
      ...(await latestReleaseTag(GRR_PROJECT)),
    },
  };

  const competitors: CompetitorRow[] = [];
  for (const competitor of COMPETITORS) {
    const row = { tool: competitor.tool, project: competitor.project, url: competitor.url };
    const obtained = await obtainCompetitor(competitor);

    versions[competitor.tool] = { local: null, ...(await latestReleaseTag(competitor.project)) };

    // Every metric key is present in every row, always. When the tool could
    // not run at all, each metric records the same root cause — a missing
    // key must never be able to masquerade as "measured and absent".
    const unavailable = (reason: string) => ({
      startup: { available: false, reason } as Metric,
      schemaDump: { available: false, reason } as Metric,
      apiList: { available: false, reason } as Metric,
      memory: { available: false, reason } as MemoryMetric,
    });

    if ('error' in obtained) {
      competitors.push({ ...row, available: false, ...unavailable(obtained.error) });
      continue;
    }

    // Trust the binary's own `--version` line over any release metadata: it
    // is what the page cites, and it is what actually ran.
    const version = runCapture(obtained.exe, ['--version']);
    if (!version.ok) {
      const reason = `obtained via ${obtained.via} but \`--version\` failed: ${version.reason}`;
      competitors.push({ ...row, available: false, ...unavailable(reason) });
      continue;
    }

    versions[competitor.tool].local = firstLine(version.text);

    // Per-metric measurement. A metric with a known equivalent command is
    // measured with the identical warm-up + timed-runs protocol; a metric
    // with no equivalent (or one that is not comparable, like a command
    // that needs the network where grr's is offline) is recorded
    // unavailable with that reason. Nothing is inferred.
    const startup = measure(obtained.exe, ['--version'], `${competitor.exeName} --version`);
    const schemaDump = competitor.schemaCommand
      ? measure(obtained.exe, competitor.schemaCommand, `${competitor.exeName} ${competitor.schemaCommand.join(' ')}`)
      : { available: false, reason: 'no schema-dump equivalent command' } satisfies Metric;
    const apiList = competitor.apiListCommand
      ? measure(obtained.exe, competitor.apiListCommand, `${competitor.exeName} ${competitor.apiListCommand.join(' ')}`)
      : { available: false, reason: competitor.apiListReason ?? 'no api-index equivalent command' } satisfies Metric;
    // Peak RSS on the same schema-dump command that was just timed, so the
    // memory figure describes a workload the tool actually offers.
    const memory = competitor.schemaCommand
      ? measurePeakRss(obtained.exe, competitor.schemaCommand, `${competitor.exeName} ${competitor.schemaCommand.join(' ')}`)
      : { available: false, reason: 'no schema-dump equivalent command' } satisfies MemoryMetric;

    competitors.push({
      ...row,
      available: true,
      obtainedVia: obtained.via,
      version: firstLine(version.text),
      startup,
      schemaDump,
      apiList,
      memory,
      binary: {
        // A launcher shim's bytes are not the tool's: when a payload was
        // resolved, the size metric follows it (see resolveGwsPayload).
        bytes: statSync(obtained.payload ?? obtained.exe).size,
        source: obtained.payload ? 'installed on PATH (npm payload)' : (obtained.asset ?? 'installed on PATH'),
        form: 'binary',
      },
    });
  }

  return {
    generatedAt: new Date().toISOString(),
    environment,
    versions,
    protocol: {
      runs: RUNS,
      warmupRunsDiscarded: WARMUP_RUNS,
      timer: 'process.hrtime.bigint() around spawnSync',
      stability: `re-runs whose medians land within max(${NOISE_ABS_MS} ms, ${Math.round(NOISE_REL * 100)}%) of the committed snapshot change nothing`,
      memoryTimer: process.platform === 'win32'
        ? `peak RSS: unavailable on Windows (no ${TIME_BIN}); never estimated`
        : `peak RSS: max of ${RUNS} runs under ${TIME_BIN} ${TIME_FLAG} (${TIME_UNITS})`,
      memoryWorkload: MEMORY_WORKLOAD,
    },
    grr,
    competitors,
  };
}

// ---------------------------------------------------------------------------
// Idempotency comparison
// ---------------------------------------------------------------------------

const timingWithinNoise = (a: Metric | null | undefined, b: Metric | null | undefined): boolean => {
  if (!a || !b) return a === b;
  if (a.available !== b.available) return false;
  if (!a.available) {
    // Corollary of the equality check above: b is the unavailable variant
    // too, so both reasons are present. (TS cannot track the correlation
    // across two variables; the re-test is free and never fires.)
    return b.available === false && a.reason === b.reason;
  }
  // Medians only. At 15 samples the reported p95 IS the sample max, and the
  // min is the single fastest run — both are one scheduler hiccup away from
  // moving by more than the whole noise band on a busy machine, so keying
  // change detection on them would make the snapshot flap and every flapping
  // rewrite a spurious PR. The median is the stable statistic; it is what
  // the page leads with and what decides whether anything actually moved.
  return b.available === true && withinNoise(a.medianMs, b.medianMs);
};

const sameJson = (a: unknown, b: unknown): boolean => JSON.stringify(a) === JSON.stringify(b);

/**
 * Peak RSS comparison for the idempotency gate. Like the timings, only the
 * reported statistic decides change: a re-run whose high-water mark lands
 * within the memory band (max(2 MB, 10%)) rewrites nothing. Both sides
 * missing is equal; one side missing is a real change (a metric appearing or
 * disappearing from the snapshot must be committed, never papered over).
 */
const memoryWithinNoise = (a: MemoryMetric | null | undefined, b: MemoryMetric | null | undefined): boolean => {
  if (!a || !b) return !a && !b;
  if (a.available !== b.available) return false;
  if (!a.available) return b.available === false && a.reason === b.reason;
  return b.available === true
    && typeof a.maxRssKb === 'number'
    && typeof b.maxRssKb === 'number'
    && withinBand(a.maxRssKb, b.maxRssKb, MEMORY_ABS_KB, MEMORY_REL);
};

function isEquivalent(previous: BenchmarkSnapshot | null, fresh: BenchmarkSnapshot): boolean {
  if (!previous) return false;
  if (!sameJson(previous.environment, fresh.environment)) return false;
  if (!sameJson(previous.versions, fresh.versions)) return false;
  if (!sameJson(previous.protocol, fresh.protocol)) return false;
  if (!timingWithinNoise(previous.grr?.startup, fresh.grr.startup)) return false;
  if (!timingWithinNoise(previous.grr?.schemaDump, fresh.grr.schemaDump)) return false;
  if (!timingWithinNoise(previous.grr?.apiList, fresh.grr.apiList)) return false;
  if (!memoryWithinNoise(previous.grr?.memory, fresh.grr.memory)) return false;
  if (!sameJson(previous.grr?.binary, fresh.grr.binary)) return false;

  const prevByTool = new Map((previous.competitors ?? []).map((c) => [c.tool, c] as const));
  if ((previous.competitors ?? []).length !== fresh.competitors.length) return false;
  return fresh.competitors.every((freshRow) => {
    const prevRow = prevByTool.get(freshRow.tool);
    if (!prevRow) return false;
    if (prevRow.available !== freshRow.available) return false;
    if (prevRow.project !== freshRow.project || prevRow.obtainedVia !== freshRow.obtainedVia) return false;
    if (prevRow.version !== freshRow.version) return false;
    if (freshRow.available) {
      return timingWithinNoise(prevRow.startup, freshRow.startup)
        && memoryWithinNoise(prevRow.memory, freshRow.memory)
        && sameJson(prevRow.binary, freshRow.binary);
    }
    return prevRow.reason === freshRow.reason;
  });
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

const fmtMs = (metric?: Metric | null): string =>
  metric?.available
    ? `median ${metric.medianMs} ms · p95 ${metric.p95Ms} ms · min ${metric.minMs} ms`
    : `unavailable: ${metric?.reason ?? 'not measured'}`;

const fmtMemory = (metric?: MemoryMetric | null): string =>
  metric?.available
    ? `max ${metric.maxRssKb} kB RSS over ${metric.runs ?? '?'} runs of ${metric.command ?? 'the workload'}`
    : `unavailable: ${metric?.reason ?? 'not measured'}`;

const fmtSize = (binary?: BinarySize | null): string => {
  if (binary?.shipped) return `${binary.shipped.bytes} bytes (${binary.shipped.source})`;
  if (binary?.bytes) return `${binary.bytes} bytes (${binary.form}, ${binary.source})`;
  return `unavailable: ${binary?.reason ?? 'not measured'}`;
};

function printSummary(snapshot: BenchmarkSnapshot): void {
  const { environment, versions, grr, competitors } = snapshot;
  console.log(
    `environment: ${environment.platform}/${environment.arch} · node ${environment.nodeVersion} · grr binary: ${environment.binaryKind ?? 'none'}`,
  );
  if (environment.grrVersion) console.log(`grr version: ${environment.grrVersion}`);
  for (const [tool, entry] of Object.entries(versions ?? {})) {
    const local = entry.local ?? 'did not run';
    const tag = entry.latestTag ?? `no tag (${entry.latestTagReason ?? 'unfetched'})`;
    console.log(`${tool} versions: measured ${local} · latest release ${tag}`);
  }
  console.log(`grr startup: ${fmtMs(grr.startup)}`);
  console.log(`grr schema: ${fmtMs(grr.schemaDump)}`);
  console.log(`grr api list: ${fmtMs(grr.apiList)}`);
  console.log(`grr memory: ${fmtMemory(grr.memory)}`);
  console.log(`grr binary: ${fmtSize(grr.binary)}`);
  if (grr.binary?.shipped && grr.binary.bytes) {
    console.log(`grr binary (unpacked build): ${grr.binary.bytes} bytes (${grr.binary.source})`);
  }
  for (const row of competitors) {
    console.log(
      row.available
        ? `${row.tool}: ${fmtMs(row.startup)} · ${fmtMemory(row.memory)} · via ${row.obtainedVia}`
        : `${row.tool}: unavailable — ${row.reason}`,
    );
  }
}

// ---------------------------------------------------------------------------
// Version gate (`--gate`): the cheap pre-flight the daily cron runs BEFORE
// any build. Compares the committed snapshot's `versions` block against (a)
// a fresh latest-release-tag fetch for grr, gog and gws and (b) the version
// in Cargo.toml — which catches main moving ahead of the last release.
// Prints GitHub Actions output pairs (`skip=`, `reason=`) on stdout; human
// logs go to stderr. Never exits non-zero and FAILS OPEN (skip=false): a
// flaky GitHub API must never be able to hide a competitor release.
// ---------------------------------------------------------------------------

const TRACKED_PROJECTS: Array<{ tool: string; project: string }> = [
  { tool: 'grr', project: GRR_PROJECT },
  ...COMPETITORS.map(({ tool, project }) => ({ tool, project })),
];

const semverOf = (text: unknown): string | null => {
  const match = /(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)/.exec(String(text ?? ''));
  return match ? match[1] : null;
};

const isDeterministicMissing = (reason: string | undefined): boolean => /HTTP 404/.test(String(reason ?? ''));

async function runGate(): Promise<number> {
  const changes: string[] = [];
  try {
    // Committed snapshot: generated JSON, cast to the snapshot shape and
    // read defensively — every field access below keeps the `??`/`?.` the
    // .mjs had, so a truncated file degrades the same way.
    let committed: BenchmarkSnapshot | null = null;
    try {
      committed = JSON.parse(readFileSync(OUT_FILE, 'utf8')) as BenchmarkSnapshot;
    } catch (error) {
      changes.push(`no readable committed snapshot (${errorMessage(error)})`);
    }

    if (committed) {
      const recorded: VersionsBlock = committed.versions ?? {};

      // (a) grr's working-tree version: Cargo.toml is what the daily build
      // compiles, so main moving past the last measured version counts as a
      // change even before a release is cut.
      const cargoMatch = /^version\s*=\s*"([^"]+)"/m.exec(readFileSync(join(ROOT, 'Cargo.toml'), 'utf8'));
      const cargoVersion = cargoMatch ? cargoMatch[1] : null;
      const measuredVersion = semverOf(recorded.grr?.local);
      if (cargoVersion && measuredVersion) {
        if (cargoVersion !== measuredVersion) {
          changes.push(`grr working tree is ${cargoVersion}, last snapshot measured ${measuredVersion}`);
        }
      } else {
        changes.push('grr version missing from Cargo.toml or the committed snapshot');
      }

      // (b) latest release tag of every tracked project.
      for (const { tool, project } of TRACKED_PROJECTS) {
        const fresh = await latestReleaseTag(project);
        const entry = recorded[tool];
        const recordedTag = entry && typeof entry.latestTag === 'string' ? entry.latestTag : null;

        if (fresh.latestTag !== null) {
          if (fresh.latestTag !== recordedTag) {
            changes.push(`${tool} latest release is ${fresh.latestTag}, snapshot recorded ${recordedTag ?? 'none'}`);
          }
        } else if (recordedTag !== null) {
          changes.push(`${tool} latest release could not be fetched (${fresh.latestTagReason}), snapshot recorded ${recordedTag}`);
        } else if (!isDeterministicMissing(fresh.latestTagReason) || !isDeterministicMissing(entry?.latestTagReason)) {
          // Both sides missing is only "unchanged" when both are the
          // deterministic "no releases exist" 404 — anything else (rate
          // limit, transient 5xx) fails open.
          changes.push(`${tool} latest release still unfetchable (${fresh.latestTagReason})`);
        }
      }
    }
  } catch (error) {
    changes.push(`gate error: ${errorMessage(error)}`);
  }

  const skip = changes.length === 0;
  console.log(`skip=${skip}`);
  console.log(
    `reason=${skip ? 'no version of grr, gog or gws moved since the last measured snapshot' : changes.join('; ')}`,
  );
  return 0;
}

async function main(): Promise<number> {
  if (GATE_ONLY) return runGate();

  const fresh = await buildSnapshot();
  printSummary(fresh);

  const previous = existsSync(OUT_FILE) ? (JSON.parse(readFileSync(OUT_FILE, 'utf8')) as BenchmarkSnapshot) : null;
  const equivalent = isEquivalent(previous, fresh);

  if (equivalent) {
    console.log('benchmarks:changed=false');
    return 0;
  }

  if (CHECK_ONLY) {
    console.error(
      'benchmarks: stale — site/src/data/benchmarks.json disagrees with a fresh measurement beyond the noise band.',
    );
    return 1;
  }

  mkdirSync(dirname(OUT_FILE), { recursive: true });
  writeFileSync(OUT_FILE, `${JSON.stringify(fresh, null, 2)}\n`);
  console.log('benchmarks:changed=true');
  return 0;
}

if (process.argv[1] && resolve(process.argv[1]) === import.meta.filename) {
  process.exit(await main());
}
