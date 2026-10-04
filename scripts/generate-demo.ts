#!/usr/bin/env node
// Generate demo/demo.cast — an asciinema recording (asciicast v2, JSON lines)
// of the real grr CLI, for the repo and the site's terminal embed.
//
// Why generated rather than recorded with `asciinema rec`: a live recording
// captures whatever the terminal printed that day — log noise, ANSI escapes
// from whatever theme was active, and (for anything that reads the mailbox)
// private data. The demo must be reproducible and leak nothing, so the script
// instead RUNS every command for real, captures stdout (stderr is logs — grr's
// contract), sanitizes it, and renders the frames with deterministic
// inter-line timing. The output is real; only the pacing is synthetic.
//
// Modes:
//   node scripts/generate-demo.ts                  # CI mode: offline/dry-run commands only
//   node scripts/generate-demo.ts --local          # + credential-backed commands + agent segment
//   node scripts/generate-demo.ts --out <path>     # write elsewhere (CI: a throwaway path)
//   node scripts/generate-demo.ts --validate [<path>]  # parse the .cast as JSON lines, assert it
//
// The committed demo/demo.cast is generated with --local on the owner's
// machine — the only place that has a Google token, a TypesSafe API key and
// opencode. CI never regenerates the committed cast: in a CI run the agent
// segment would be absent, so a regenerated committed cast would differ by
// exactly that missing segment — the regression .github/workflows/demo.yml
// exists to prevent. The workflow validates only; regeneration is local.
//
// The agent segment (--local only) runs `opencode run` — the owner's agentic
// harness — with a prompt that drives grr end to end. The prompt is
// deliberately self-describing (transport, ask plan, dry-run) rather than a
// mailbox read: a demo that quoted real email subjects or sender addresses
// would leak private data, and this one cannot. The agent's raw `--format
// json` event stream is parsed and rendered as plain text (tool calls with
// their output, the agent's text, the final answer), so the transcript does
// not depend on opencode's TTY UI. If opencode is missing, fails or times
// out, the segment is skipped and the skip is recorded in the generation
// report — agent output is never fabricated.

import { spawn } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

const ROOT = resolve(import.meta.dirname, '..');
const DEFAULT_OUT = resolve(ROOT, 'demo', 'demo.cast');

// Matches the site's terminal aesthetic. Demo lines are hard-wrapped a little
// inside this so nothing clips at the edge.
// 130 cols × 34 rows: a laptop-shaped terminal. At the shell's max width
// (~937px) the player's fit-to-width maths lands the font at ~13.4px — the
// same size as the site's body text — with a natural ~16:10 aspect.
const WIDTH = 130;
const HEIGHT = 34;
const WRAP = 126;

// Deterministic pacing, tuned so the full cast plays in ~30-60s. Fixed delays
// scaled by line length: 40-120ms for command output, a little slower for the
// agent transcript so its prose stays readable.
const FIRST_DELAY = 0.6;
const TYPE_DELAY = 0.05;
const EXEC_DELAY = 0.55;
const NOTE_PAUSE = 0.5;
const SEGMENT_GAP = 1.4;
const lineDelay = (len: number): number => Math.min(0.04 + len * 0.0005, 0.12);
const agentLineDelay = (len: number): number => Math.min(0.12 + len * 0.0008, 0.35);

const CMD_TIMEOUT_MS = 30_000;
// opencode's model latency swings widely between runs (the same prompt has
// landed anywhere from ~170s to over 240s), so the cap is generous: a skipped
// segment is reported honestly, agent output is never fabricated.
const AGENT_TIMEOUT_MS = 300_000;
const AGENT_MAX_LINES = 40;
const AGENT_TAIL = 8;
const TOOL_OUTPUT_LINES = 5;

// Drives grr three ways without ever touching the mailbox: the transport
// facts, a natural-language plan, and the exact request envelope. Real agent
// usage, zero private data.
const AGENT_PROMPT =
  "Use the grr CLI to show what it can do. Use grr's own JSON output for everything. " +
  '1) Run grr transport to show the negotiated transport and runtime features. ' +
  '2) Run grr ask "show my unread messages" to see how natural language resolves to a typed method and params. ' +
  '3) Run grr gmail users messages list --user-id me --dry-run to print the exact request JSON the API would receive. ' +
  'One short line of explanation after each step.';

interface CaptureResult {
  stdout: string;
  code: number | null;
  killed: boolean;
}

interface SegmentReport {
  display: string;
  kind: string;
  status: 'ran' | 'skipped' | 'failed';
  lines: number;
  note: string;
}

interface PreparedCommand {
  kind: 'command';
  mode: 'ci' | 'local';
  display: string;
  lines: string[];
}

interface PreparedAgent {
  kind: 'agent';
  mode: 'local';
  display: string;
  lines: string[];
}

type Prepared = PreparedCommand | PreparedAgent;

type Frame =
  | { kind: 'note'; text: string }
  | { kind: 'seg'; seg: Prepared };

// ---------------------------------------------------------------------------
// spawning + capture
// ---------------------------------------------------------------------------

