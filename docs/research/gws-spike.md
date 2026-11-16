# haulmer-connect spike: gws v0.22.5 facts

Date: 2026-11-16. Source: `googleworkspace/cli` at tag `v0.22.5` (commit `705fb0ec`), installed binary `gws 0.22.5`, plus yup-oauth2 12.1.2 (the version in Cargo.lock).
Short links: `AC` = https://github.com/googleworkspace/cli/blob/v0.22.5/crates/google-workspace-cli/src/auth_commands.rs, `CS` = …/src/credential_store.rs, `AU` = …/src/auth.rs.
All probes ran with `GOOGLE_WORKSPACE_CLI_CONFIG_DIR` set to a scratch folder and a **fake** client id and secret. There was no real sign-in, and nothing read or wrote `~/.config/gws`. Probe scripts were throwaway and are not kept.

---

## 1. PKCE + loopback redirect

**ANSWER: Loopback redirect yes, PKCE no.** The redirect URI is `http://localhost:<ephemeral-port>` with no path. The listener binds `127.0.0.1:0`. The auth URL has no `code_challenge`, and the code exchange sends `client_secret`. The OAuth client must be a **Desktop app** type, because that type accepts any localhost port.

- yup-oauth2 path (the default): `AC#L181-L231` uses `InstalledFlowAuthenticator` + `HTTPRedirect`. yup-oauth2 12.1.2 `src/installed.rs` L27-58 builds the URL with `scope, access_type=offline, redirect_uri, response_type=code, client_id, prompt=select_account+consent` and no PKCE params. L191: `format!("http://localhost:{}", port)`. L274-276 binds `127.0.0.1:0`.
- Proxy path (used if any `http_proxy/HTTP_PROXY/https_proxy/HTTPS_PROXY/all_proxy/ALL_PROXY` is set, `AU#L28-L35`, `AC#L623`): `AC#L113-L163` also binds `127.0.0.1:0` and uses `redirect_uri=http://localhost:{port}`. It also has no PKCE (`AC#L75-L89`, `AC#L38-L51`).
- Observed (probe2): the auth URL printed `redirect_uri=http://localhost:52073`, and `lsof` showed `TCP 127.0.0.1:52073 (LISTEN)`.

## 2. `--scopes` format; running without a terminal; exit codes

**ANSWER:**
- Use **full scope URLs**, separated by commas. gws passes them through unchanged (`AC#L487-L495`, `AC#L762`), so short names reach Google as-is and are invalid. In probe2, `gmail.readonly` was sent literally. gws always appends `openid`, `…/userinfo.email` and `…/userinfo.profile` (`AC#L605-L614`). It drops `gmail.metadata` when a broader Gmail scope is present (`AC#L855-L879`).
- **No prompts when `--scopes` is given.** `Custom` scopes return before the TTY check. The scope picker only runs when stdin is a TTY and no `--scopes/--readonly/--full` is given (`AC#L761-L779`). Login never reads stdin. Probe2 ran with `</dev/null` and did not block on input.
- **gws does not open the browser itself.** The help text says "(opens browser)", but no browser-opening code exists in the crate (grep for `open::` / `webbrowser` returns nothing). gws prints the URL and waits on the loopback listener:
  - default path: prints `Open this URL in your browser to authenticate:\n\n  <url>\n` to **stderr** (`AC#L568-L569`);
  - proxy path: prints the same text to **stdout** (`AC#L129-L130`).
  - **The app must read the `https://accounts.google.com/...` line from stderr (and from stdout if a proxy is set) and open it.**
- **No timeout.** gws waits for the callback forever. The app must kill the process if the person cancels.
- **On success** gws prints JSON to stdout and exits 0 (`AC#L647-L658`):
  `{"status":"success","message":…,"account":"<email>|(unknown)","credentials_file":…,"encryption":…,"scopes":[…]}`.
  ⚠ `scopes` here is what was **requested**, not what was granted. With granular consent the person can untick some, so use `auth status` for the granted list (item 7).
- **On failure** gws prints `{"error":{"code","message","reason"}}` to stdout, a coloured `error[...]` line to stderr, and exits with (`crates/google-workspace/src/error.rs` L46-54, `src/main.rs` L51-58):
  - 1 = api
  - 2 = auth
  - 3 = validation
  - 4 = discovery
  - 5 = other

  Observed: no client gives exit 2 with `reason:"authError"`. A flag conflict gives exit 3 with `validationError`.
