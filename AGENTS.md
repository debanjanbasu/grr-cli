# grr — Agent Execution Ruleset

`grr` is a Rust CLI for Google Workspace: one binary (`grr`), one crate (`grr-cli`), a generated command tree covering **401 methods across 14 Google APIs** (Gmail, Calendar, Drive, People, Chat, Forms, Tasks, Docs, Sheets, Slides, Apps Script, Analytics Admin, Analytics Data, Search Console) — compiled from the committed Discovery index, never hand-written — plus the id-based `grr api` escape hatch and static commands (`auth`, `api`, `mcp`, `transport`, `schema`, `ask`, `skills`). HTTP/3 (QUIC) is always on. Requires **Rust nightly**.

Use this file as the execution ruleset. Deep detail lives in the files linked from it.

## Task Routing Matrix (read before editing)

| If your task touches... | Read first |
|---|---|
| The service command tree (`grr gmail ...`, all 14 services) | `scripts/generate-commands.ts` + `src/commands/generated.rs` (GENERATED — never edit by hand) + `src/commands/gen_dispatch.rs` (dispatch) |
| OAuth client, config resolution, `.env`, `auth setup` | `build.rs`, `src/core/config.rs`, `src/core/config_loader.rs`, `.env.example` |
| `grr api` / discovery (methods, scopes, URLs) | `src/discovery.rs`, `src/discovery/*.json`, `scripts/fetch-discovery.ts` |
| Release binaries, UPX, archives, targets | `.github/workflows/release-binaries.yml`, `.cargo/config.toml`, `Cargo.toml [profile.*]` |
| MCP / ask / safety | `src/commands/{mcp,safety,ask}.rs` — the MCP server, the safety gate (global args), the System One ask flow |
| The website | `site/README.md` if present, else `site/astro.config.mjs` + `site/src/pages/` |
| The pixel-art assets (`site/src/assets/*.svg`, `site/public/*.svg`, favicons, og card) | `site/scripts/generate-mascot.mjs` — GENERATED, never hand-edit; run `node site/scripts/generate-mascot.mjs` |
| The `grr --version` banner | `src/logo.rs` — the pixel map, palette, half-block renderer, and terminal-capability gate |
| Docs / changelog automation | `scripts/generate-changelog.ts`, `.github/workflows/changelog.yml`, `.github/workflows/discovery.yml` |
| The agent skill (`skills/grr/SKILL.md`) | `src/commands/skills.rs` (the embedded table + install/list), `scripts/generate-skills.ts` (the generated service skills and the README template). Keep it in lockstep with the output contract and naming rules in `gen_dispatch.rs` and `generated.rs` |
| Crates.io publishing | `.github/workflows/publish.yml` (read the comment block first) |

## Key commands

```sh
cargo build --release                   # release build (nightly required)
cargo test                              # full suite
cargo clippy --all-targets -- -D warnings   # enforced: zero warnings
cargo fmt --all -- --check
cargo package --allow-dirty             # must succeed with NO env set

node scripts/fetch-discovery.ts        # refresh discovery index
node scripts/fetch-discovery.ts --check    # exit 1 if upstream moved (network; the daily workflow, not CI, is the gate)
node scripts/generate-commands.ts      # regenerate the service command tree
node scripts/generate-commands.ts --check   # exit 1 if stale (CI gate, offline)
node scripts/generate-skills.ts        # regenerate skills/<service>/SKILL.md + skills/README.md
node scripts/generate-skills.ts --check    # exit 1 if stale (CI gate, offline)
node scripts/generate-coverage.ts      # regenerate the site's coverage table
node scripts/generate-coverage.ts --check   # exit 1 if stale (CI gate)
node scripts/generate-winget.ts v0.8.1      # regenerate the winget submission (downloads the release; not a CI gate)
node scripts/generate-changelog.ts     # regenerate CHANGELOG.md + site data
node scripts/generate-changelog.ts --check # idempotency gate
npm run demo                             # regenerate demo/demo.cast (local: + opencode segment)

grr schema                                  # the full command tree as JSON — the contract
grr api list [--service X] [--filter SUBSTR] # the 401 methods, offline, no login
grr skills install [--claude] [--force]      # the packaged agent skills into ~/.agents/skills, offline
grr skills list                              # what is installed, per target directory
```