function spawnCapture(
  cmd: string,
  args: string[],
  opts: { timeoutMs?: number; pathPrepend?: string } = {},
): Promise<CaptureResult> {
  return new Promise((res, rej) => {
    const env: NodeJS.ProcessEnv = { ...process.env };
    if (opts.pathPrepend) {
      // process.env on Windows spells the key "Path"; spreading {...env, PATH}
      // would add a second, conflicting entry.
      const key = process.platform === 'win32' ? 'Path' : 'PATH';
      const sep = process.platform === 'win32' ? ';' : ':';
      env[key] = opts.pathPrepend + sep + (env[key] ?? '');
    }
    // stdin is ignored on purpose: with a piped stdin that never closes,
    // opencode waits for EOF instead of running.
    const child = spawn(cmd, args, {
      cwd: ROOT,
      env,
      windowsHide: true,
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    let stdout = '';
    let timer: ReturnType<typeof setTimeout> | null = null;
    let killed = false;
    if (opts.timeoutMs) {
      timer = setTimeout(() => {
        killed = true;
        if (child.pid) {
          if (process.platform === 'win32') {
            spawn('taskkill', ['/pid', String(child.pid), '/T', '/F'], {
              windowsHide: true,
              stdio: 'ignore',
            });
          } else {
            child.kill('SIGKILL');
          }
        }
      }, opts.timeoutMs);
    }
    child.stdout.on('data', (d: Buffer) => {
      stdout += d.toString('utf8');
    });
    // stderr is logs — grr's contract; the demo renders stdout only.
    child.stderr.on('data', () => {});
    child.on('error', (e) => {
      if (timer) clearTimeout(timer);
      rej(e);
    });
    child.on('close', (code) => {
      if (timer) clearTimeout(timer);
      res({ stdout, code, killed });
    });
  });
}

// ---------------------------------------------------------------------------
// sanitizing + shaping
// ---------------------------------------------------------------------------

const ANSI_RE = /\u001B\[[0-9;?]*[ -/]*[@-~]|\u001B\][^\u0007]*(?:\u0007|\u001B\\)/g;

function sanitize(text: string): string {
  let out = text.replace(ANSI_RE, '');
  out = out.replace(/\r\n?/g, '\n');
  // Home-directory paths carry the username; the demo says `~`. The doubled
  // backslashes are Rust Debug formatting — the agent's captured output has
  // them, a plain path does not.
  out = out.replace(/C:[\\/]{1,2}(?:Users|home)[\\/]{1,2}[^\s"'`]+/g, '~');
  out = out.replace(/\/home\/[^\s"'`]+/g, '~');
  // The OAuth client id must never enter the repo (AGENTS.md invariant 7) —
  // grr's config_loader logs print it, and the agent's shell captures those
  // logs mixed with stdout.
  out = out.replace(/[A-Za-z0-9-]+\.apps\.googleusercontent\.com/g, '[redacted]');
  // No email address beyond `me` — the demo redacts every one, including
  // Google's own `someuser@example.com` documentation example.
  out = out.replace(/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/g, '[redacted]');
  out = out.replace(/Bearer\s+\S+/gi, 'Bearer [redacted]');
  // Long hex / base64 runs: token or key material until proven otherwise.
  out = out.replace(/\b[A-Fa-f0-9]{32,}\b/g, '[redacted]');
  out = out.replace(/\b[A-Za-z0-9+/]{44,}={0,2}\b/g, '[redacted]');
  return out;
}

// grr's stderr log lines, as they appear when a shell captures stderr and
// stdout mixed — the agent's tool output. Dropped, not redacted: the demo
// renders stdout only (grr's contract), and the log lines are the exact place
// the client id leaked from.
const LOG_LINE_RE = /^\d{4}-\d{2}-\d{2}T[\d:.]+Z\s+(?:INFO|WARN|ERROR|DEBUG|TRACE)\b/;

function dropLogLines(lines: string[]): string[] {
  return lines.filter((l) => !LOG_LINE_RE.test(l));
}

function cleanLines(text: string): string[] {
  const lines = text.split('\n').map((l) => l.replace(/[ \t]+$/g, ''));
  const out: string[] = [];
  let blanks = 0;
  for (const l of lines) {
    if (l.length === 0) {
      blanks += 1;
      if (blanks <= 1) out.push(l);
    } else {
      blanks = 0;
      out.push(l);
    }
  }
  while (out.length > 0 && out[0].length === 0) out.shift();
  while (out.length > 0 && out[out.length - 1].length === 0) out.pop();
  return out;
}

function wrapLine(line: string): string[] {
  if (line.length <= WRAP) return [line];
  // Wrap at SPACES, never mid-token: the fixed-width slices this once used
  // broke JSON keys and values across lines ("messa | ges"), which is the
  // single ugliest thing a terminal demo can do.
  const out: string[] = [];
  let rest = line;
  while (rest.length > WRAP) {
    const window = rest.slice(0, WRAP + 1);
    const lastSpace = window.lastIndexOf(' ');
    if (lastSpace <= 0) {
      // One token longer than the wrap: hard-break it rather than loop.
      out.push(rest.slice(0, WRAP));
      rest = rest.slice(WRAP);
    } else {
      out.push(rest.slice(0, lastSpace));
      rest = rest.slice(lastSpace + 1);
    }
  }
  if (rest.length > 0) out.push(rest);
  return out;
}

function headLines(lines: string[], n: number): { lines: string[]; note: string | null } {
  if (lines.length <= n) return { lines, note: null };
  return { lines: lines.slice(0, n), note: `# ... ${lines.length - n} more lines` };
}

function headChars(lines: string[], n: number): { lines: string[]; note: string | null } {
  const joined = lines.join('\n');
  if (joined.length <= n) return { lines, note: null };
  return { lines: wrapLine(joined.slice(0, n)), note: `# ... ${joined.length - n} more bytes` };
}

// ---------------------------------------------------------------------------
// the real commands
// ---------------------------------------------------------------------------

function findGrr(): string {
  const env = process.env.GRR_BIN;
  if (env) return env;
  const exe = process.platform === 'win32' ? 'grr.exe' : 'grr';
  // Debug before release: the demo must show the current build.
  for (const dir of ['target/debug', 'target/release']) {
    const p = resolve(ROOT, dir, exe);
    if (existsSync(p)) return p;
  }
  return 'grr';
}

// ---------------------------------------------------------------------------
// the shell the demo shows
// ---------------------------------------------------------------------------

// Two separate concerns that must not be conflated:
//
// WSL_INTEROP — the MECHANISM: route commands through wsl.exe so grr.exe's
// output is captured from a real bash session on Windows. Windows-only;
// on Linux/macOS the shell IS bash and commands run directly (spawning
// wsl.exe there is ENOENT — the exact failure the first CI recording hit).
//
// BASH_PROMPT — the COSMETIC: show the bash-style prompt and /bin/bash in
// the cast header. True everywhere except an explicit --shell=powershell.
const WSL_INTEROP: boolean =
  process.platform === 'win32' &&
  (process.argv.includes('--shell=wsl') || process.argv.includes('--local')) &&
  !process.argv.includes('--shell=powershell');

const BASH_PROMPT: boolean = !process.argv.includes('--shell=powershell');

/** A Windows path as WSL sees it: /mnt/c/... */
function toWslPath(windowsPath: string): string {
  const m = /^([A-Za-z]):[\\/](.*)$/.exec(windowsPath);
  if (!m) return windowsPath;
  const rest = m[2].replace(/\\/g, '/');
  return `/mnt/${m[1].toLowerCase()}/${rest}`;
}

// The prompt shows the real user: debanjanbasu locally (WSL), whatever the
// CI runner reports otherwise (GitHub runners are 'runner').
const BASH_PROMPT_USER = process.platform === 'win32' ? 'debanjanbasu' : process.env.USER || process.env.LOGNAME || 'runner';
const BASH_PROMPT_HOST = 'grr';

/** POSIX shell quote: only values with metacharacters need it. */
function shellQuote(value: string): string {
  return /^[A-Za-z0-9_@%+=:,./-]+$/.test(value) ? value : `'${value.replace(/'/g, `'\\''`)}'`;
}

async function runGrr(grrBin: string, args: string[]): Promise<string> {
  // WSL mode: the command runs through bash so the demo shows the Unix
  // shell; the binary path becomes the /mnt/c form WSL understands. The
  // display stays `grr ...` — the .exe suffix is a Windows-interop detail,
  // and on every other platform the tool is `grr`.
  if (WSL_INTEROP) {
    const chain = `${toWslPath(grrBin)} ${args.map(shellQuote).join(' ')}`;
    const r = await spawnCapture(
      'wsl.exe',
      ['-d', 'Ubuntu', '--', 'bash', '-lc', chain],
      { timeoutMs: CMD_TIMEOUT_MS },
    );
    if (r.code !== 0) {
      throw new Error(`wsl bash -lc "${chain}" exited with code ${r.code}${r.killed ? ' (timed out)' : ''}`);
    }
    return r.stdout;
  }
  const r = await spawnCapture(grrBin, args, { timeoutMs: CMD_TIMEOUT_MS });
  if (r.code !== 0) {
    throw new Error(`${grrBin} ${args.join(' ')} exited with code ${r.code}${r.killed ? ' (timed out)' : ''}`);
  }
  return sanitize(r.stdout);
}

interface Trunc {
  headLines?: number;
  headChars?: number;
}

async function prepCommand(
  mode: 'ci' | 'local',
  display: string,
  args: string[],
  trunc: Trunc = {},
): Promise<PreparedCommand> {
  const cleaned = cleanLines(await runGrr(findGrr(), args));
  const t = trunc.headLines !== undefined
    ? headLines(cleaned, trunc.headLines)
    : trunc.headChars !== undefined
      ? headChars(cleaned, trunc.headChars)
      : { lines: cleaned, note: null };
  const lines = t.note ? [...t.lines, t.note] : t.lines;
  return { kind: 'command', mode, display, lines };
}

// The agent's raw `--format json` event stream, rendered as plain text: tool
// calls with their output, the agent's prose, the final answer — in order,
// without depending on opencode's TTY UI.
interface OcEvent {
  type: string;
  part?: {
    tool?: string;
    text?: string;
    state?: {
      status?: string;
      input?: { command?: string; [k: string]: unknown };
      output?: string;
    };
  };
}

function renderAgentEvents(raw: string): string[] {
  const out: string[] = [];
  for (const line of raw.split(/\r?\n/)) {
    if (!line.trim()) continue;
    let ev: OcEvent;
    try {
      ev = JSON.parse(line) as OcEvent;
    } catch {
      continue;
    }
    if (ev.type === 'text' && typeof ev.part?.text === 'string') {
      for (const l of dropLogLines(cleanLines(sanitize(ev.part.text)))) out.push(l);
    } else if (ev.type === 'tool_use' && ev.part?.state?.status === 'completed') {
      const input = ev.part.state.input ?? {};
      const cmd = typeof input.command === 'string' ? input.command : null;
      if (cmd) {
        out.push(`$ ${sanitize(cmd)}`);
      } else {
        const { additionalContext: _ignored, ...rest } = input;
        const s = JSON.stringify(rest);
        out.push(`$ [${ev.part.tool ?? 'tool'}] ${s.length > 80 ? s.slice(0, 80) + '...' : s}`);
      }
      // The agent's shell captures stderr and stdout mixed; drop grr's log
      // lines so the transcript shows what the tool actually printed.
      const outLines = dropLogLines(cleanLines(sanitize(ev.part.state.output ?? '')));
      for (const l of outLines.slice(0, TOOL_OUTPUT_LINES)) out.push(l);
      if (outLines.length > TOOL_OUTPUT_LINES) out.push(`# ... ${outLines.length - TOOL_OUTPUT_LINES} more lines`);
    }
  }
  return out;
}

function truncateAgent(lines: string[]): string[] {
  if (lines.length <= AGENT_MAX_LINES) return lines;
  const head = lines.slice(0, AGENT_MAX_LINES - AGENT_TAIL - 1);
  const tail = lines.slice(-AGENT_TAIL);
  const omitted = lines.length - head.length - tail.length;
  return [...head, `# ... ${omitted} lines omitted`, ...tail];
}

/**
 * A committed-cast segment extractor: collect the 'o' output lines between a
 * start marker and the next segment boundary (a `$` prompt or a `#` note).
 * Used to carry the segments CI cannot re-record — `grr transport` (needs a
 * Google token) and the opencode agent run — so a CI regeneration never
 * strips them from the site.
 */
function carrySegment(
  castPath: string,
  startMarker: string,
): { lines: string[] } | null {
  let raw: string;
  try {
    raw = readFileSync(castPath, 'utf8');
  } catch {
    return null;
  }
  const lines: string[] = [];
  let inside = false;
  for (const line of raw.split('\n')) {
    if (line.length === 0) continue;
    let event: unknown;
    try {
      event = JSON.parse(line);
    } catch {
      continue;
    }
    if (!Array.isArray(event) || event.length !== 3) continue;
    const [, type, text] = event as [number, string, string];
    if (typeof text !== 'string') continue;
    const plain = text.replace(/\u001b\[[0-9;]*m/g, '');
    if (!inside && type === 'o' && plain.includes(startMarker)) {
      inside = true;
      continue;
    }
    if (inside) {
      // The next prompt or narration note ends the segment.
      if (plain.startsWith('$ ') || plain.startsWith('# ')) break;
      for (const l of plain.split('\r\n')) {
        const trimmed = l.replace(/\r/g, '');
        if (trimmed.length > 0) lines.push(trimmed);
      }
    }
  }
  return lines.length > 0 ? { lines } : null;
}

/**
 * The committed cast's agent segment, extracted for carry-forward.
 *
 * CI (`--ci-record`) regenerates the cast from source. It can run the live
 * agent segment when opencode works there (free default models or
 * credentials in the environment); when it cannot, this salvages the
 * previously recorded segment from the committed cast so a CI regeneration
 * never strips the agent demo from the site.
 */
function carryAgentSegment(castPath: string): { seg: PreparedAgent; report: SegmentReport } | null {
  const carried = carrySegment(castPath, 'opencode agent driving');
  if (!carried) return null;
  return {
    seg: { kind: 'agent', mode: 'local', display: `opencode run "${AGENT_PROMPT}"`, lines: carried.lines },
    report: {
      display: 'opencode run "<prompt>"',
      kind: 'agent',
      status: 'ran',
      lines: carried.lines.length,
      note: 'carried forward from the committed cast (live agent run unavailable)',
    },
  };
}

async function prepAgent(): Promise<{ seg: PreparedAgent | null; report: SegmentReport }> {
  const skipped = (note: string): { seg: null; report: SegmentReport } => ({
    seg: null,
    report: { display: 'opencode run "<prompt>"', kind: 'agent', status: 'skipped', lines: 0, note },
  });
  let ocVersion = '';
  try {
    const r = await spawnCapture('opencode', ['--version'], { timeoutMs: CMD_TIMEOUT_MS });
    ocVersion = r.stdout.trim();
  } catch {
    return skipped('opencode not found on PATH');
  }
  if (!ocVersion) return skipped('opencode not found on PATH');

  // The grr binary dir is prepended to the child PATH so the agent's `grr`
  // resolves to the current build, not whatever stale copy is first on PATH.
  let raw = '';
  let killed = false;
  try {
    const r = await spawnCapture('opencode', ['run', AGENT_PROMPT, '--title', 'demo-agent', '--format', 'json'], {
      timeoutMs: AGENT_TIMEOUT_MS,
      pathPrepend: dirname(findGrr()),
    });
    raw = r.stdout;
    killed = r.killed;
  } catch (e) {
    return skipped(`opencode run failed: ${e instanceof Error ? e.message : String(e)}`);
  }
  if (killed) return skipped(`opencode run exceeded the ${AGENT_TIMEOUT_MS / 1000}s cap; partial output discarded`);
  const transcript = renderAgentEvents(raw);
  if (!transcript.some((l) => l.startsWith('$ '))) {
    return skipped('opencode produced no grr tool calls');
  }
  const lines = truncateAgent(transcript);
  return {
    seg: { kind: 'agent', mode: 'local', display: `opencode run "${AGENT_PROMPT}"`, lines },
    report: {
      display: 'opencode run "<prompt>"',
      kind: 'agent',
      status: 'ran',
      lines: lines.length,
      note: `opencode ${ocVersion}, transcript captured from --format json events`,
    },
  };
}

async function buildSegments(mode: 'ci' | 'local' | 'ci-record'): Promise<{ segs: Prepared[]; reports: SegmentReport[] }> {
  const segs: Prepared[] = [];
  const reports: SegmentReport[] = [];
  const push = async (p: Promise<PreparedCommand>, display: string): Promise<void> => {
    const seg = await p;
    segs.push(seg);
    reports.push({ display, kind: 'command', status: 'ran', lines: seg.lines.length, note: 'real output, captured' });
  };
  // A command that may legitimately fail (e.g. `grr ask` with no key on a
  // given runner): skip with a report instead of aborting the recording.
  const tryPush = async (p: Promise<PreparedCommand>, display: string, skipNote: string): Promise<void> => {
    try {
      await push(p, display);
    } catch {
      reports.push({ display, kind: 'command', status: 'skipped', lines: 0, note: skipNote });
    }
  };

  await push(prepCommand('ci', 'grr --version', ['--version']), 'grr --version');
  await push(
    prepCommand('ci', 'grr api list --service gmail --filter "messages.list"', [
      'api',
      'list',
      '--service',
      'gmail',
      '--filter',
      'messages.list',
    ]),
    'grr api list --service gmail --filter "messages.list"',
  );
  await push(
    prepCommand('ci', 'grr api describe gmail.users.messages.list', ['api', 'describe', 'gmail.users.messages.list'], {
      headChars: 400,
    }),
    'grr api describe gmail.users.messages.list',
  );
  await push(
    prepCommand('ci', 'grr gmail users messages list --user-id me --dry-run', [
      'gmail',
      'users',
      'messages',
      'list',
      '--user-id',
      'me',
      '--dry-run',
    ]),
    'grr gmail users messages list --user-id me --dry-run',
  );
  await push(prepCommand('ci', 'grr schema | head -c 400', ['schema'], { headChars: 400 }), 'grr schema | head -c 400');
  await push(prepCommand('ci', 'grr --help | head', ['--help'], { headLines: 10 }), 'grr --help | head');

  if (mode === 'local') {
    await push(prepCommand('local', 'grr transport', ['transport']), 'grr transport');
    await push(
      prepCommand('local', 'grr ask "show my unread messages" --dry-run', ['ask', 'show my unread messages', '--dry-run']),
      'grr ask "show my unread messages" --dry-run',
    );
    const agent = await prepAgent();
    if (agent.seg) {
      segs.push(agent.seg);
      reports.push(agent.report);
    } else {
      // opencode is flaky (agent runs regularly hit the 300s cap). The
      // committed cast is the carry source — same rule as CI: a failed
      // live run degrades to the previous recording rather than stripping
      // the agent demo from the site.
      const carried = carryAgentSegment(DEFAULT_OUT);
      if (carried) {
        segs.push(carried.seg);
        reports.push(carried.report);
      } else {
        reports.push(agent.report);
      }
    }
  } else if (mode === 'ci-record') {
    // Self-maintenance mode: CI regenerates the cast from source. It has no
    // Google token (transport stays local-only), but with a TYPESAFE_API_KEY
    // secret it records the real natural-language plan, and opencode's free
    // default models (or credentials via env secrets) may allow a LIVE agent
    // segment — with carry-forward from the committed cast as the safety net,
    // so a CI regeneration never strips the agent demo from the site.
    // `grr transport` needs a live Google token — unrecordable in CI. Its
    // committed segment is carried forward: the transport facts (HTTP/3
    // negotiated, runtime features) only change when the transport code
    // itself changes, and a fresh --local run refreshes them.
    const carriedTransport = carrySegment(DEFAULT_OUT, 'grr transport');
    if (carriedTransport) {
      segs.push({ kind: 'command', mode: 'local', display: 'grr transport', lines: carriedTransport.lines });
      reports.push({
        display: 'grr transport',
        kind: 'command',
        status: 'ran',
        lines: carriedTransport.lines.length,
        note: 'carried forward from the committed cast (live transport needs a Google token)',
      });
    } else {
      reports.push({
        display: 'grr transport',
        kind: 'command',
        status: 'skipped',
        lines: 0,
        note: 'needs a Google token; recorded by --local runs only',
      });
    }
    await tryPush(
      prepCommand('local', 'grr ask "show my unread messages" --dry-run', ['ask', 'show my unread messages', '--dry-run']),
      'grr ask "show my unread messages" --dry-run',
      'no System One key in the environment (set the TYPESAFE_API_KEY secret)',
    );
    const liveAgent = await prepAgent();
    if (liveAgent.seg) {
      segs.push(liveAgent.seg);
      reports.push(liveAgent.report);
    } else {
      const carried = carryAgentSegment(DEFAULT_OUT);
      if (carried) {
        segs.push(carried.seg);
        reports.push(carried.report);
      } else {
        reports.push({
          display: 'opencode run "<prompt>"',
          kind: 'agent',
          status: 'skipped',
          lines: 0,
          note: `agent run failed (${liveAgent.report.note}) and no committed segment to carry forward`,
        });
      }
    }
  } else {
    reports.push({
      display: 'grr transport / grr ask / opencode agent',
      kind: 'local-only',
      status: 'skipped',
      lines: 0,
      note: 'credential-backed segments are local-only (no Google token, no TypesSafe key, no opencode on CI)',
    });
  }
  return { segs, reports };
}

// ---------------------------------------------------------------------------
// the cast
// ---------------------------------------------------------------------------

const r3 = (x: number): number => Math.round(x * 1000) / 1000;

function buildCast(frames: Frame[], closing: string): { events: Array<[number, 'o' | 'i', string]>; duration: number } {
  const ev: Array<[number, 'o' | 'i', string]> = [];
  let t = FIRST_DELAY;
  for (let i = 0; i < frames.length; i++) {
    const f = frames[i];
    if (i > 0) t += SEGMENT_GAP;
    if (f.kind === 'note') {
      t += NOTE_PAUSE;
      // \r\n, never a bare \n: asciinema-player's canvas terminal treats a
      // bare LF as "next row, SAME column" (strict LF, no implicit CR), so a
      // synthesized cast with bare \n renders as a staircase — each line
      // starting where the previous one ended. Real recordings carry \r\n
      // from the TTY, which is why only synthesized casts ever broke.
      ev.push([r3(t), 'o', `\u001b[2m${f.text}\u001b[0m\r\n`]);
      t += lineDelay(f.text.length);
      continue;
    }
    const seg = f.seg;
    // The typed command, as an input event, then its echoed prompt line.
    ev.push([r3(t), 'i', seg.display]);
    t += TYPE_DELAY;
    // The bash prompt shape everywhere except --shell=powershell.
    const echo = BASH_PROMPT
      ? `\u001b[36m${BASH_PROMPT_USER}@${BASH_PROMPT_HOST}:~$\u001b[0m ${seg.display}`
      : `\u001b[36m$\u001b[0m ${seg.display}`;
    for (const l of wrapLine(echo)) {
      ev.push([r3(t), 'o', l + '\r\n']);
      t += 0.02;
    }
    t += EXEC_DELAY;
    const delay = seg.kind === 'agent' ? agentLineDelay : lineDelay;
    for (const line of seg.lines) {
      for (const l of wrapLine(line)) {
        ev.push([r3(t), 'o', l + '\r\n']);
        t += delay(l.length);
      }
      if (seg.kind === 'agent') t += 0.06;
    }
  }
  t += SEGMENT_GAP;
  ev.push([r3(t), 'o', `\u001b[2m${closing}\u001b[0m\r\n`]);
  return { events: ev, duration: t };
}

function writeCast(path: string, events: Array<[number, 'o' | 'i', string]>, duration: number): void {
  mkdirSync(dirname(path), { recursive: true });
  const header = {
    version: 2,
    width: WIDTH,
    height: HEIGHT,
    timestamp: Math.floor(Date.now() / 1000),
    duration: r3(duration),
    title: 'grr — Google tools from the terminal',
    env: {
      SHELL: BASH_PROMPT ? '/bin/bash' : process.env.SHELL ?? 'powershell.exe',
      TERM: 'xterm-256color',
    },
  };
  const lines = [JSON.stringify(header), ...events.map(([t, type, text]) => JSON.stringify([t, type, text]))];
  // Node writes UTF-8 without a BOM — the PowerShell out-cmdlets would mangle
  // the em dash in the title (a real bug in this repo once).
  writeFileSync(path, lines.join('\n') + '\n', 'utf8');
}

// ---------------------------------------------------------------------------
// validation
// ---------------------------------------------------------------------------

const ALLOWED_EMAILS = new Set<string>();

function validateCast(path: string): boolean {
  const problems: string[] = [];
  const raw = readFileSync(path, 'utf8');
  if (!raw.endsWith('\n')) problems.push('file does not end with a newline');
  if (raw.includes('\r')) problems.push('file contains carriage returns');
  const lines = raw.split('\n').filter((l) => l.length > 0);
  if (lines.length < 15) problems.push(`only ${lines.length} lines — too short to be the demo`);

  let header: Record<string, unknown> | null = null;
  let lastT = -1;
  let events = 0;
  let inputHasAgent = false;
  const texts: string[] = [];
  // A plain for loop, not .forEach: the header assignment must participate
  // in control-flow analysis for the narrowing below, and narrowing does not
  // track assignments made inside a callback.
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    let parsed: unknown;
    try {
      parsed = JSON.parse(line);
    } catch {
      problems.push(`line ${i + 1}: not valid JSON`);
      continue;
    }
    if (i === 0) {
      if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
        problems.push('line 1: header is not a JSON object');
        continue;
      }
      header = parsed as Record<string, unknown>;
      continue;
    }
    if (!Array.isArray(parsed) || parsed.length !== 3) {
      problems.push(`line ${i + 1}: event is not a 3-element array`);
      continue;
    }
    const [t, type, text] = parsed as [unknown, unknown, unknown];
    if (typeof t !== 'number' || !Number.isFinite(t) || t < 0) {
      problems.push(`line ${i + 1}: event time is not a finite non-negative number`);
      continue;
    }
    if (t < lastT) problems.push(`line ${i + 1}: event time went backwards (${t} < ${lastT})`);
    lastT = t;
    if (type !== 'o' && type !== 'i') {
      problems.push(`line ${i + 1}: unknown event type ${JSON.stringify(type)}`);
      continue;
    }
    if (typeof text !== 'string') {
      problems.push(`line ${i + 1}: event text is not a string`);
      continue;
    }
    events += 1;
    if (type === 'i' && text.includes('opencode run')) inputHasAgent = true;
    texts.push(text);
  }

  if (header) {
    if (header.version !== 2) problems.push('header version is not 2');
    if (header.width !== WIDTH) problems.push(`header width is not ${WIDTH}`);
    if (header.height !== HEIGHT) problems.push(`header height is not ${HEIGHT}`);
    const env = header.env;
    if (typeof env !== 'object' || env === null) {
      problems.push('header env is missing');
    } else if (typeof (env as Record<string, unknown>).SHELL !== 'string') {
      problems.push('header env SHELL is missing');
    }
    if (typeof header.timestamp !== 'number') problems.push('header timestamp is missing');
  } else {
    problems.push('no header line');
  }

  // Privacy: no Bearer, no key material, no email address beyond `me`, no
  // OAuth client id (it appears in grr's config_loader logs), no absolute
  // Windows home path (the username).
  const all = lines.join('\n');
  if (/bearer/i.test(all)) problems.push('cast contains "Bearer"');
  if (/\.apps\.googleusercontent\.com/.test(all)) problems.push('cast contains an OAuth client id');
  if (/C:[\\]+Users[\\/]/.test(all)) problems.push('cast contains an absolute Windows home path');
  const emails = all.match(/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/g) ?? [];
  const unexpected = [...new Set(emails)].filter((e) => !ALLOWED_EMAILS.has(e));
  if (unexpected.length > 0) problems.push(`cast contains email address(es): ${unexpected.join(', ')}`);
  if (/\b[A-Fa-f0-9]{32,}\b/.test(all)) problems.push('cast contains long hex runs (possible key material)');
  if (/\b[A-Za-z0-9+/]{44,}={0,2}\b/.test(all)) problems.push('cast contains long base64 runs (possible key material)');

  if (lastT > 90) problems.push(`last event at ${lastT}s exceeds the 90s cap`);
  if (events < 15) problems.push(`only ${events} events`);

  // The expected command sequence: the offline commands must be present in
  // both the CI and the local flavor of the cast.
  const required: Array<[string, RegExp]> = [
    ['grr --version output', /grr \d+\.\d+\.\d+/],
    ['grr api list JSON', /gmail\.users\.messages\.list/],
    ['grr api describe JSON', /discoveryRevision/],
    ['grr gmail dry-run request JSON', /https:\/\/gmail\.googleapis\.com\/gmail\/v1\/users\/me\/messages/],
    // The cast header line, as emitted by grr schema's JSON: tracks WIDTH so
    // a geometry change never silently breaks this gate.
    ['grr schema dump', new RegExp(`"version":2,"width":${WIDTH}`)],
    ['grr --help head', /Google tools from the terminal/],
  ];
  for (const [name, re] of required) {
    if (!re.test(all)) problems.push(`expected command output missing: ${name}`);
  }
  const optional: Array<[string, RegExp]> = [
    ['grr transport (HTTP/3 facts)', /negotiated_protocol/],
    ['grr ask plan (System One)', /api\.typesafe\.ai/],
    ['opencode agent segment', /opencode run "/],
  ];
  const present = optional.filter(([, re]) => re.test(all)).map(([name]) => name);

  console.log(`validate ${path}`);
  console.log(`  header: asciicast v2, ${WIDTH}x${HEIGHT}, env.SHELL present`);
  console.log(`  events: ${events}, last at ${r3(lastT)}s`);
  console.log(`  offline command sequence: ${required.every(([, re]) => re.test(all)) ? 'present' : 'INCOMPLETE'}`);
  console.log(`  credential-backed segments present: ${present.length > 0 ? present.join(', ') : 'none (CI flavor)'}`);
  console.log(`  agent segment (input events): ${inputHasAgent ? 'present' : 'absent'}`);
  if (problems.length > 0) {
    for (const p of problems) console.error(`  FAIL: ${p}`);
    console.error(`validate ${path}: FAILED`);
    return false;
  }
  console.log(`  PASS: well-formed asciicast v2, expected sequence present, no private data`);
  return true;
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

function fmtKB(bytes: number): string {
  return bytes >= 1024 ? `${(bytes / 1024).toFixed(1)} KB` : `${bytes} B`;
}

async function main(): Promise<number> {
  const args = process.argv.slice(2);
  let local = false;
  let ciRecord = false;
  let outPath = DEFAULT_OUT;
  let validatePath: string | null = null;
  for (let i = 0; i < args.length; i++) {
    const a = args[i];
    if (a === '--local') {
      local = true;
    } else if (a === '--ci-record') {
      ciRecord = true;
    } else if (a === '--out' && i + 1 < args.length) {
      outPath = resolve(ROOT, args[++i]);
    } else if (a === '--validate') {
      validatePath = i + 1 < args.length && !args[i + 1].startsWith('--') ? resolve(ROOT, args[++i]) : DEFAULT_OUT;
    } else {
      console.error(`unknown argument: ${a}`);
      console.error('usage: node scripts/generate-demo.ts [--local] [--ci-record] [--out <path>] [--validate [<path>]]');
      return 2;
    }
  }

  if (validatePath) {
    return validateCast(validatePath) ? 0 : 1;
  }

  const grrBin = findGrr();
  const mode: 'ci' | 'local' | 'ci-record' = ciRecord ? 'ci-record' : local ? 'local' : 'ci';
  console.log('demo generation report');
  console.log(`  grr binary: ${grrBin}`);
  let versionLine = '';
  try {
    versionLine = sanitize((await spawnCapture(grrBin, ['--version'], { timeoutMs: CMD_TIMEOUT_MS })).stdout.trim());
    // `--version` is a banner (semver first, then the mascot crab and the
    // tagline); only the `grr <semver>` line is the version proper, and it
    // is the only part that belongs in the closing note below.
    const semverLine = versionLine.match(/^grr \d+\.\d+\.\d+$/m)?.[0];
    if (semverLine) versionLine = semverLine;
    console.log(`  grr version: ${versionLine}`);
  } catch (e) {
    console.error(`  FAIL: grr is not runnable (${e instanceof Error ? e.message : String(e)})`);
    return 1;
  }
  console.log(`  mode: ${mode}${local ? '' : ' (offline/dry-run only)'}`);

  const { segs, reports } = await buildSegments(mode);

  const frames: Frame[] = [
    { kind: 'note', text: '# every line below is real output — recorded offline, no credentials needed' },
  ];
  let localNotePushed = false;
  for (const seg of segs) {
    if (mode === 'local' && seg.mode === 'local' && seg.kind === 'command' && !localNotePushed) {
      frames.push({ kind: 'note', text: '# credential-backed: real HTTP/3, real natural-language plan' });
      localNotePushed = true;
    }
    if (seg.kind === 'agent') {
      frames.push({ kind: 'note', text: '# an opencode agent driving grr — reasoning + tool calls, plain text' });
    }
    frames.push({ kind: 'seg', seg });
  }
  const ran = reports.filter((r) => r.status === 'ran');
  const closing = `# grr ${versionLine.replace(/^grr /, '')} — ${ran.length} segments recorded by scripts/generate-demo.ts`;

  const { events, duration } = buildCast(frames, closing);
  writeCast(outPath, events, duration);
  const size = statSync(outPath).size;

  for (const r of reports) {
    console.log(`  [${r.status}] ${r.display} — ${r.lines} line${r.lines === 1 ? '' : 's'} (${r.note})`);
  }
  console.log(`  opencode: ${reports.some((r) => r.kind === 'agent' && r.status === 'ran') ? 'agent segment captured' : 'agent segment absent (see above)'}`);
  console.log(`  wrote ${outPath} (${events.length} events, ${r3(duration)}s, ${fmtKB(size)})`);
  return 0;
}

main().then(
  (code) => {
    process.exit(code);
  },
  (e) => {
    console.error(e instanceof Error ? e.stack ?? e.message : String(e));
    process.exit(1);
  },
);
