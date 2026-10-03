## Contributing to grr

Thank you for wanting to contribute! Please read through the following guidelines to make the process smooth.

> **AI agent or coding harness?** Start with [`AGENTS.md`](./AGENTS.md) — the execution ruleset: a task-routing matrix, the build/test verification gate, the architecture invariants that have each caused a real bug, and the documentation sync policy. This file covers human-oriented contribution flow; AGENTS.md covers the rules an agent should work under.

### Development Setup

1. **Install Rust**: grr requires **nightly** — there is no stable path, and the build script fails with a clear message on other toolchains. [`rust-toolchain.toml`](./rust-toolchain.toml) pins it and supplies the components the build needs, while [`.cargo/config.toml`](./.cargo/config.toml) sets the `--cfg reqwest_unstable` flag that HTTP/3 (always compiled in) requires.
2. **Clone the repo**: `git clone https://github.com/debanjanbasu/grr-cli.git`
3. **OAuth client**: see [Local OAuth defaults](#local-oauth-defaults) below — either a repo-root `.env` (compiled into the binary) or `cargo run -- auth setup` (writes `~/.grr/config.toml`)
4. **Auth**: run `cargo run -- auth login` to complete Google consent
5. **Build**: `cargo build`
6. **Test**: `cargo test --locked`

### Local OAuth defaults

Official release builds are compiled in CI with `GRR_CLIENT_ID` / `GRR_CLIENT_SECRET` supplied as repository secrets, which is why a downloaded `grr` is zero-config. Your local build gets neither unless you supply them, so pick one of these:

- **`.env` (recommended for development).** Copy [`.env.example`](./.env.example) to `.env`, fill in `GRR_CLIENT_ID` and `GRR_CLIENT_SECRET` from Google Cloud console → Google Auth Platform → Clients, then rebuild. `build.rs` reads the process environment first and the repo-root `.env` second, and emits both through `cargo:rustc-env`, so every subsequent build is zero-config too. You can also pass them for a single build with `GRR_CLIENT_ID=… GRR_CLIENT_SECRET=… cargo build --release`.
- **`grr auth setup`.** No rebuild needed — it validates the client-id shape, writes `~/.grr/config.toml` (mode `0600` on unix), and refuses to clobber an existing file without `--force`. `--print-only` hands you the console URLs and the `gcloud services enable` line without writing anything, and `--enable-apis` runs that gcloud command for you. Under a non-TTY stdin it errors instead of hanging, so pass `--client-id`/`--client-secret` explicitly in scripts.

`.env` is gitignored and **must never be committed** — neither must a real client secret land in a commit message, an issue, or a CI log. The installed-app secret is not treated as confidential by Google (PKCE protects the flow, and it is always on), but it still does not belong in version control.

If you have no client at all, the full walkthrough is in [`docs/gcp-setup.md`](./docs/gcp-setup.md). You only need it when building from source or when you deliberately want your own client; release binaries ship with one.

### Discovery index and the generated tree

The entire service command tree — every `grr gmail …`, `grr calendar …`, … leaf — is generated at build time from a distilled index of Google's Discovery documents, committed at `src/discovery/*.json` (~420 KiB across 14 services) and embedded with `include_str!`. There are no hand-written service commands: the index is the surface.

- **Refresh it**: `node scripts/fetch-discovery.ts` (Node 24+; no dependencies to install). The output is deterministic — sorted keys — so an unchanged upstream produces an empty diff.
- **Regenerate what depends on it**: `node scripts/generate-commands.ts` re-emits `src/commands/generated.rs` (401 leaves, 994 typed flags), `node scripts/generate-skills.ts` re-emits `skills/<service>/SKILL.md`, and `node scripts/generate-coverage.ts` re-emits the site's `site/src/data/discovery-coverage.ts`. After any index change, run all three; an index and its derived artifacts move together or not at all.
- **Verify freshness**: the three `--check` modes exit 1 when their output is stale, and all three run in CI — they read only committed files, so they need no network. `node scripts/fetch-discovery.ts --check` is deliberately *not* a CI gate: it compares against live Google, so it would go red every time Google ships a method. The daily workflow is what keeps the index current.
- **Automation**: [`.github/workflows/discovery.yml`](./.github/workflows/discovery.yml) runs daily, regenerates the index and every artifact derived from it, and opens a PR when any differs. The repository never edits itself.
- **The generated files are committed on purpose.** Committing them keeps builds hermetic (no network at build time), makes `grr api list` and `grr schema` work offline, and means a Google-side change can never break a build. Do not hand-edit them, and do not gitignore them — regenerate and commit the diff instead. Review refresh diffs for surprising *removals*: a method disappearing usually means Google renamed it upstream.

If you add a service to the index, it goes in the `SERVICES` map in `scripts/fetch-discovery.ts` and the `include_str!` table in `src/discovery.rs`; the manifest is regenerated for you, and the next `generate-commands.ts` run gives it a full command namespace.

### Adding New Features

- This is one package, `grr-cli`, whose binary is `grr`. The four hand-written commands live in `src/commands/` (`auth.rs`, `api.rs`, `setup.rs`, `transport.rs`); shared transport, auth, and config live in `src/core/` (`http.rs`, `auth/`, `config.rs`).
- **Service commands are not hand-written.** To add or change one, change the Discovery index (via the fetch script) and regenerate — never edit `src/commands/generated.rs` directly. A new method Google ships upstream reaches the CLI through the daily PR with no code change.
- Keep the zero-config philosophy: new behavior should need no new config knobs unless there is no alternative
- Each change needs a clear brief, tests, and self-review
- Run `cargo clippy --all-targets -- -D warnings` before committing — must be clean
- Run `cargo fmt --all --check` to maintain formatting consistency
- Ensure `cargo test --locked` passes (green) before committing

### Code Style

- **Clippy**: `cargo clippy --all-targets -- -D warnings` must pass; `-D warnings` is the active lint enforcement (there is no workspace lints table to inherit)
- **Fmt**: `cargo fmt --all --check` must pass
- **Doc comments**: Public API functions must have `///` doc comments
- **No `expect`/`unwrap`/`panic!`**: avoid these in library and CLI code — degrade gracefully instead (tests use the `unwrap_or_else` pattern, e.g. `figment.extract::<T>().unwrap_or_else(|_| T::default())`). The root package has no workspace lints table; `-D warnings` in the Clippy command is the enforcement.

### Submitting Changes

1. Commit with a clear message: `git commit -m "feat: <descriptive title>"`
2. Push to your fork
3. Open a Pull Request against `main` branch
4. PR must pass `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`
5. Include test coverage for any new functionality

### Reporting Issues

- Use the GitHub issue tracker
- Include `cargo metadata --no-deps --format-version 1` output if build issues
- Specify OS, Rust version, and `grr` version