## Architecture

| Layer | Files | What lives there |
|---|---|---|
| CLI | `src/cli.rs`, `src/commands/*.rs` | Static commands (auth, api, mcp, transport, schema, ask, skills) as derive types; the entire service tree is `src/commands/generated.rs` (compiled from the index by the generator, dispatched by `gen_dispatch.rs` through the shared call path in `api.rs`). `auth setup`, `schema` and `skills` run before config load (they must work with zero configuration; `skills` embeds its payload with `include_str!`). |
| Discovery | `src/discovery.rs` | The embedded index + `grr api` resolution: method ids, path templates, scopes, params. Parsed once into a `OnceLock`. This index is the single source of truth for BOTH surfaces — the generated tree is compiled from it. |
| Core | `src/core/` | Auth (OAuth+PKCE, keyring), HTTP (HTTP/3), config, errors, runtime probes (io_uring availability). |
| Build | `build.rs`, `.cargo/config.toml` | Nightly guard + compile-time OAuth client injection; build-std + per-target rustflags. |
| Site | `site/` | Astro static site: docs, changelog, llms.txt. Every image is generated pixel art from `site/scripts/generate-mascot.mjs`. |
| Skills | `skills/grr/SKILL.md`, `src/commands/skills.rs` | The packaged agent skills — discovery-first discipline, naming rule, output contract. `skills.rs` embeds all 16 files (`include_str!`) and serves `grr skills install` / `grr skills list`; installs are global-only (`~/.agents/skills`, plus `~/.claude/skills` under `--claude`). |

### Invariants worth knowing before you change something

1. **A `[target.*]` rustflags table REPLACES `[build]` rustflags** — cargo does not merge them. The three `--cfg` flags (`reqwest_unstable`, `hyper_unstable_ffi`, `tokio_unstable`) must be repeated in every target table, or crates gating on them fail to compile on that target.
2. **The discovery index is committed on purpose** (`src/discovery/*.json`, ~430 KiB across the 14 services). Do not move it to runtime-only fetch: that breaks offline use, adds first-run latency, and sacrifices determinism. Refresh via the script; the daily workflow opens a PR.
3. **The command tree is generated** — never hand-edit `src/commands/generated.rs`. Change the generator (`scripts/generate-commands.ts`) or the index (`scripts/fetch-discovery.ts`), then regenerate; CI gates both with `--check`. The daily discovery PR regenerates the tree, the skills, and the site coverage table in the same commit as the index, so nothing derived from it can drift.
4. **Method ids include resource names**: `users.messages.list`, not `messages.list`. The distiller walks `Object.entries(doc.resources)` and passes the name as the path prefix — and the CLI command mirrors the id verbatim (`grr gmail users messages list`), resource segments included.
5. **`basePath` is inconsistent across Google's docs** (`gmail/v1/` vs `/drive/v3/`). `Service::base()` strips the leading slash — `rootUrl` always ends in `/`, so a naive concat yields `//` and a 404.
6. **Per-method scope escalation is the design**: 124 unique scopes across the 14 services vs Google's ~25-scope cap on unverified apps means "request everything" fails at consent. A method needing a scope outside the consented set gets a stderr note, not a silent escalation; a resulting 403 names the scope.
7. **The OAuth client must never enter the repo** — not in source, tests, CI logs, or the `.crate` tarball. `build.rs` reads `GRR_CLIENT_ID`/`GRR_CLIENT_SECRET` from the environment (falling back to a repo-root `.env`) and re-exports via `cargo:rustc-env`. Release binaries embed them (Google treats installed-app secrets as non-confidential; PKCE protects the flow); source builds fall through to `grr auth setup`.
8. **`panic = "immediate-abort"`** in `[profile.release]` (gated by `panic-immediate-abort` in `.cargo/config.toml [unstable]`, paired with `build-std`). Panic messages become context-free; do not write tests that assert on panic text.
9. **stdout is machine output, stderr is logs** — `grr gmail users messages list --user-id me | jq` must never receive log lines. The known plain-text stdout exceptions are `grr transport`, `grr skills install` (a human install summary; `grr skills list` is JSON), and the pre-JSON device-login line; note any new one in docs when you add it.
10. **Global args are IDs, not names**: a `global(true)` arg's ID must be the FIELD name (`deny_service`), not the long flag (`deny-service`) - the hyphenated variant is the exact "Mismatch between definition and access" clap panic. And a flag may only be declared ONCE in the tree: a second declaration with the same ID panics arg-matching.
11. **The demo cast is generated** (`scripts/generate-demo.ts`), never hand-edited; the committed cast is recorded with `--local` on the owner's machine (CI validates via demo.yml and opens PRs from `--ci-record` runs with carry-forward), so the opencode agent segment cannot silently regress. The committed cast must never contain private data: no Bearer tokens, no client ids, no real message subjects.
12. **Counts in the site are derived, never typed.** `site/src/data/discovery-coverage.ts` is generated by `scripts/generate-coverage.ts` from the index, and every page that prints a method or service count imports `discoveryMethodTotal` / `discoveryServiceCount` from it. Hardcoding "401 methods" or "14 APIs" in an `.astro` file reintroduces the drift the generator exists to remove — the counts in prose belong in the coverage table.
13. **Site graphics are generated pixel art, never vector.** Every mascot pose, service illustration, glyph, favicon and the og card comes from `site/scripts/generate-mascot.mjs`, which authors them on character grids and emits run-length-merged, axis-aligned `<path>` runs. Two rules hold the style together: no curve commands and no blurred shadows anywhere in the set — a `Q`/`C` or a `<filter>` in an asset means it slipped back into vector, and a shadow is a Bayer-dithered *strip*, not an ellipse (a flat ellipse can only vary along x and renders as a row of bars). The CLI banner in `src/logo.rs` embeds the same mascot map and the same palette, so the terminal and the site are one brand.

