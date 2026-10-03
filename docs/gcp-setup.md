# Google Cloud setup for grr — optional

**You probably do not need this page.** Official release binaries (GitHub Releases, Homebrew, winget) ship with an OAuth client already compiled in, so `grr auth login` works the moment you install. Read [Authentication and configuration](../README.md#authentication-and-configuration) first.

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

Gmail is required; enable the rest as you need them (Calendar, Drive, Contacts, Chat, and Forms are all live `grr` services):

```sh
gcloud services enable gmail.googleapis.com

# all the other grr services at once:
gcloud services enable calendar-json.googleapis.com drive.googleapis.com people.googleapis.com chat.googleapis.com forms.googleapis.com
```

`grr auth setup --enable-apis` runs exactly that first command for you, for all six, when `gcloud` is on `PATH`.

**If you use `grr api`**, four more APIs open up — Tasks, Docs, Sheets, and Slides have no dedicated `grr` command and are reachable only through [`grr api`](../README.md#every-method-not-just-the-curated-ones):

```sh
gcloud services enable tasks.googleapis.com docs.googleapis.com sheets.googleapis.com slides.googleapis.com
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

One login covers every `grr` service, so the consent screen will ask for all of them — Gmail (read/compose/modify/labels), Calendar, Drive, Contacts, Chat (messages/spaces/memberships/reactions), and Forms (body/responses) — even if you only plan to use one. Enable the matching APIs (step 4) for the services you use.

While the consent screen is in **Testing** mode, Google expires refresh tokens after about 7 days — rerun `grr auth login` when that happens. The 0.4 release also adds `chat.delete`, `chat.memberships`, `chat.messages.reactions`, and `contacts.other.readonly`; existing users must run `grr auth login` again to grant those scopes. Publishing the app avoids the testing-mode expiry, but is unnecessary for personal use.

## 6. Create the OAuth client

Open <https://console.cloud.google.com/apis/credentials>:

1. **Create credentials → OAuth client ID**
2. Application type: **Desktop app** (name it anything, e.g. `grr`)
3. Copy the **Client ID** (ends in `.apps.googleusercontent.com`) and the **client secret** shown next to it

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
grr gmail profile
```

## Agent environments: no MCP setup

`grr` replaces per-service MCP servers with one fast CLI. There is no MCP setup for `grr` itself: point your agent at the `grr` binary and have it read `grr schema` for the complete machine-readable command contract.
