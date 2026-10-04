<p align="center"><img src="assets/logo-wordmark.svg" width="420" alt="grr — Google Rust Rewrite"></p>
<p align="center"><img src="assets/favicon.svg" width="32" height="32" alt="grr favicon"></p>

# grr

[![CI](https://github.com/debanjanbasu/grr-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/debanjanbasu/grr-cli/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![crates.io](https://img.shields.io/crates/v/grr-cli.svg)](https://crates.io/crates/grr-cli)

**Google tools from the terminal, at maximum performance.** `grr-cli` is one published Rust package built around the `grr` command-line binary. Gmail, Calendar, Drive, People, Chat, Forms, Tasks, Docs, Sheets, and Slides share one OAuth login and one command tree — generated straight from Google's own Discovery Service — while stdout stays clean and machine-readable.

Project site: [grr-cli.pages.dev](https://grr-cli.pages.dev/) · [Privacy](https://grr-cli.pages.dev/privacy/)

grr is an independent project and is not affiliated with or endorsed by Google.

```sh
grr auth login
grr gmail users messages list --user-id me --q "in:inbox" --max-results 5
grr drive files list --page-size 10
grr api describe calendar.events.list
grr schema
```

## Install

**Prebuilt binaries** — macOS arm64, Linux x86_64, Windows x86_64, and Windows on ARM (aarch64, Snapdragon X / Oryon):
[GitHub Releases](https://github.com/debanjanbasu/grr-cli/releases)

Release binaries are zero-config — an OAuth client is compiled in, so `grr auth login` works immediately. Archives are `.tar.zst` (zstd level 22) on unix and `.zip` on Windows; every binary is UPX-packed.

**From source** (the default CLI build requires Rust nightly — see [Development](#development)):

```sh
cargo install --git https://github.com/debanjanbasu/grr-cli --locked
```

or from a local clone at the repository root:

```sh
git clone https://github.com/debanjanbasu/grr-cli
cd grr-cli
cargo install --path . --locked
```

A source build has no OAuth client compiled in — see [Authentication and configuration](#authentication-and-configuration).

**Package managers**:

```sh
winget install debanjanbasu.grr       # Windows
cargo install grr-cli                 # crates.io
```

Homebrew (macOS + Linux), from the `debanjanbasu/tap` tap:

```sh
brew tap debanjanbasu/tap
brew trust debanjanbasu/tap
brew install grr
```

`brew trust` is required, not decorative: since Homebrew 4.4 third-party taps are untrusted by default, and `brew install` refuses to install a formula from an untrusted tap until you trust it once. Skipping that line is the most common first-run failure.

The crates.io CLI build needs nightly Rust and `RUSTFLAGS="--cfg reqwest_unstable"` for HTTP/3; the prebuilt releases avoid that source-build step. `cargo install` also produces a binary with no embedded OAuth client, so budget five minutes for `grr auth setup` (or a `.env`) on that path.

## 60-second quickstart

```sh
brew tap debanjanbasu/tap
brew trust debanjanbasu/tap
brew install grr

grr auth login
grr gmail users getProfile --user-id me
```

There is nothing to configure first: a release binary already carries an OAuth client, so `grr auth login` opens the browser, you consent, and every service works. Headless machine? `grr auth login --device` prints a URL + code instead of opening a browser.

Building from source instead of installing a release? Read [Authentication and configuration](#authentication-and-configuration) before your first `grr` invocation.

## Authentication and configuration

**Release binaries are zero-config.** `build.rs` reads `GRR_CLIENT_ID` / `GRR_CLIENT_SECRET` from the build environment — GitHub Actions repo secrets for the official builds — and compiles them in, so GitHub Releases, Homebrew, and winget installs need no config file and no console visit.

**Source builds bring their own client.** `cargo install grr-cli` compiles the published sources with nothing in the build environment, so that binary has no embedded client. Any one of these fixes it:

1. **A `.env` next to `Cargo.toml`** — copy [`.env.example`](.env.example) to `.env`, fill in both values, rebuild. `build.rs` reads the environment first and `.env` second, so every build after that is zero-config too. `.env` is gitignored; never commit it.
2. **Environment variables for a single build** — `GRR_CLIENT_ID=… GRR_CLIENT_SECRET=… cargo build --release`.
3. **`grr auth setup`** — no rebuild required, because it writes `~/.grr/config.toml` (Windows: `%USERPROFILE%\.grr\config.toml`) instead.

### `grr auth setup`

```text
Create and store an OAuth client. Release builds ship with one, so this is only needed when using
your own client

Usage: grr auth setup [OPTIONS]

Options:
      --client-id <CLIENT_ID>          Google OAuth client id (…apps.googleusercontent.com).
                                       Prompted for when omitted
      --client-secret <CLIENT_SECRET>  Google OAuth client secret. Prompted for when omitted
      --print-only                     Print the instructions and the resolved path, but write
                                       nothing. Useful for a first look, and for agents that only
                                       want the recipe
      --force                          Overwrite an existing ~/.grr/config.toml instead of refusing
      --enable-apis                    Try `gcloud services enable` for the APIs when gcloud is on
                                       PATH
  -f, --format <FORMAT>                Output format [default: json] [possible values: json, jsonl,
                                       table, pretty]
      --readonly                       Refuse every write method (POST/PATCH/PUT/DELETE); only reads
                                       run
      --deny-service <SERVICE>         Refuse every method of this service (repeatable)
      --deny-verb <VERB>               Refuse this HTTP verb (repeatable)
  -h, --help                           Print help
```

It checks the client-id shape before writing anything (must end in `.apps.googleusercontent.com`, and a truncated paste is rejected), rejects a truncated secret, writes the file mode `0600` on unix, and refuses to clobber an existing `config.toml` without `--force`. It prints the exact Cloud Console URLs and the `gcloud services enable` line covering every API behind a command — gmail, calendar-json, drive, people, chat, forms, tasks, docs, sheets, slides, script, analyticsadmin, analyticsdata, and searchconsole `…googleapis.com` — and `--print-only` hands you that recipe without touching disk, which is the right first command when you have no client at all:

```sh
grr auth setup --print-only
```

Because it runs before any client is resolved, `grr auth setup` works even when nothing is configured yet: it is the escape hatch, not a last resort. Under a non-TTY stdin (CI, agents, pipes) it errors instead of hanging, so pass `--client-id` and `--client-secret` explicitly rather than relying on the prompts.

**Resolution order at runtime:** `GRR_OAUTH__CLIENT_ID` / `GRR_OAUTH__CLIENT_SECRET` → `~/.grr/config.toml` → the client compiled into the binary (env wins over the file; the compiled-in client is the fallback). `GRR_CONFIG_PATH` moves the file; `RUST_LOG` sets the log level.

Embedding a client secret is acceptable here because Google treats installed-app client secrets as non-confidential — the flow is protected by PKCE, which is always on. The secret lives in GitHub Actions repo secrets and is compiled in at release time; it must never enter the repository or a CI log.

To create a client from scratch, follow [docs/gcp-setup.md](docs/gcp-setup.md) — you only need that if you are building from source or deliberately want your own client.

The project/fork is **Google Rust Rewrite**; the Google consent-screen application is named **Rust Rewrite**. The consent screen is where that shorter name appears.

**0.4 re-consent:** if you used a pre-0.4 token, run `grr auth login` again. The new service permissions include `chat.delete`, `chat.memberships`, `chat.messages.reactions`, and `contacts.other.readonly`; an existing grant does not pick them up automatically.

## Usage highlights

```sh
grr gmail users messages list --user-id me --q "from:github.com" --max-results 10
grr gmail users messages get --user-id me --id 191f8ab2 --param-format metadata
grr gmail users threads list --user-id me --max-results 10
grr calendar events list --calendar-id primary --time-min 2026-10-01T00:00:00Z --max-results 10
grr drive files list --q "name contains 'report'" --page-size 10
grr drive files export --file-id <id> --mime-type application/pdf
grr chat spaces messages create --parent spaces/AAAA --body-file ./message.json
grr sheets spreadsheets values get --spreadsheet-id 1AbC… --range Sheet1!A1:B10
grr tasks tasklists list
grr transport
grr schema
```

Every command takes `-f/--format json|jsonl|table|pretty` (default `json`). For `gmail users messages get`, the Gmail MIME parameter named `format` arrives as `--param-format full|metadata|minimal|raw` (see the collision rule below), while `-f/--format` still selects the printed output format. Logs go to stderr, so stdout is always parseable:

```sh
grr gmail users messages list --user-id me --q "in:inbox" --max-results 1 | jq -r '.[0].id'
```

`grr --version` prints the semver on its first line (scripts depend on that), then the crab mascot: in full 24-bit-colour pixel art on a terminal that advertises `COLORTERM`/direct colour, and as plain ASCII everywhere else — a pipe, `TERM=dumb`, or `NO_COLOR`. Set `CLICOLOR_FORCE=1` to force the colour logo through a pipe (`grr --version | less -R`); `NO_COLOR` still wins over it.

### Command reference

The command tree is **generated** from the committed Discovery index at build time, so this table is deliberately coarse: **`grr --help` and `grr schema` are the source of truth**, and every one of the 401 methods is a real, typed command.

| Group | What it covers | Methods |
| --- | --- | --- |
| `grr auth` | `login [--device] [--account name]`, `status [--account name]`, `setup [--client-id] [--client-secret] [--print-only] [--force] [--enable-apis]` | — |
| `grr ask` | `"<request>" [--run] [--service] [--method]` — natural language to a typed method + params via a System One model; prints the plan by default | - |
| `grr mcp` | an MCP (JSON-RPC over stdio) server exposing every method as a typed tool; `--readonly` for read-only | - |
| `grr api` | `list [--service X] [--filter substr] [--grouped]`, `describe <id>`, `call <id> …`, `refresh [--service X]` — every method by id, the flat escape hatch | 401 |
| `grr schema` | the whole command tree as JSON | — |
| `grr transport` | negotiated HTTP version + runtime features | — |
| `grr skills` | `install [--dir PATH] [--claude] [--force]`, `list` — the 16 packaged agent skills into the user-level skills directory (`~/.agents/skills`, offline; `--claude` mirrors to `~/.claude/skills`) | — |
| `grr gmail` | messages, threads, drafts, labels, history, attachments, filters, forwarding, POP/IMAP, send-as, CSE, delegates, watches | 79 |
| `grr calendar` | calendars, events, instances, ACL, free/busy, colors, settings | 38 |
| `grr drive` | files, permissions, comments, replies, revisions, changes, drives, apps, approvals | 64 |
| `grr people` | contacts, connections, contact groups, other contacts, directory people | 24 |
| `grr chat` | spaces, members, messages, reactions, media, custom emoji, read state | 54 |
| `grr forms` | form bodies, responses, watches, publish settings | 10 |
| `grr tasks` | tasklists and the tasks inside them | 14 |
| `grr docs` | documents `get` / `create` / `batchUpdate` | 3 |
| `grr sheets` | spreadsheets, values, batch operations, developer metadata | 17 |
| `grr slides` | presentations, pages, thumbnails | 5 |
| `grr script` | script projects, deployments, versions, processes, running functions | 16 |
| `grr analyticsadmin` | accounts, properties, data streams, custom dimensions and metrics, conversion events | 55 |
| `grr analyticsdata` | standard, realtime, pivot and batch reports, audience exports | 11 |
| `grr searchconsole` | search analytics queries, sitemaps, sites, URL inspection | 11 |

**Safety profiles.** `--readonly`, `--deny-service <name>` and `--deny-verb <VERB>` are global flags honored by the generated tree, `grr api call` and `grr mcp` alike — they parse before or after the subcommand, and turn destructive methods into actionable errors:

```sh
grr --readonly gmail users messages delete --user-id me --id abc   # refused: DELETE
grr --deny-service chat chat spaces list                            # refused: the chat service
```

## The generated tree

Every service command above is generated at build time — there are no hand-written per-service commands and no hand-written per-service clients. `scripts/generate-commands.ts` reads the committed Discovery index (`src/discovery/*.json`, ~420 KiB across the 14 services) and emits the whole tree into `src/commands/generated.rs` — 401 leaves, 994 typed flags, using clap's builder API. A daily [workflow](.github/workflows/discovery.yml) refetches Google's Discovery Service, regenerates the index along with the command tree, the per-service agent skills, and the site's coverage table, and opens a PR, so new API surface reaches you without waiting for a grr release. The tree and the index are generated artifacts: never hand-edit them.

The rules, so you can predict any command without memorizing it:

- **The naming rule.** A leaf mirrors its Discovery method id: `gmail.users.messages.list` → `grr gmail users messages list`. Resources nest as subcommands; each leaf also carries its bare method name as a visible alias (`list`, `get` — camelCase methods keep their casing, e.g. `getProfile`).
- **Typed flags per method.** Parameter names come from the same ids: `userId` → `--user-id`, `maxResults` → `--max-results`. Integers are parsed as `i64`, booleans are presence flags, repeated parameters repeat (`--label-ids a --label-ids b`), enum parameters validate their values, and required parameters are enforced by clap.
- **The collision rule.** A parameter named `format`, `query`, `params`, `body-file`, `dry-run`, or `help` would collide with the shared escape hatches, so it is exposed as `--param-format` / `--param-query` / … (see `RESERVED_FLAG_IDS` in the generator).
- **Untyped bodies.** Discovery's request schemas are not part of the index, so `POST`/`PATCH`/`PUT` bodies pass through `--params <JSON>` (merged; typed flags win) or `--body-file <PATH|->` verbatim.
- **Every leaf also carries the escape hatches:** `--params <JSON>`, `--body-file`, repeatable `--query KEY=VALUE`, `--dry-run`, and `-f json|jsonl|table|pretty`.

Dispatch resolves the leaf's id against the embedded index and funnels into **one shared call path** — the same engine `grr api call` uses. The two routes are interchangeable:

```sh
grr api call gmail.users.messages.list --param userId=me --dry-run
grr gmail users messages list --user-id me --dry-run
# byte-identical output
```

`grr api` stays as the flat escape hatch: `list [--service] [--filter] [--grouped]` to browse, `describe <id>` for parameters and scopes, `call <id>` to invoke, `refresh [--service]` to pull the index forward between releases. Because the index is embedded rather than fetched, both `grr api list` and `grr schema` work with no network at all.

Authorisation is checked per method: `grr auth login` consents to a fixed set of scopes, and each call compares the method's least-privilege scope against that set instead of silently escalating — so a method needing something you never granted prints a note naming the scope and may fail with a spelled-out `403` rather than an opaque error. Docs, Sheets, and Slides work through the `drive` scope grr already holds; **Tasks needs the `tasks` scope, which this build does not request**, so a Tasks call prints a note naming the scope and can come back `403`. Their APIs (`tasks.googleapis.com`, `docs.googleapis.com`, `sheets.googleapis.com`, `slides.googleapis.com`) must be enabled on your Cloud project too — see [docs/gcp-setup.md](docs/gcp-setup.md).

## Design philosophy

- **Zero-config.** Release binaries carry an OAuth client compiled in by `build.rs`, so a fresh install runs `grr auth login` with nothing to configure. `~/.grr/config.toml` is the override, not the prerequisite, and holds exactly one thing: an OAuth client ID (and optionally a secret). Scopes, redirect URI, pool sizes, timeouts, and retry policy are compile-time constants tuned for Google's frontends ([src/core/http.rs](src/core/http.rs)). `GRR_CONFIG_PATH` overrides the file location, `RUST_LOG` the log level (`GRR_OAUTH__*` env vars exist for headless overrides) — nothing else is configurable, on purpose.
- **Generated, namespaced services.** The command tree is compiled from the Discovery index — the same machine-readable description Google publishes — so method additions land as a daily PR instead of a hand-written backlog. Mail is `grr gmail …`; Calendar, Drive, People, Chat, Forms, Tasks, Docs, Sheets, and Slides live alongside it, and account-level concerns stay top-level (`grr auth`, `grr api`, `grr transport`, `grr schema`). One login covers every service.
- **stdout purity.** Logs go to stderr, results go to stdout, so `| jq` always works. `-f jsonl` streams arrays one object per line.
- **Keyring-first token storage.** Tokens live in the OS keyring (Windows Credential Manager, macOS Keychain, Linux Secret Service via D-Bus), with automatic fallback to `<cache dir>/grr/token.json` on headless systems. A token found in the fallback file auto-imports into the keyring on first sight.
- **Agent-first.** `grr schema` dumps the complete command tree as JSON with zero configuration — the machine-readable contract for AI agents, discoverable without touching a config file or scraping `--help`. One fast CLI replaces per-service MCP servers: no MCP setup, just `grr schema`. There is also a packaged agent skill at [skills/grr/SKILL.md](skills/grr/SKILL.md) — see [Agent skills](#agent-skills).

## Agent skills

grr ships a packaged agent skill: [skills/grr/SKILL.md](skills/grr/SKILL.md) — the discovery-first discipline (`schema` → `api list` → `api describe` → `--dry-run`), the method-id naming rule, and the output contract in one file, written for any AI agent. The core skill plus the 14 generated per-service skills are compiled into the binary, so `grr skills install` writes them into `~/.agents/skills/` with no network (Codex, opencode, Cursor, Gemini CLI, Copilot CLI and omp all read that location; `--claude` mirrors into `~/.claude/skills/` for Claude Code, which reads nothing else):

```sh
grr skills install            # all 16 into ~/.agents/skills/ (idempotent; identical files are a no-op)
grr skills install --claude   # also mirror into ~/.claude/skills/
grr skills install --force    # replace files that differ from the packaged copy
grr skills list               # the packaged set, and what each target directory holds
```

or install straight from the repository with the skills CLI:

```sh
npx skills add https://github.com/debanjanbasu/grr-cli
```

The site's [agents guide](https://grr-cli.pages.dev/docs/agents/) carries the same contract for humans and harness authors.

## Performance

- **HTTP/3 (QUIC) by default** — prior-knowledge h3 with one authenticated probe at startup and silent HTTP/2 fallback; `grr transport` shows what was actually negotiated.
- **Tokio multi-threaded runtime**, auto-sized to cores — no thread pool to tune.
- **In-flight request valve** — a semaphore (not a thread pool) caps concurrent HTTP requests at 64, staying under Gmail's per-user rate limits so bursts don't self-DOS into 429s. 429s are retried honoring `Retry-After` (waits capped at 30s); whole-request timeout is 30s.
- **Compression always on** — gzip, deflate, zstd, and brotli response decompression.
- **io_uring file I/O** is a Linux-only target-specific dependency, auto-detected at runtime; it is not a Cargo feature.

Measured startup, binary size, and request-latency numbers against the other Google Workspace CLIs live at [grr-cli.pages.dev/compare/](https://grr-cli.pages.dev/compare/), refreshed daily by an automated workflow.

## Architecture

`grr-cli` is one published crate at the repository root. `src/lib.rs` builds the library target `grr_cli`; `src/main.rs` is a thin wrapper over `src/cli.rs`, and the binary is named `grr`.

```text
.
├── src/
│   ├── core/                 # auth/device/oauth/server/store, http.rs,
│   │                         # config.rs, config_loader.rs, error.rs,
│   │                         # fs_io.rs, runtime.rs
│   ├── commands/             # auth.rs, api.rs, setup.rs, transport.rs,
│   │                         # skills.rs, mcp.rs, ask.rs (the static commands)
│   │                         # + generated.rs (GENERATED — the
│   │                         # whole service tree, ~470 KiB) and gen_dispatch.rs
│   │                         # (resolves leaf ids, funnels into the shared path)
│   ├── discovery.rs          # loader over the embedded index
│   ├── discovery/             # generated *.json index (~420 KiB, committed,
│   │                         # refreshed daily by workflow PR)
│   ├── schema.rs
│   ├── output.rs
│   ├── cli.rs
│   ├── lib.rs
│   └── main.rs
└── tests/                    # 5 integration test files
```

One shared core, no per-service client modules: the CLI speaks Discovery through a single call path (`src/commands/api.rs`), and the typed request/response models of the 0.3.x library era are gone. The repository also contains the `assets/`, `site/` (the Astro site at [grr-cli.pages.dev](https://grr-cli.pages.dev/), deployed from main by Cloudflare Pages), `packaging/`, and `scripts/` material used for the project site and distribution.

Every build requires Rust **nightly** and the `reqwest_unstable` cfg (`.cargo/config.toml` supplies it for in-repo builds; downstream users need `RUSTFLAGS="--cfg reqwest_unstable"`). There is deliberately no stable-Rust path. HTTP/3 (rustls + quinn, via reqwest's unstable http3 support) is **always compiled in**, and HTTP/2 exists only as a runtime fallback; io_uring is a Linux-only target-specific dependency that is detected at runtime.

## Development

grr requires Rust **nightly** — the build script fails with a clear message on any other toolchain. [rust-toolchain.toml](rust-toolchain.toml) pins it and supplies the components needed by the build, while [.cargo/config.toml](.cargo/config.toml) sets the `reqwest_unstable` cfg for HTTP/3.

```sh
cargo build
cargo test --locked
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
cargo run -- api list

node scripts/fetch-discovery.ts          # refresh the Discovery index
node scripts/generate-commands.ts        # regenerate the service command tree
node scripts/generate-changelog.ts       # regenerate CHANGELOG.md + site data
```

Each generator takes `--check` and exits 1 when its output is stale — that is the CI gate. Change the index or the generator, never `src/commands/generated.rs` by hand.

- Tests never touch real credentials — token paths are injected, and wiremock serves the API endpoints.
- `RUST_LOG=debug` traces requests; the default level mutes quinn_udp's harmless IPv6 warnings (`info,quinn_udp=error`).
- A local build needs an OAuth client before a live call can work: copy `.env.example` to `.env` and fill it in (compiled in by `build.rs`), or run `grr auth setup` to write `~/.grr/config.toml`. See [CONTRIBUTING.md](CONTRIBUTING.md#local-oauth-defaults).
- `cargo run -- api list` and `cargo run -- schema` need no client at all — the index is embedded.

## Packaging & status

| Channel | Install | Status |
| --- | --- | --- |
| GitHub Releases | 5-platform binaries (macOS arm64, Linux x86_64, Linux aarch64, Windows x86_64, Windows on ARM) built on `v*` tags, UPX-packed, `.tar.zst` on unix and `.zip` on Windows | **live — v0.7.0** — [releases](https://github.com/debanjanbasu/grr-cli/releases) |
| crates.io | `cargo install grr-cli` (binary installs as `grr`; needs nightly + `RUSTFLAGS="--cfg reqwest_unstable"` for the default CLI HTTP/3 build, and brings no embedded OAuth client) | **live — 0.7.0, one crate**. Trusted publishing uses OIDC (no stored API tokens) |
| winget | `winget install debanjanbasu.grr` | **not yet published** — the first submission is held on Microsoft's validation; manifests live in `packaging/winget/` |
| Homebrew | `brew tap debanjanbasu/tap && brew trust debanjanbasu/tap && brew install grr` (tap: [debanjanbasu/homebrew-tap](https://github.com/debanjanbasu/homebrew-tap), formula `Formula/grr.rb`, updated automatically on every release) | live (arm64 macOS + x86_64/arm64 Linux) |

The project publishes one package, `grr-cli`, whose binary is `grr`. `v*` tags trigger the release workflow, and crates.io publishing is handled through trusted publishing.

## License

MIT — see [LICENSE](./LICENSE).