## Rust coding standards

- **No locking, ever.** Never pass `--locked` and never `npm ci`/`--frozen-lockfile`; `Cargo.lock` is not tracked. Every build resolves the latest semver-compatible versions — newest dependency over frozen one, majors applied in the manifest, and the gates are the only correctness arbiter. This is a deliberate trade of reproducibility for currency: a regression from a fresh dependency is fixed by a revert or a bump, not by pinning.
- **Lazy by default.** No incidental work on the hot path: credentials (core/auth reads the keychain at first credential use, never at construction), caches, probes, and upgrade checks (the skills migration short-circuits on a version-compare before hashing anything) run only when the requested operation actually needs them. A command that does not touch a resource must not read, hash, probe, or migrate it.
- Conventional commits (`feat:`, `fix:`, `perf:`, `feat!:` for breaking). The changelog generator parses them — a malformed subject lands under "Other" instead of its proper section.
- Comments explain **why**, not what. Every non-obvious invariant gets one.
- `thiserror` for typed domain errors, `anyhow` at command boundaries.
- Tests never need real credentials: use dummy strings (`test-id.apps.googleusercontent.com`), runtime-env injection, or `#[cfg(test)]` seams. `GoogleAuth::with_token` is the test hook that skips OAuth entirely.
- `cli` is the only cargo feature; `--features cli` (and the default build) must compile warning-free. The per-service features of the 0.3.x library era are gone.
- No `#[allow(dead_code)]` — wire it up or delete it.

## Verification gate (all must pass before you claim done)

```sh
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
cargo package --allow-dirty      # with NO GRR_* env and no .env present
node scripts/generate-commands.ts --check   # generated tree in sync with the index
node scripts/generate-skills.ts --check     # agent skills in sync with the index
node scripts/generate-coverage.ts --check   # site coverage table in sync with the index
```

