# demo/

An [asciinema](https://asciinema.org) recording of the real `grr` CLI —
`demo.cast` — for the repo and the site's terminal embed. Every command in it
is executed for real by `scripts/generate-demo.ts`, which captures stdout,
sanitizes it, and renders the frames with deterministic inter-line timing. The
output is real; only the pacing is synthetic.

## What's in the recording

- **Offline segments** (no credentials, regenerate anywhere): `grr --version`,
  `grr api list`, `grr api describe`, the `grr gmail users messages list
  --user-id me --dry-run` request envelope, `grr schema`, `grr --help`.
- **Ask segment**: `grr ask "show my unread messages"` — the natural-language
  plan (a System One model resolving to a typed method + params). Records live
  wherever a `TYPESAFE_API_KEY` is available; skipped gracefully where not.
- **Transport segment**: `grr transport` (HTTP/3 + runtime features). Needs a
  live Google token, so it records on `--local` runs and is **carried forward**
  unchanged on CI regenerations.
- **Agent segment**: an `opencode run` transcript in which the
  [opencode](https://opencode.ai) agentic harness drives grr end to end — tool
  calls with their output, rendered to plain text from the agent's raw
  `--format json` event stream. Recorded live when opencode works (locally, or
  in CI with its free default models); otherwise carried forward.

## Regenerating

```sh
node scripts/generate-demo.ts --local     # full recording: everything live
node scripts/generate-demo.ts --ci-record # what CI runs (see below)
node scripts/generate-demo.ts --validate  # assert the .cast is well-formed
```

- `--local` needs a Google token (transport), a **TypeSafe API key** (the ask
  plan) and `opencode` on PATH (the agent segment). Anything unavailable is
  skipped — or carried forward from the committed cast — and recorded in the
  generation report. Agent output is never fabricated.
- `--ci-record` is the self-maintenance mode the
  [demo workflow](../.github/workflows/demo.yml) runs on every push that
  touches the CLI surface, plus weekly: it builds the binary from source,
  re-records the cast, validates it (privacy assertions included), and opens a
  PR when it changed. Segments CI cannot reproduce — the transport probe, the
  agent run — are **carried forward from the committed cast**, so a CI
  regeneration can never strip them from the site; only a fresh `--local` run
  refreshes their content.
- The generator prepends the current build's directory to the opencode child's
  PATH, so the agent's `grr` resolves to the current binary, not whatever
  stale copy is first on PATH.

## Privacy

The agent prompt is deliberately self-describing (transport, ask plan,
dry-run) rather than a mailbox read: a demo that quoted real email subjects or
sender addresses would leak private data, and this one cannot. The sanitizer
additionally redacts any email address, Bearer token and long hex/base64 run
from every captured line, and `--validate` asserts none of them are present —
the CI PR fails closed on a violation.
