# demo/

An [asciinema](https://asciinema.org) recording of the real `grr` CLI —
`demo.cast` — for the repo and the site's terminal embed. Every command in it
is executed for real by `scripts/generate-demo.ts`, which captures stdout,
sanitizes it, and renders the frames with deterministic inter-line timing. The
output is real; only the pacing is synthetic.

## What's in the recording

- **Offline segment** (works with no credentials, regenerates anywhere):
  `grr --version`, `grr api list`, `grr api describe`, the
  `grr gmail users messages list --user-id me --dry-run` request envelope,
  `grr schema`, `grr --help`.
- **Credential-backed segment** (`--local` only): `grr transport` (HTTP/3 +
  runtime features) and `grr ask` (natural language → a typed method + params
  via a System One model).
- **Agent segment** (`--local` only): an `opencode run` transcript in which
  the [opencode](https://opencode.ai) agentic harness drives grr end to end —
  tool calls with their output, rendered to plain text from the agent's raw
  `--format json` event stream.

## Regenerating locally

```sh
node scripts/generate-demo.ts --local        # the committed demo.cast
node scripts/generate-demo.ts                # CI mode: offline commands only
node scripts/generate-demo.ts --validate     # assert the .cast is well-formed
```

- `--local` needs a Google token (for `grr transport`), a **TypesSafe API key**
  (for `grr ask` planning) and `opencode` on PATH (for the agent segment). If
  opencode is missing, fails or times out, the agent segment is skipped and
  the skip is recorded in the generation report — agent output is never
  fabricated.
- The generator prepends the current build's directory (`target/debug`) to the
  opencode child's PATH, so the agent's `grr` resolves to the current binary,
  not whatever stale copy is first on PATH.
- After regenerating, commit the new `demo.cast` — that is the only way the
  committed recording changes.

## Why the CI workflow does not regenerate

`.github/workflows/demo.yml` **validates only**: it parses the committed
`demo.cast` as asciicast v2 JSON lines, asserts the expected offline command
sequence, asserts there is no private data in it (no `Bearer`, no key
material, no email address beyond `me`), re-renders a throwaway cast with the
published grr binary to prove the offline pipeline still works, and
smoke-plays the recording with real asciinema. It never opens a pull request:
in CI the agent segment would be absent, so a regenerated committed cast would
differ by exactly that missing segment — a regression. The committed
`demo.cast` is refreshed only by local `--local` runs.

## Privacy

The agent prompt is deliberately self-describing (transport, ask plan,
dry-run) rather than a mailbox read: a demo that quoted real email subjects or
sender addresses would leak private data, and this one cannot. The sanitizer
additionally redacts any email address, Bearer token and long hex/base64 run
from every captured line, and `--validate` asserts none of them are present.