- Gotchas:
  - `main()` runs `dotenvy::dotenv()` (`src/main.rs` L49), so a `.env` in the **working directory** of gws can inject env vars. Spawn gws with a controlled cwd.
  - stderr also gets `Using keyring backend: …` (`CS#L360`).

## 3. Client id/secret storage; env vars; config-dir layout

**ANSWER:** The env vars are **not** needed on every call. Login stores the client id and secret twice:
- in `client_secret.json`: written from the env vars on first login if the file is missing (`AC#L587-L593`, `src/oauth_config.rs` L56-89). It is plaintext, in the GCP "installed" format.
- inside `credentials.enc` as `{"type":"authorized_user","client_id","client_secret","refresh_token"}` (`AC#L630-L645`). API calls read from here (`AU#L203-L229`).

Env vars (`README.md` L380-389):

| Variable | Purpose |
|---|---|
| `GOOGLE_WORKSPACE_CLI_CLIENT_ID` | OAuth client id |
| `GOOGLE_WORKSPACE_CLI_CLIENT_SECRET` | OAuth client secret |
| `GOOGLE_WORKSPACE_CLI_CONFIG_DIR` | Config folder |
| `GOOGLE_WORKSPACE_CLI_CREDENTIALS_FILE` | Credentials file path |
| `GOOGLE_WORKSPACE_CLI_TOKEN` | Raw access token |
| `GOOGLE_WORKSPACE_CLI_KEYRING_BACKEND` | `keyring` (default) or `file` |
| `GOOGLE_WORKSPACE_CLI_LOG` | Log level |
| `GOOGLE_WORKSPACE_CLI_LOG_FILE` | Log folder |
| `GOOGLE_WORKSPACE_CLI_SANITIZE_TEMPLATE` | Model Armor template |
| `GOOGLE_WORKSPACE_CLI_SANITIZE_MODE` | Model Armor mode |
| `GOOGLE_WORKSPACE_PROJECT_ID` | Project for quota and billing |

When login is resolving the client, env vars win over `client_secret.json` (`AC#L718-L749`).

**Config folder:** `$GOOGLE_WORKSPACE_CLI_CONFIG_DIR`. If unset, gws uses `~/.config/gws`, or the legacy `~/Library/Application Support/gws` / `%APPDATA%\gws` if that folder exists (`AC#L311-L335`).

Files in the config folder:

| File | When it exists |
|---|---|
| `client_secret.json` | After first login |
| `credentials.enc` | After login. AES-256-GCM, nonce‖ciphertext |
| `credentials.json` | Optional plaintext alternative |
| `token_cache.json` | After API calls |
| `sa_token_cache.json` | Service-account token cache |
| `credentials.tmp` | Only during login |
| `.encryption_key` | File backend only |

**Detect "credentials exist":** run `gws auth status`, then check `encrypted_credentials_exists || plain_credentials_exists` and `has_refresh_token` (the probe printed these fields). Or check that `<dir>/credentials.enc` exists.

⚠ **Fallback:** with no `credentials.enc` or `credentials.json`, gws silently falls back to gcloud ADC (`GOOGLE_APPLICATION_CREDENTIALS` or `~/.config/gcloud/application_default_credentials.json`; `AU#L205-L213`). "Not connected" does not always mean gws has no credentials. Consider unsetting `GOOGLE_APPLICATION_CREDENTIALS` for agents.

