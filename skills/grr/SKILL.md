---
name: grr
description: Use grr, a fast Rust CLI for Google Workspace (Gmail, Calendar, Drive, People, Chat, Forms, Tasks, Docs, Sheets, Slides), whenever the user wants to read or write Google data from a terminal, script, or agent. Discover methods before calling, predict command names from method ids, and parse JSON from stdout.
---

# grr — Google Workspace from the terminal

One binary, one OAuth login, **308 methods across 10 Google APIs**, JSON on stdout. The entire command tree is generated from Google's Discovery Service, so the move is always the same: discover what exists, then call it.

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

Top level: `grr auth | api | schema | transport` plus one subcommand per service (gmail, calendar, drive, people, chat, forms, tasks, docs, sheets, slides). Resources nest as subcommands; each method is a leaf that also carries its bare name as a visible alias (`list`, `get` — camelCase methods keep their casing, e.g. `getProfile`).

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
- Plain-text exceptions (do not JSON-parse): `grr transport`, and `grr auth login --device` (prints URL + code before the JSON).

## Auth state

- `grr auth status` — check first. One live `gmail.users.getProfile` call; JSON `{authenticated, email, …}`; non-zero exit when there is no valid credential. Do not retry auth failures in a loop.
- `grr auth login` — PKCE browser flow. `grr auth login --device` for headless machines (prints a URL + code; a human still approves it).
- Release binaries are zero-config (an OAuth client is compiled in). Source builds need `grr auth setup` first.

## Scope honesty

grr consents to a fixed scope set at login. A method whose least-privilege scope (the `leastPrivilegeScope` field in `describe`) falls outside that set prints a **note on stderr** and is attempted anyway; Google may then answer 403, and the error names the exact scope. That is a user decision, not a retry candidate — widening scopes requires a human to re-run `grr auth login`. Surface the note to the user instead of looping.

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