For site changes: `npm run build`, `npm run lint`, `npm run typecheck` in `site/`, plus a Lighthouse pass for anything user-facing. The hard-preserve strings in `site/src/layouts/BaseLayout.astro` (title pattern, JSON-LD name/alternateName, `og:site_name`, the `google-site-verification` meta) must survive any edit verbatim.

## Automation

| Workflow | Trigger | What it does |
|---|---|---|
| `ci.yml` | push/PR to main, weekly Mon 04:00 UTC | nextest/clippy/fmt on ubuntu+windows, `cargo package --allow-dirty` (no-env publish/docs.rs gate), scripts typecheck, **site `astro check` + `astro build`**, generated-artifact gates, cli-only feature-subset clippy, weekly rustsec audit (advisory-only: `continue-on-error`; the job runs `cargo generate-lockfile` first, because `Cargo.lock` is untracked and `cargo audit` needs a lock to read) |
| `release-binaries.yml` | tag `v*` | 5 targets (linux x86_64, linux aarch64, macOS arm64, Windows x86_64, Windows on ARM — the ARM legs on native ARM runners), UPX `--best --lzma`, zstd-22 `.tar.zst` + max-deflate `.zip` (archive members are `grr`/`grr.exe`, never the target name), SHA256SUMS, GitHub Release, the generated winget submission (`grr-<tag>-winget-manifests.tar.gz`), then dispatches the watchers (demo/benchmark/stats/verify-delivery, plus changelog with `verify=true`) because a token-authored release starts no workflows |
| `publish.yml` | release/manual | `cargo publish --no-verify` (crate is source-only — read its comment) |
| `tag-release.yml` | push to main touching `Cargo.toml` | pushes the tag for a version on main that has none — fires the binary release and the crates.io publish |
| `auto-release.yml` | push to main (every push), Friday 04:31 UTC backstop, manual dispatch | computes the next version from the commit history and opens the bump PR with auto-merge — releases ship immediately after any meaningful push |
| `discovery.yml` | daily 04:17 UTC, manual | refetches discovery docs, regenerates the index AND everything derived from it (command tree, agent skills, site coverage table), opens a PR when any differs, runs `cargo test --lib` against the new data first |
| `changelog.yml` | push to main + release, manual (`dry_run`, `verify`) | regenerates `CHANGELOG.md` + `site/src/data/changelog.json`, opens a PR; a release-only report (`verify=true`, fired by release-binaries.yml since a token-authored release starts no workflows) checks that main's `CHANGELOG.md` has a section for the released version — a section is generated *from* its tag, so it can only land after the release, and a miss warns rather than failing; the grace-windowed hard gate is `verify-delivery.yml` |
| `benchmark.yml` | daily 03:23 UTC, release published | rebuilds grr in release mode, re-measures the credential-free metrics into `site/src/data/benchmarks.json`, opens a PR when they move (a version-gate step skips the daily run unless a new version shipped) |
| `demo.yml` | push to main (CLI/demo/generator/Cargo paths), release, weekly Sun 03:41 UTC | re-records `demo/demo.cast` in CI (`--ci-record`, carry-forward of the transport and agent segments), opens a PR |
| `automation-merge.yml` | `pull_request_target` (opened/synchronize/reopened) on an automation branch (`chore/changelog-regenerate`, `chore/demo-recording`, `chore/benchmark-refresh`, `chore/stats-refresh`, `chore/discovery-refresh`, `chore/release-v*`) | the unblocker for the bot PRs above: GITHUB_TOKEN-authored events land in `action_required`, so it approves the pending runs (letting the real `ci.yml` gate the merge), then merges artifact-only PRs (changelog/demo/benchmark/stats) immediately with `--admin` and enables auto-merge for code-affecting PRs (discovery refresh, release bump) — never `--admin` on those. Same-repo heads only; never checks out the PR head |
| `stats.yml` | push to `Cargo.toml`, daily 05:07 UTC, release | refreshes `site/src/data/stats.json` (download counts, latest tag/date, measured archive sizes) and opens a PR; idempotent, so a quiet day opens nothing |
| `verify-delivery.yml` | daily 06:17 UTC, manual dispatch (also fired by release-binaries.yml after a release) | runs `scripts/verify-delivery.ts`: checks the crates.io version, the docs.rs build verdict, the release asset set, the Homebrew tap, the live site, the changelog section for the released version, and generated-data freshness against `Cargo.toml`; a mismatch warns inside the release grace window and fails after it |
| `repo-metadata.yml` | weekly Mon 06:47 UTC, manual | syncs the repo description (method/service counts from the Discovery index), homepage, and topics via `scripts/update-repo-metadata.ts`; no-ops green when the `REPO_ADMIN_TOKEN` secret (Administration: write) is unset |
| `dependabot.yml` | weekly | cargo / github-actions / npm, all version types, grouped; every patch group is named `patches` so the auto-merge workflow can tell patches apart |
| `dependabot-auto-merge.yml` | dependabot PR opened/synchronize/reopened/ready_for_review | **patches merge immediately with `--admin`, bypassing required checks** — the repo prefers the latest patch even if it regresses; minors and majors wait for green CI |