⚠⚠ **Encryption key collision (important for the pilot's custom folder).** With the default `keyring` backend on macOS and Windows, the key lives in the OS keychain under **service `gws-cli`, account `$USER`**. That entry is shared by every config folder (`CS#L362-L367`).
- If a key is already there, gws uses it and **deletes `<dir>/.encryption_key`** (`CS#L204-L255`).
- So a haulmer-connect login with a custom folder reads, or creates, the same keychain item that the other person's `~/.config/gws` setup uses.
- If that setup relies on `.encryption_key`, our first login would create a keychain key. Their next gws run would then use the keychain key and delete their `.encryption_key`, and `credentials.enc` would no longer decrypt. A reasonable inference from the code; not reproduced.
- **Recommendation:** always set `GOOGLE_WORKSPACE_CLI_KEYRING_BACKEND=file` together with a non-default config folder. Then the key is `<dir>/.encryption_key`, separate per folder (`CS#L325-L340`). Also write it into the installed skills. The trade-off is that the key sits as a file next to the encrypted credentials, so the encryption protects much less.

## 4. Meet transcripts scopes

**ANSWER: `meetings.space.readonly` alone is enough to list transcripts and read transcript entries (the text) through the Meet API.** `drive.meet.readonly` (restricted) is only needed to download or export the Google Docs transcript file or the MP4 recording from Drive.
- `conferenceRecords.transcripts.list` and `transcripts.entries.list` require one of `meetings.space.created` or `meetings.space.readonly`: https://developers.google.com/workspace/meet/api/reference/rest/v2/conferenceRecords.transcripts.entries/list (and `…/transcripts/list`).
- The Meet auth guide lists `drive.readonly` ("download recording and transcript files from Google Drive API", Restricted) and `drive.meet.readonly` (Restricted): https://developers.google.com/workspace/meet/api/guides/authenticate-authorize
- Drive `files.export` accepts `drive`, `drive.file`, `drive.meet.readonly` or `drive.readonly`: https://developers.google.com/workspace/drive/api/reference/rest/v3/files/export
- Caveats:
  - transcript entries are **deleted 30 days after the conference**;
  - entries may differ from the Docs file;
  - artifacts are only produced if transcription was turned on (https://developers.google.com/workspace/meet/api/guides/artifacts).
- Recommendation: Meet Read = `meetings.space.readonly` only. If the Drive Read box is ticked, `drive.readonly` already covers the Docs file. Add `drive.meet.readonly` only if Docs-file access without Drive Read becomes a requirement.

## 5. Drive write: `drive` vs `drive.file`

**ANSWER:** Recommend **`drive`** for the "Drive Write" option, but keep it opt-in (as the spec says).
- `drive.file` is non-sensitive and per-file. It covers only files the app created, or files the person opened or shared with the app through the Google Picker or the app's own file picker.
- An agent using gws has no picker, so with `drive.file` it can write only files that gws itself created. It cannot edit any file the person already owns.
- `drive` is restricted and on the admin "high-risk" list. For an Internal app, Google verification and the security assessment do not apply.
- Source: https://developers.google.com/workspace/drive/api/guides/api-specific-auth (Non-sensitive: `drive.file`; Restricted: `drive`, `drive.readonly`, `drive.meet.readonly`, …).
- Possible middle ground: Drive Write = `drive.file` ("create and upload only") plus `drive.readonly` for reading. A product decision.

## 6. Admin trust for Gmail/Drive/Chat

**ANSWER: Only if the admin has restricted those services.** By default, services are unrestricted and an Internal app works without being trusted.
- If the admin sets a service to **Restricted**, or turns on the high-risk-scope restriction for Gmail, Drive or Chat, the app must be **Trusted**, have **Specific Google data** access, or be covered by **"Trust internal apps"**.
- Source: https://support.google.com/a/answer/7281227 (now knowledge.workspace.google.com/admin/apps/control-which-apps-access-google-workspace-data), sections:
  - "Restrict or unrestrict Google services": only apps set to Trusted or Specific Google data can reach a Restricted service, and existing tokens of untrusted apps are revoked.
  - "Restrict access to high-risk OAuth scopes": the high-risk lists.
  - "Choose settings for unconfigured apps → Internal apps": the **Trust internal apps** checkbox lets all internal apps reach restricted APIs.
- High-risk scopes in our catalogue:
  - Gmail: `gmail.readonly`, `gmail.send`, `gmail.modify`
  - Drive: `drive`, `drive.readonly`, `drive.meet.readonly`
  - Chat: `chat.messages`, `chat.messages.readonly`
  - `chat.spaces.readonly` and `chat.memberships.readonly` are **not** on the list.
- Pilot action: ask the admin to either tick "Trust internal apps" or set this OAuth client ID to Trusted. Either works.

## 7. `gws auth status`, logout, export

**ANSWER: Yes, it reports both.** `gws auth status` always prints a pretty JSON object and exits 0 (`AC#L1202-L1454`).
- How: it exchanges the refresh token at `oauth2.googleapis.com/token`, then calls `oauth2/v1/userinfo` and fills `user` (email). It calls `oauth2.googleapis.com/tokeninfo` and fills `scopes` (an array of granted scopes, including `openid` and the userinfo scopes) and `scope_count`.
- Other fields:
  - `auth_method`: `oauth2` or `none`
  - `storage`: `encrypted`, `plaintext` or `none`
  - `keyring_backend`
  - `encrypted_credentials` and `encrypted_credentials_exists`
  - `plain_credentials` and `plain_credentials_exists`
  - `token_cache_exists`
  - `client_config` and `client_config_exists`
  - `project_id` and `config_client_id` (masked)
  - `credential_source`
  - `client_id` (masked)
  - `has_refresh_token`
  - `encryption_valid` / `encryption_error`
  - `token_valid` / `token_error`, where `token_error` is Google's `error_description`, e.g. a revoked token
  - `enabled_apis`
- Verified with empty credentials (probe1). The fields that need the network or real credentials (`user`, `scopes`, `token_valid`) are confirmed from source only, so they are **UNVERIFIED live**. A real pilot login would confirm them.
- ⚠ When `project_id` is set, `status` runs `gcloud services list --enabled --project <id> --format=json` (`src/setup.rs` L821-831). That is slow, uses gcloud's own identity, and returns an empty list if gcloud is missing. Writing an empty `project_id` does **not** skip it; gws still runs `gcloud --project ""`. The only way to skip it is to have no `client_secret.json`. API calls don't need that file, because `credentials.enc` already holds the client.
- **`gws auth logout`** (`AC#L1456-L1494`):
  - deletes `credentials.enc`, `credentials.json`, `token_cache.json` and `sa_token_cache.json`, and clears the timezone cache;
  - **does NOT revoke anything at Google**;
  - keeps `client_secret.json` and `.encryption_key`;
  - always exits 0 with `{"status":"success","message":…,"removed":[…]}` (probe2: "No credentials found to remove.").
- **Getting the refresh token to revoke:** `gws auth export --unmasked` prints the decrypted JSON as-is (`AC#L683-L715`): `{"type":"authorized_user","client_id":…,"client_secret":…,"refresh_token":…}`. Then POST `token=<refresh_token>` to `https://oauth2.googleapis.com/revoke`. Without `--unmasked`, the secret and token are shown as `abcd...wxyz`. With no credentials it exits 2 with an authError.
- ⚠ Each `login` sends `prompt=select_account+consent` without `include_granted_scopes` and **replaces** `credentials.enc`. The old refresh token is not revoked, so its grant stays live. This confirms the spec's "revoke first when reducing" design.

## 8. Release assets v0.22.5

**ANSWER:** Published 2026-03-31 (`gh release view v0.22.5 -R googleworkspace/cli --json assets`).

| Platform | Asset | Checksum file |
|---|---|---|
| macOS arm64 | `google-workspace-cli-aarch64-apple-darwin.tar.gz` | `.tar.gz.sha256` |
| macOS x64 | `google-workspace-cli-x86_64-apple-darwin.tar.gz` | `.tar.gz.sha256` |
| Windows x64 | `google-workspace-cli-x86_64-pc-windows-msvc.zip` | `.zip.sha256` |
| (also) | linux gnu and musl for aarch64 and x86_64, `.tar.gz` | yes |

- **Checksum format:** one sha256 file per asset, one line `<64-hex>␠␠<asset-filename>`. The **Windows file ends in CRLF**, so trim it. All three checksums matched (probe3).
- **Layout:** files sit at the archive **root**, with no subfolder:
  - tar entries are `./gws`, `./LICENSE`, `./README.md`, `./CHANGELOG.md`;
  - the zip has `gws.exe`, `LICENSE`, `README.md`, `CHANGELOG.md`.
- The extracted macOS binary is `Mach-O 64-bit executable arm64` and is not signed by us. When downloaded with curl, the app gets no quarantine xattr. Only `com.apple.provenance` was seen, after a `gh` download.
- There are no skills in the release assets.

## 9. gws agent skills

**ANSWER:** Skills live in the repo at `skills/<name>/SKILL.md`, one file per folder, at tag v0.22.5. There are 95 dirs: `gws-*`, `persona-*` and `recipe-*`.
- Shared: `gws-shared`. Each per-service skill starts with "Read `../gws-shared/SKILL.md`", so they must be installed as **siblings**.
- Per service:
  - gmail: `gws-gmail` (+ helpers `gws-gmail-read/-send/-reply/-reply-all/-forward/-triage/-watch`)
  - calendar: `gws-calendar` (+ `-agenda`, `-insert`)
  - chat: `gws-chat` (+ `-send`)
  - meet: `gws-meet`
  - drive: `gws-drive` (+ `-upload`)
  - people: `gws-people`
- Frontmatter: `name`, `description`, `metadata.version: 0.22.5`, `metadata.openclaw.requires.bins: [gws]`.
- How to get them:
  1. Download the repo tarball at the tag: `https://github.com/googleworkspace/cli/archive/refs/tags/v0.22.5.tar.gz`, then `skills/`. **Recommended**, because it is pinned.
  2. `npx skills add https://github.com/googleworkspace/cli` (README L220-239).
  3. `gws generate-skills --output-dir <relative-dir>`. The dir must be relative to the cwd; absolute paths are rejected (`crates/google-workspace/src/validate.rs` L72-83). It also writes `docs/skills.md` into the cwd. It fetches live Discovery docs, so its **output differs from the tagged skills** (probe4: 8 service skills differed). Don't use it for pinned installs.
- **Custom config folder:** no skill mentions `GOOGLE_WORKSPACE_CLI_CONFIG_DIR`. The `gws-shared/SKILL.md` "Authentication" section only says `gws auth login`. The app has to change the copied `gws-shared/SKILL.md`, e.g. add an "Environment" section: "Always run gws as `GOOGLE_WORKSPACE_CLI_CONFIG_DIR=<dir> GOOGLE_WORKSPACE_CLI_KEYRING_BACKEND=file gws …`". A safer option is a wrapper script on PATH that exports both. Agents don't reliably follow prose, so a wrapper is more robust.

## 10. Env vars for prompts, telemetry, update checks

**ANSWER:**
- **No telemetry and no update check exist** in v0.22.5. Grep for `releases/latest`, `analytics`, `telemetry` or `update_check` found nothing in `crates/`, so there is nothing to disable.
- The only interactive prompt is the login scope picker. It is skipped when stdin is not a TTY or when `--scopes` is passed (item 2).
- `NO_COLOR` removes colour codes from stderr labels (`src/output.rs` L51). Colour is already off when stderr is not a TTY.
- Logging is off unless `GOOGLE_WORKSPACE_CLI_LOG` or `GOOGLE_WORKSPACE_CLI_LOG_FILE` is set.
- Remaining stderr noise that can't be turned off: `Using keyring backend: …`.
- Set a controlled cwd so `.env` loading does nothing (item 2).

---

### Implications for the spec / implementation
1. Treat the client as public-ish. There is no PKCE and the secret is stored in `client_secret.json` and `credentials.enc`. Create a **Desktop** OAuth client.
2. Run `gws auth login --scopes <full URLs, comma-separated>` with stdin=/dev/null.
   - Parse stderr (and stdout) for the `https://accounts.google.com/` line and open it with Tauri's opener.
   - Add a cancel button that kills the process.
   - Parse the success JSON from stdout.
3. The client id and secret env vars are only needed for the first login. The app can also write `client_secret.json` itself.
4. Pass `GOOGLE_WORKSPACE_CLI_KEYRING_BACKEND=file` with a custom config folder. **Must** for the pilot (shared keychain item `gws-cli`/`$USER`).
5. For `status()`, use `gws auth status` → `user` and `scopes`. Don't use the `scopes` that login returns.
6. For `revoke_all`, run `gws auth export --unmasked` → read `refresh_token` → POST to `/revoke` → `gws auth logout`.
7. Meet Read = `meetings.space.readonly` only.
8. Admin: "Trust internal apps" or trust this client ID.

### UNVERIFIED (needs a real pilot login)
- Live `auth status` output (`user`, `scopes`, `token_valid`) and the success JSON of `login`.
- Whether the keychain collision actually breaks the existing `~/.config/gws` setup. Not tested on purpose: it would need that setup's keychain item. A test on a throwaway macOS user would settle it.
