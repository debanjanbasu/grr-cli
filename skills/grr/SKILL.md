---
name: grr
description: Use grr, a fast Rust CLI for Google Workspace (Gmail, Calendar, Drive, People, Chat, Forms, Tasks, Docs, Sheets, Slides, Apps Script, Analytics, Search Console), whenever the user wants to read or write Google data from a terminal, script, or agent. Discover methods before calling, predict command names from method ids, and parse JSON from stdout.
---

# grr — Google Workspace from the terminal

One binary, one OAuth login, **401 methods across 14 Google APIs**, JSON on stdout. The entire command tree is generated from Google's Discovery Service, so the move is always the same: discover what exists, then call it.

## Progressive loading

This is the CORE skill. When the work concentrates on one service, also load that service's generated skill — `skills/gmail/SKILL.md`, `skills/calendar/SKILL.md`, … (14 total) — for its method surface, verified examples, and scope caveats. Load this core first; pull a service skill only when needed.

## Golden rule: DISCOVER FIRST

Never guess a method id, parameter, or scope. All of these work offline, before any login:

```sh
grr schema                                          # the full command tree as JSON — the machine contract
grr api list [--service X] [--filter SUBSTR] [--grouped]   # browse every method
grr api describe <method-id>                        # params, required flags, scopes, verb, URL for one method
```

Read `describe` before any first call — it is the authority on required parameters, repeats, and enums. `grr <service> --help` gives the same for one branch.

## Predict any command from its method id

Service commands mirror Discovery method ids exactly; dots become spaces:

```sh
gmail.users.messages.list       ->  grr gmail users messages list
calendar.events.list            ->  grr calendar events list
drive.files.list                ->  grr drive files list
sheets.spreadsheets.values.get  ->  grr sheets spreadsheets values get
tasks.tasklists.list            ->  grr tasks tasklists list
```

Top level: `grr auth | api | ask | completions | mcp | schema | skills | transport` plus one subcommand per service (gmail, calendar, drive, people, chat, forms, tasks, docs, sheets, slides, script, analyticsadmin, analyticsdata, searchconsole). Resources nest as subcommands; each method is a leaf that also carries its bare name as a visible alias (`list`, `get` — camelCase methods keep their casing, e.g. `getProfile`).

Flags are generated from the same ids, camelCase → kebab-case:

```sh
grr gmail users messages list --user-id me --q "is:unread" --max-results 5 \
    --label-ids INBOX --label-ids UNREAD --include-spam-trash
```

- Integers parse as numbers, booleans are presence flags, repeated parameters repeat, enum parameters validate, required parameters are enforced.
- A parameter literally named `format` or `query` is exposed as `--param-format` / `--param-query` (escape-hatch collision rule).
- Request bodies are untyped by design: pass JSON via `--params`, or a file/stdin via `--body-file PATH|-`. Typed flags win over `--params` on conflict.
- Every leaf also accepts `--params`, `--body-file`, repeatable `--query KEY=VALUE` (raw query pairs), `--dry-run`, and `-f json|jsonl|table|pretty` (default json).

## Output contract

- **stdout is data; stderr is logs.** Default output is JSON, so `grr … | jq` always works.
- **Non-zero exit on failure.** Errors land on stderr and are actionable: a 403 names the missing scope, an unknown method id gets ranked suggestions, a missing required parameter names itself.
- **`--dry-run` before anything destructive.** It prints the resolved verb, URL, body, and scopes as JSON and sends nothing — the cheapest correctness check there is.
- Plain-text exceptions (do not JSON-parse): `grr transport`, `grr skills install` (a human install summary), `grr auth login --device` (prints URL + code before the JSON), and `grr --version` (a banner, not data &mdash; its **first line is the semver**, `grr <x.y.z>`; the rest is the mascot).
- `grr --version` may carry ANSI colour when stdout is a 24-bit-colour terminal. Read the version with `head -1`, or `NO_COLOR=1 grr --version | head -1`, never with a whole-output parse.

## Agent skills: install and self-migration

`grr skills install` copies the 16 packaged skills (this file, the 14 service skills, and the index) into `~/.agents/skills/` — offline, no auth, zero config; `--claude` also mirrors them into `~/.claude/skills/` (the one directory Claude Code reads).

Installs are provenance-tracked and self-migrating:

- `grr skills install` records a manifest beside the files (`~/.agents/skills/.grr-skills.json`): the grr version, the packaged content it installed, and the exact bytes it wrote. A per-file backup under `.grr-backup/` is the 3-way merge base.
- After a version change, the next grr command runs the migration **lazily** — one manifest read; nothing is hashed when the recorded version matches the running binary. Then, per file:
  - never touched since install → replaced with the new packaged content;
  - locally edited where the packaged side changed elsewhere → **3-way merged**, the local edit kept;
  - a genuine conflict → your file is left untouched and the new packaged copy is written beside it as `<name>.grr-incoming`, with one note on stderr.
- `--force` still overwrites regardless. A file grr never installed is never touched (no manifest entry = not ours).
- `grr skills list` reports per target and file: `current`, `outdated` (packaged moved, local untouched), `locally-modified`, `missing`, or `unmanaged`.

stdout stays JSON for `grr skills list`; `grr skills install` prints a human summary (a documented plain-text exception).

## Auth state

- `grr auth status` — check first. One live `gmail.users.getProfile` call; JSON `{authenticated, email, …}`; non-zero exit when there is no valid credential. Do not retry auth failures in a loop.
- `grr auth login` — PKCE browser flow. `grr auth login --device` for headless machines (prints a URL + code; a human still approves it).
- Release binaries are zero-config (an OAuth client is compiled in). Source builds need `grr auth setup` first.

## Scope honesty

grr consents to every scope the embedded index names, so any method the index can express is inside the grant — there is no per-method consent note and no escalation path. A 403 is therefore not a missing consent: it is either a grant that predates the current scope set (the remedy is a human re-running `grr auth login`) or an API that is not enabled on the Cloud project. Surface that distinction; never retry a 403 in a loop.

## Safety profiles — global flags, honored everywhere

`--readonly`, `--deny-service <name>`, `--deny-verb <VERB>` are global: they parse before or after the subcommand, and the gate covers the generated tree, `grr api call`, and `grr mcp` alike.

```sh
grr --readonly gmail users messages delete --user-id me --id abc   # refused: DELETE
grr gmail users messages delete --user-id me --id abc --readonly   # same refusal
grr --deny-service chat chat spaces list                            # refused: the chat service
```

An agent driving grr for a user should pass `--readonly` unless the user asked for a write — it converts every destructive method into an actionable error instead of a side effect.

## Multi-account

`grr auth login --account work` and `grr auth status --account work`: each named account keeps its own token in the OS keyring (the default account's naming is untouched). Names are validated: trimmed, lowercased, no spaces/colons/slashes. Pass `--account` whenever the user has more than one Google account and named which one.

## MCP server: `grr mcp`

`grr mcp` serves a Model Context Protocol (JSON-RPC 2.0 over stdio) server exposing every method in the index as a typed MCP tool:

- Tool name = the dotted method id (`gmail.users.messages.list`); `tools/list` returns every tool with an inputSchema built from Discovery (typed properties, required list, enum values) plus `params`/`body_file`/`query`/`dry_run`.
- `tools/call` runs through the same engine as the CLI — results come back as JSON text content; errors are `isError` results, never a dead server.
- Connect any MCP client: register `grr mcp` as a stdio server (Claude Desktop, Gemini CLI, VS Code, Cursor). `--readonly` gives a server that refuses writes.
- The stream is stdout; logs go to stderr. Do not print to the stream.

## Natural language: `grr ask`

`grr ask "<request>"` turns plain English into a typed method + parameters using a System One model (TypeSafe's Jev by default; any provider speaking the same contract works by config — `[systemone] endpoint/model` in `~/.grr/config.toml`, `TYPESAFE_API_KEY` in the env):

- Hierarchical classification: service (14 options) → method (≤79) → one typed question per required parameter. Every candidate is code-supplied from the discovery index — the model can only select, never invent.
- **It prints the PLAN by default** (method, params, url, scopes, confidence as JSON) and sends nothing. `--run` executes through the same engine as `grr api call`.
- Confidence below the threshold (default 0.6) is flagged, not hidden. Planning needs no Google credential — only the API key.

```sh
grr ask "show my unread gmail"                    # -> gmail.users.messages.list, {"userId":"me"}, the plan
grr ask "next week's calendar" --run              # judge + execute
```

## The flat escape hatch: `grr api`

Any method by id, no tree walking — the same engine, byte-identical requests:

```sh
grr api list --service sheets
grr api describe sheets.spreadsheets.values.get
grr api call sheets.spreadsheets.values.get --param spreadsheetId=1AbC… --param range=Sheet1!A1:B10
grr api call gmail.users.messages.list --param userId=me --dry-run   # identical to: grr gmail users messages list --user-id me --dry-run
grr api refresh [--service X]   # pull the Discovery index forward between releases
```

Prefer the generated tree for readability — `grr api call` adds no power. Reach for `grr api` when composing method ids dynamically.