## Windows PowerShell quirks (for agents on this machine)

These have each caused a real bug in this repo. Do not rediscover them:

- **`Set-Content` / `Out-File -Encoding utf8` mangle non-ASCII** (em-dashes → `â€"`) and add a BOM. Use the editor tools or `[IO.File]::WriteAllText` (UTF-8 without BOM). Verify with a strict UTF-8 decode after any scripted write.
- **`cd` does not persist for .NET APIs**: `[IO.File]::ReadAllText('relative\path')` resolves against the process start directory, not the shell's location. Use absolute paths.
- **`gh --jq '<expr with spaces>'` mangles** into multiple args. Use simple jq (`.full_name`), `ConvertFrom-Json`, or `--input` with a JSON file.
- **PowerShell 5.1 has no `&&`, no ternary `? :`** — use `if ($?) { }` and `if/else`.
- **GitHub Releases REST assets expose size in `asset.size`**, not `size_in_bytes` (that field is null).
- `gh api -f key=value` does not apply to repo renames — use `gh repo rename <name> -R owner/repo -y`.

## Documentation sync policy

Treat a commit as incomplete if docs are stale. What triggers a doc update:

- Adding/removing/renaming a command or flag → `README.md` command table, `site/src/pages/docs/commands.astro`, `skills/grr/SKILL.md`, `AGENTS.md` key commands
- Changing `grr skills` (targets, flags, embedded set) → `src/commands/skills.rs`, the `skills/README.md` template in `scripts/generate-skills.ts` (then `node scripts/generate-skills.ts`), `README.md`, `AGENTS.md`, `site/src/pages/docs/commands.astro`, `site/src/pages/docs/agents.astro`, `site/public/llms.txt`
- Changing auth/config behavior → `README.md`, `docs/gcp-setup.md`, `.env.example`, `config.toml.example`, the FAQ in `site/src/pages/index.astro`
- Changing packaging/archives → `README.md` packaging table, `site/src/pages/install.astro`
- Adding a service or changing discovery → `src/discovery/` (via the script, never by hand), `site/src/data/discovery-coverage.ts` (via `node scripts/generate-coverage.ts`, never by hand), the docs discovery page, `site/public/llms.txt`
- Changing the agent contract (output, discovery discipline, naming rules) → `skills/grr/SKILL.md`, `site/src/pages/docs/agents.astro`, `site/public/llms.txt`

Generated files (`CHANGELOG.md`, `src/discovery/*.json`, `src/commands/generated.rs`, `skills/**`, `site/src/data/changelog.json`, `site/src/data/discovery-coverage.ts`, every SVG under `site/src/assets/` and `site/public/` except hand-authored patterns) are never hand-edited; rerun the generator.

Key generator, not previously listed: `node site/scripts/generate-mascot.mjs` (pixel art + favicons + og card). `node scripts/subset-font.mjs` regenerates the subsetted IBM 3270 webfont from `site/fonts-src/` (Node + harfbuzz — there is no Python in this repo).
