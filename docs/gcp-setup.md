# Google Cloud setup for grr — optional

**You probably do not need this page.** Official release binaries (GitHub Releases, Homebrew) ship with an OAuth client already compiled in, so `grr auth login` works the moment you install. Read [Authentication and configuration](../README.md#authentication-and-configuration) first.

This walkthrough is for two cases only:

- **You are building grr from source** (`cargo install`, or a local `cargo build`) — the compiled binary has no client, so you need one of your own.
- **You want your own client anyway** — your own Cloud project, your own quota, your own consent screen.

Either way the work splits in two halves, and only the first one needs this page:

| Half | Who does it |
| --- | --- |
| Create the project, enable the APIs, create the OAuth client | You, in the Cloud console (steps 1–6 below). `grr` cannot create Google Cloud projects. |
| Write the client into `~/.grr/config.toml` | [`grr auth setup`](#7-write-the-config) — one command, no file editing |

Budget about five minutes.

## 1. Install the Google Cloud CLI

Only needed for the project and API steps; `grr auth setup` works without it (it just prints the `gcloud services enable` line for you to run).

Windows (PowerShell):

```powershell
winget install Google.CloudSDK
```

macOS:

```sh
brew install --cask google-cloud-sdk
```

Linux (apt/dnf/etc.): follow the official instructions at <https://cloud.google.com/sdk/docs/install>.

## 2. Sign in to Google

```sh
gcloud auth login
```

This signs **gcloud** in, not grr — grr has its own consent flow (`grr auth login`). You need it only to run the `gcloud projects` and `gcloud services` commands below.

## 3. Pick (or create) a project

List what you have and select one:

```sh
gcloud projects list
gcloud config set project YOUR_PROJECT_ID
```

Starting from scratch:

```sh
gcloud projects create YOUR_PROJECT_ID
gcloud config set project YOUR_PROJECT_ID
```

A personal project on the free tier comfortably covers grr's usage of these APIs for your own account.

## 4. Enable the APIs

Gmail is required; enable the rest as you need them (Calendar, Drive, Contacts, Chat, and Forms are all live `grr` services, and so are Tasks, Docs, Sheets, Slides, Apps Script, Analytics, and Search Console):

```sh
gcloud services enable gmail.googleapis.com

# all the other grr services at once:
gcloud services enable calendar-json.googleapis.com drive.googleapis.com people.googleapis.com chat.googleapis.com forms.googleapis.com
```

`grr auth setup --enable-apis` runs `gcloud services enable` for all 14 grr services at once — Gmail, the five above, and Tasks, Docs, Sheets, Slides, Apps Script, Analytics Admin, Analytics Data, and Search Console — when `gcloud` is on `PATH` ([src/commands/setup.rs](../src/commands/setup.rs) holds the list).

Enable the remaining service APIs too:

```sh
gcloud services enable tasks.googleapis.com docs.googleapis.com sheets.googleapis.com slides.googleapis.com script.googleapis.com analyticsadmin.googleapis.com analyticsdata.googleapis.com searchconsole.googleapis.com
```

Note: Google Keep's API is Workspace-enterprise-only (no consumer API), so Keep will never appear as a `grr` service.

## 5. OAuth consent screen

Open <https://console.cloud.google.com/apis/credentials/consent>:

1. User type: **External**
2. Fill in the minimal form. Use **Rust Rewrite** as the app name, choose a support email, and upload [`assets/logo-google-app.png`](../assets/logo-google-app.png) as the app logo — the form takes a square PNG (≥120×120), and the crab there is the same pixel-art mascot as the repo icon. The project/fork is still **Google Rust Rewrite**; **Rust Rewrite** is the name shown by Google's consent screen.
3. Use the project's public URLs when the form asks for them:
   - Homepage: <https://grr-cli.pages.dev/>
   - Privacy policy: <https://grr-cli.pages.dev/privacy/>
   - Terms: <https://grr-cli.pages.dev/terms/>
4. Add `grr-cli.pages.dev` as the authorized domain.
5. Add the Google account you will sign in with as a **Test user**.

If the account is missing from that list, Google still *shows* the consent page and then refuses to complete it: the page keeps loading, **Continue** appears to do nothing, and no error ever surfaces — in every browser, on every input method. In Testing mode only test users may authorize sensitive or restricted scopes; `openid`, `email`, `profile` and the `userinfo.*` pair are exempt, which is why a login that grants only those still succeeds and looks healthy. Check **Audience → Test users** before suspecting the browser, the network, or the scope list.

One login covers every `grr` service, so the consent screen asks for everything the embedded Discovery index names — the union of every scope any of the 401 methods accepts, across all 14 APIs — even if you only plan to use one, minus the handful Google only ever grants to a *service account* (Chat's `chat.bot`, `chat.app.*`, `chat.import`). Those are filtered out on purpose: a user consent screen cannot show them, and requesting one fails the whole authorization with `invalid_scope`, so leaving them in would mean no login at all. That set is derived from the index rather than hand-maintained, so it grows when Google publishes new surface. Enable the matching APIs (step 4) for the services you use.

### Declare the scopes: Google Auth Platform → Data access

A consent screen grants only the scopes the project has **declared**. There are two failure modes, and only the first one is quiet:

- **Empty list.** Everything is dropped silently: the login succeeds and hands you a token holding just `openid`, `email`, `profile` and `userinfo.*`, and the first real call fails with `403 insufficientPermissions`.
- **Partial list** — the state a half-finished declaration leaves behind. A request naming a scope that is *not* declared can leave Google's consent page **hanging forever** after Continue: no error, no redirect, no callback, nothing in the log, and `grr auth login` times out after five minutes. This looks exactly like a broken browser and is not one.

Declare the whole list, and expect to do it in **batches**: the console's *Manually add scopes* box refuses a large paste with "unable to save changes", and a silently partial save is what produces the hang above. The panel lists only scopes whose APIs are enabled, so run step 4 first — a scope for a disabled API cannot be declared at all.

1. Open **Google Auth Platform → Data access → Add or remove scopes**.
2. Paste the scope list into **Manually add scopes** (one per line), click **Add to table**, then **Update**.
3. Re-run `grr auth login`.

Take the list from the embedded index rather than typing it — from a checkout:

```sh
node -e 'const fs=require("fs"),p="src/discovery/";const s=new Set();const w=o=>{if(o&&typeof o==="object")Array.isArray(o.scopes)?o.scopes.forEach(x=>s.add(x)):Object.values(o).forEach(w)};fs.readdirSync(p).filter(f=>f.endsWith(".json")).forEach(f=>w(JSON.parse(fs.readFileSync(p+f))));console.log([...s].sort().join("\n"))' > /tmp/grr-scopes.txt
```

Declaring a scope does not grant it — you still approve scopes on the consent screen, and you can leave any of them unticked there. Declaring sensitive or restricted ones is also what puts the app on Google's verification path; an unverified app can still use them with its own test users.

While the consent screen is in **Testing** mode, Google expires refresh tokens after about 7 days — rerun `grr auth login` when that happens. A release that widens the derived scope set (a discovery refresh that brings new API surface) needs one fresh `grr auth login` to grant it; otherwise an existing token keeps working. Publishing the app avoids the testing-mode expiry, but is unnecessary for personal use.

## 6. Create the OAuth client

Open <https://console.cloud.google.com/apis/credentials>:

1. **Create credentials → OAuth client ID**
2. Application type: **Desktop app** (name it anything, e.g. `grr`)
3. Copy the **Client ID** (ends in `.apps.googleusercontent.com`) and the **client secret** shown next to it

### Optional: a second client for `grr auth login --device`

Google's device endpoint (`RFC 8628`) accepts exactly one application type — **TVs and Limited Input devices** — and answers a Desktop client with `invalid_client: Invalid client type`, so `--device` cannot ride on the client above. If you log in on headless machines, repeat the steps above with **Application type: TVs and Limited Input devices**, then add the pair to `~/.grr/config.toml`:

```toml
[oauth]
device-client-id = "123456789-def.apps.googleusercontent.com"
device-client-secret = "GOCSPX-..."
```

Without it, `grr auth login --device` says exactly this and stops — no half-finished request to Google. The browser flow never needs it.

**What a device login can reach.** Google's device flow keeps its own scope allowlist, and it is small: `drive.file`, `calendar`, `calendar.readonly`, the legacy Calendar/Contacts feeds, `userinfo.email`, `userinfo.profile`, `openid`. A device login therefore covers Calendar and Drive-file calls and nothing else — Gmail, Chat, Docs, Sheets, Slides, Tasks, Forms, Apps Script, Analytics and the admin surfaces need `grr auth login` in a browser. `grr auth login --device` prints that list before you approve anything. (Google's own documentation for the flow lists `drive.appdata` and `drive.metadata.readonly`, which its endpoint actually rejects — `grr` requests only what the endpoint accepts.)

## 7. Write the config

Let `grr` do it:

```sh
grr auth setup --client-id 123456789-abc.apps.googleusercontent.com --client-secret GOCSPX-...
```

It validates the client-id shape (a truncated paste is the most common mistake, so it is rejected up front), writes `~/.grr/config.toml` mode `0600` on unix, and refuses to overwrite an existing file without `--force`. Run `grr auth setup` with no flags to be prompted instead, or `--print-only` to get the recipe without writing anything. Both key spellings are accepted:

```toml
[oauth]
client_id = "123456789-abc.apps.googleusercontent.com"
client_secret = "GOCSPX-..."
```

See [`config.toml.example`](../config.toml.example) for the annotated template. `GRR_CONFIG_PATH` moves the file elsewhere.

Google shows a client secret even for Desktop clients. `grr` sends it at the token endpoint only when present — PKCE is always on either way. It is not a confidential value in Google's model (which is why release binaries may carry one), but keep it out of version control and out of CI logs all the same.

## 8. Log in with grr

```sh
grr auth login
```

Your browser opens, you consent, done. On a headless machine use `grr auth login --device` and follow the printed URL + code. Verify with:

```sh
grr auth status
grr gmail users getProfile --user-id me
```

## Agent environments: no MCP setup

`grr` replaces per-service MCP servers with one fast CLI. There is no MCP setup for `grr` itself: point your agent at the `grr` binary and have it read `grr schema` for the complete machine-readable command contract.
