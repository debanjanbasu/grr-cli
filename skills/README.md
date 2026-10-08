# grr agent skills

Agent skills for driving [grr](../README.md), the Rust CLI for Google Workspace. One core skill plus one skill per service, every service skill generated from the committed Discovery index (`src/discovery/*.json`) by `node scripts/generate-skills.ts` (`npm run skills`) — the skills, the command tree, and `grr api` all read the same source, so they can never drift.

## Progressive loading

1. **Load the core skill first** — it teaches the moves that work for every service.
2. **Pull a service skill on demand** — when the task settles on Gmail, Calendar, Drive, or any of the 14 services, load that service's skill for its command map, verified examples, and scope callout.

## Install

### With grr (offline)

```sh
grr skills install            # all 16 into ~/.agents/skills/ — the cross-client user-level location
grr skills install --claude   # mirror into ~/.claude/skills/ too (Claude Code reads only that)
grr skills install --force    # replace files that differ from the packaged copy
grr skills list               # the packaged set, and what each target directory holds
```

The skill files are compiled into the `grr` binary, so install needs no network: writing an identical file is a no-op, and a locally edited file is left alone (with a non-zero exit) unless `--force` is passed.

### With the skills CLI

```sh
npx skills add https://github.com/debanjanbasu/grr-cli                          # the whole skills directory (core + services)
npx skills add https://github.com/debanjanbasu/grr-cli/tree/main/skills/gmail   # one service skill
```

## The skills

- **[grr](./grr/SKILL.md)** — the core skill: the discover-first discipline (`grr schema` → `grr api list` → `grr api describe` → `--dry-run`), the method-id naming rule, the output contract, safety flags, and multi-account. Load this one first.
- **[grr-analyticsadmin](./analyticsadmin/SKILL.md)** — Use when the task touches Google Analytics Admin through grr (55 methods under `grr analyticsadmin` — properties, accounts, accountSummaries). Carries the service's generated command map, the naming rule applied to analyticsadmin, example commands verified with --dry-run, and the least-privilege scope callout.
- **[grr-analyticsdata](./analyticsdata/SKILL.md)** — Use when the task touches Google Analytics Data through grr (11 methods under `grr analyticsdata` — audienceExports, batchRunPivotReports, batchRunReports, checkCompatibility, getMetadata, …). Carries the service's generated command map, the naming rule applied to analyticsdata, example commands verified with --dry-run, and the least-privilege scope callout.
- **[grr-calendar](./calendar/SKILL.md)** — Use when the task touches Calendar through grr (38 methods under `grr calendar` — events, acl, calendarList, calendars, settings, …). Carries the service's generated command map, the naming rule applied to calendar, example commands verified with --dry-run, and the least-privilege scope callout.
- **[grr-chat](./chat/SKILL.md)** — Use when the task touches Google Chat through grr (54 methods under `grr chat` — spaces, users, customEmojis, media). Carries the service's generated command map, the naming rule applied to chat, example commands verified with --dry-run, and the least-privilege scope callout.
- **[grr-docs](./docs/SKILL.md)** — Use when the task touches Google Docs through grr (3 methods under `grr docs` — batchUpdate, create, get). Carries the service's generated command map, the naming rule applied to docs, example commands verified with --dry-run, and the least-privilege scope callout.
- **[grr-drive](./drive/SKILL.md)** — Use when the task touches Google Drive through grr (64 methods under `grr drive` — files, approvals, drives, comments, permissions, …). Carries the service's generated command map, the naming rule applied to drive, example commands verified with --dry-run, and the least-privilege scope callout.
- **[grr-forms](./forms/SKILL.md)** — Use when the task touches Google Forms through grr (10 methods under `grr forms` — watches, responses, batchUpdate, create, get, …). Carries the service's generated command map, the naming rule applied to forms, example commands verified with --dry-run, and the least-privilege scope callout.
- **[grr-gmail](./gmail/SKILL.md)** — Use when the task touches Gmail through grr (79 methods under `grr gmail` — settings, messages, drafts, labels, threads, …). Carries the service's generated command map, the naming rule applied to gmail, example commands verified with --dry-run, and the least-privilege scope callout.
- **[grr-people](./people/SKILL.md)** — Use when the task touches People through grr (24 methods under `grr people` — people, contactGroups, otherContacts). Carries the service's generated command map, the naming rule applied to people, example commands verified with --dry-run, and the least-privilege scope callout.
- **[grr-script](./script/SKILL.md)** — Use when the task touches Apps Script through grr (16 methods under `grr script` — projects, processes, scripts). Carries the service's generated command map, the naming rule applied to script, example commands verified with --dry-run, and the least-privilege scope callout.
- **[grr-searchconsole](./searchconsole/SKILL.md)** — Use when the task touches Google Search Console through grr (11 methods under `grr searchconsole` — sitemaps, sites, searchanalytics, urlInspection, urlTestingTools). Carries the service's generated command map, the naming rule applied to searchconsole, example commands verified with --dry-run, and the least-privilege scope callout.
- **[grr-sheets](./sheets/SKILL.md)** — Use when the task touches Google Sheets through grr (17 methods under `grr sheets` — values, developerMetadata, batchUpdate, create, get, …). Carries the service's generated command map, the naming rule applied to sheets, example commands verified with --dry-run, and the least-privilege scope callout.
- **[grr-slides](./slides/SKILL.md)** — Use when the task touches Google Slides through grr (5 methods under `grr slides` — pages, batchUpdate, create, get). Carries the service's generated command map, the naming rule applied to slides, example commands verified with --dry-run, and the least-privilege scope callout.
- **[grr-tasks](./tasks/SKILL.md)** — Use when the task touches Google Tasks through grr (14 methods under `grr tasks` — tasks, tasklists). Carries the service's generated command map, the naming rule applied to tasks, example commands verified with --dry-run, and the least-privilege scope callout.

<!-- GENERATED by scripts/generate-skills.ts from the committed Discovery index — index generated at 2026-10-08T11:21:45.619Z. Do not edit by hand; regenerate with `npm run skills`. -->
