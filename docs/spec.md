---
title: haulmer-connect — self-service Google Workspace access for AI agents
status: ready-for-agent
date: 2026-11-16
---

# haulmer-connect — spec

## Problem Statement

People at Haulmer want their AI agents (Claude Code, Codex, Hermes and similar local agents) to read and act on their own Google Workspace data: Gmail, Calendar, Chat, Meet and Drive. Today each person must create an OAuth app in the Google Cloud console, enable APIs, configure a consent screen and download a client secret. This is too hard for most people, and it is slow and error-prone even for engineers. The result is that only a few technical people connect their agents, each with their own ad-hoc setup.

The company also needs this to be safe. Haulmer runs a financial product, so tokens must stay on each person's machine, each person must decide which services an agent can use, and each person must be able to remove that access at any time.

## Solution

A small desktop app, `haulmer-connect`, that a person downloads as a normal app (`.exe` installer on Windows, `.dmg` on macOS).

The app shows one screen with a checkbox per Google service, in two levels: **Read** and **Write**. The person ticks what their agents may do and clicks **Save**. The app opens Google's sign-in page, the person approves, and the connection is ready. The same screen lets the person add services later, remove a service, or revoke everything.

Behind the screen, the app uses one shared OAuth client that Haulmer creates once (Desktop app type, Internal audience), so nobody else touches the Google Cloud console. The app delegates login, token storage, token refresh and all API calls to the Google Workspace CLI (`gws`). Agents use `gws` commands and the `gws` agent skills. The tray item **Conectar un agente…** gives the person an install prompt to paste into an agent. The agent connects itself to the app and installs the skills.

No Haulmer server is involved. Tokens never leave the person's machine.

## User Stories

### Install and first run

1. As a Haulmer employee on Windows, I want to download a normal `.exe` installer, so that installing feels like any other app.
2. As a Haulmer employee on macOS, I want to download a `.dmg` with a normal app, so that installing feels like any other app.
3. As a Haulmer employee without `gws` installed, I want the app to get `gws` for me, so that I do not need a terminal or a package manager.
4. As a Haulmer employee, I want the app to verify the `gws` download against its published SHA-256 checksum, so that a tampered binary is never run.
5. As a Haulmer employee, I want a clear error if the `gws` download or checksum fails, so that I know the app did not continue in an unsafe state.
6. As a Haulmer employee who already has `gws` installed, I want the app to use my existing `gws` binary if it is the pinned version, so that I do not get a second copy.
7. As a Haulmer employee, I want to see which Google account is connected (or that none is), so that I know the current state at a glance.

### Choosing services

8. As a Haulmer employee, I want one checkbox per service (Gmail, Calendar, Chat, Meet, Drive), so that I can choose exactly what my agents can reach.
9. As a Haulmer employee, I want each service to have a Read level and a separate Write level, so that I can let an agent read my email without letting it send email.
10. As a Haulmer employee, I want Read for Gmail, Calendar, Chat and Meet to be ticked by default, so that the common setup takes one click.
11. As a Haulmer employee, I want Drive and every Write level to be unticked by default, so that the default is the safest useful setup.
12. As a Haulmer employee, I want a short plain-language description next to each checkbox (for example "Write: send and modify email"), so that I understand what I grant without knowing OAuth scope names.
13. As a Haulmer employee, I want ticking Write to also tick Read for that service, so that I never end up with a configuration that cannot work.

### Connecting and changing access

14. As a Haulmer employee, I want **Save** to open Google's sign-in page with exactly the scopes I selected, so that Google's consent screen matches my choices.
15. As a Haulmer employee, I want the app to show success or a clear error after Google's sign-in, so that I know whether my agents can now work.
16. As a Haulmer employee, I want to add a service later by ticking it and clicking **Save**, so that I can grow access when I need it.
17. As a Haulmer employee, I want unticking a service and clicking **Save** to really remove that access, so that "unticked" always means "the agent cannot use it".
18. As a Haulmer employee, I want a small, subtle note under **Save** that says a Google sign-in will follow when I remove a service, so that the extra sign-in does not surprise me.
19. As a Haulmer employee, I want a **Revoke all** button, so that I can cut every agent's access in one step.
20. As a Haulmer employee, I want **Revoke all** to revoke the token at Google and also clear local credentials, so that a copy of the token anywhere is useless.
21. As a Haulmer employee, I want a link to Google's own "Third-party apps" page, so that I can also check and revoke access from Google directly.

### Existing gws setups and config location

22. As a Haulmer employee who already has `gws` credentials in the default config folder, I want the app to ask "You already have gws connected with another app. Replace?" before it changes anything, so that I do not lose a working setup by accident.
23. As a Haulmer employee who answers "No" to the replace question, I want the app to leave my existing credentials untouched, so that nothing breaks.
24. As an advanced user, I want to choose a different `gws` config folder under **Advanced**, so that I can keep `haulmer-connect` separate from another `gws` setup.
25. As an advanced user, I want the app to remember my chosen config folder, so that I set it only once.

### Agents

26. As a Haulmer employee, I want a tray item **Conectar un agente…** that shows an install prompt with a **Copiar** button, so that my agent connects itself to haulmer-connect and learns how to use Gmail, Calendar, Chat and Meet without me editing config files or copying skill files.
27. As a Haulmer employee, I want the install prompt to install the core `gws` service skills (the "Services" table of gws `docs/skills.md`), so that my agents learn every service once, even ones I enable later. If `npx` is not available, I want the agent to say so and point me to the ZIP in the Skills window.
28. As a Haulmer employee using a custom config folder, I want the MCP server to pass that folder to every `gws` call, so that agents find my credentials.
29. As an agent, I want to run `gws` commands (for example `gws gmail +send`, `gws calendar +agenda`) that return JSON, so that I can act on the person's data reliably.
30. As an agent, I want a clear error when `gws` is missing, no account is connected, the session expired or a permission is missing, so that I can tell the person exactly what to click or tick in `haulmer-connect`. I also want a tool that opens the app for them.

### Company and admin

31. As the owner of the shared OAuth client, I want the client to live in a dedicated Google Cloud project with two owners, so that the company does not depend on one person.
32. As the owner, I want the client to be Internal, so that only haulmer.com accounts can grant access.
33. As a Workspace admin, I want to see which users authorized the app and which scopes they granted using the existing Admin console reports, so that I get visibility without new privileges or new infrastructure.
34. As a Workspace admin, I want to be able to block or trust this client in the Admin console, so that the company keeps final control.
35. As the project owner, I want the repository to be public, so that anyone can audit what the app does with their Google access.

## Implementation Decisions

### Architecture

- **Desktop shell:** Tauri v2. The UI runs in the app's own window, not in a browser. The UI talks to the Rust core through Tauri IPC commands only. There is no local HTTP server, so other websites cannot send requests to the app.
- **UI:** React + TypeScript + Tailwind CSS v4, built with Vite (latest versions). The UI holds no logic about scopes or tokens; it renders state from the core and sends the person's selection.
- **Rust core:** a small module with the whole behaviour behind three commands:
  - `status()`: returns whether `gws` is available, which account is connected, which levels are currently granted, and the active config folder.
  - `save(selection)`: maps the selection to scopes, revokes first if any previously granted level was removed, then runs the `gws` login with the new scope set.
  - `revoke_all()`: revokes the refresh token at Google's revoke endpoint, then runs `gws auth logout`.
  - Plus `set_config_dir(path)` for the Advanced setting. The core also builds the install prompt for the **Conectar un agente** window.
- **Running `gws`:** every call runs with a cwd the app controls (gws loads `.env` from its cwd), stdin closed, and `NO_COLOR=1`.
- **Login flow:** gws v0.22.5 prints the Google sign-in URL to stderr (stdout when a proxy env var is set) and does **not** open the browser. The core reads the first `https://accounts.google.com/` line from either stream and opens it with the system browser. gws waits for the loopback callback with no timeout, so the UI has a **Cancel** button that kills the process. Exit 0 prints a success JSON; exit 1-5 prints `{"error":{...}}` (2 = auth, 3 = validation).
- **Status:** `status()` reads `gws auth status` (JSON: `user`, `scopes`, `token_valid`, `encrypted_credentials_exists`), because the `scopes` printed by login are the requested scopes, not the granted ones.
- **Engine:** the Google Workspace CLI (`gws`, github.com/googleworkspace/cli), pinned to **v0.22.5**. `gws` does the OAuth login, stores credentials encrypted (AES-256-GCM, key in the OS keyring), refreshes tokens, and provides the commands and skills agents use. The app passes the shared client through `GOOGLE_WORKSPACE_CLI_CLIENT_ID` and `GOOGLE_WORKSPACE_CLI_CLIENT_SECRET`, and the config folder through `GOOGLE_WORKSPACE_CLI_CONFIG_DIR` when it is not the default.
- **Getting `gws`:** if the pinned `gws` is not on the machine, the core downloads the release asset for the platform from GitHub Releases, verifies it against the published `.sha256` file, and stores it in the app's data folder. A checksum mismatch stops the flow with an error. The app never runs an unverified binary.
- **Revocation:** `gws auth logout` only deletes local files, so the core also calls Google's revoke endpoint (`https://oauth2.googleapis.com/revoke`). Revoking a token revokes the whole grant. The core gets the refresh token from `gws auth export --unmasked` (JSON field `refresh_token`), POSTs `token=<refresh_token>`, then runs `gws auth logout`. Each new login replaces the stored token without revoking the old one, which is why `save` revokes first when it removes a level.
- **`gws` release assets (v0.22.5):** `google-workspace-cli-aarch64-apple-darwin.tar.gz`, `google-workspace-cli-x86_64-apple-darwin.tar.gz`, `google-workspace-cli-x86_64-pc-windows-msvc.zip`, each with `<asset>.sha256` holding `<hex>  <filename>` (the Windows file ends in CRLF). The binary is at the archive root (`gws` / `gws.exe`).
- **Removing one service:** Google cannot revoke a single scope. When the new selection removes any granted level, `save` revokes everything and then signs in again with the reduced set (the UI shows a subtle note about the extra sign-in).

### OAuth client

- One shared OAuth client, type **Desktop app**, audience **Internal** (haulmer.com only), in a dedicated Google Cloud project `haulmer-connect` with two owners. It is marked as a client for an AI-powered agent.
- **Created 2026-11-16:** project `haulmer-connect` (number 252304543508), client ID `252304543508-fu07n7mngakrpuvmvdo28rek975lkccn.apps.googleusercontent.com`. Second owner: pending. gws uses no PKCE and sends the client secret, so the client must stay a Desktop client.
- The client is a public client (RFC 6749 §2.1, RFC 8252). Its client ID and secret ship inside the app and in the public repository; they are not treated as secrets (RFC 8252 §8.5). Only haulmer.com accounts can consent, and a Desktop client only redirects to loopback.
- APIs enabled in the project: Gmail, Calendar, Drive, Chat, Meet, People. A Chat app must be configured once in the project for Chat write access.
- Internal apps need no Google verification. If the Workspace admin restricts high-risk scopes (Gmail, Drive, Chat), the admin must mark this client as trusted.

### Scope catalogue

The core owns the mapping from the person's selection to scopes. All scopes are under `https://www.googleapis.com/auth/`.

| Service | Read (default) | Write (opt-in) | Default ticked |
|---|---|---|---|
| Gmail | `gmail.readonly` | `gmail.send`, `gmail.modify` | Read |
| Calendar | `calendar.readonly` | `calendar.events` | Read |
| Chat | `chat.spaces.readonly`, `chat.messages.readonly`, `chat.memberships.readonly` | `chat.messages` | Read |
| Meet | `meetings.space.readonly` (also covers transcript entries) | `meetings.space.created` | Read |
| Drive | `drive.readonly` | `drive` | none |
| Docs | `documents.readonly` | `documents` | none |
| Sheets | `spreadsheets.readonly` | `spreadsheets` | none |
| Slides | `presentations.readonly` | `presentations` | none |
| Forms | `forms.body.readonly` | `forms.body` | none |
| Tasks | `tasks.readonly` | `tasks` | none |
| People | `contacts.readonly` | `contacts` | none |

The catalogue also displays Admin Reports, Admin Directory, Admin Groups and Cloud Identity.
Those are disabled unless the account has Workspace administrator or delegated administrator
permissions, and their write operations are intentionally not exposed by the personal-access
model. YouTube, Google Ads, Analytics, Search Console, Business Profile and Merchant Center
are separate business connectors: they reuse the OAuth account but do not use `gws` skills.

Scopes are passed to `gws auth login --scopes` as **full URLs, comma-separated** (gws does not expand short names). gws always adds `openid`, `userinfo.email` and `userinfo.profile`; the core ignores these three when it compares granted levels.

Write always implies Read for the same service.

### Config folder

- Default: the `gws` default folder (`~/.config/gws`), so agents and `gws` skills work with zero setup.
- If credentials already exist there and were not created by `haulmer-connect`, the app asks before replacing them. "No" leaves them untouched.
- The folder is configurable under **Advanced** and persisted by the app. When it is not the default, the app passes it to every `gws` call, including the calls from the MCP server.
- **Key storage** (`GOOGLE_WORKSPACE_CLI_KEYRING_BACKEND`): configurable under **Advanced** as `auto` (default), `keyring` or `file`. `auto` means `keyring` for the default folder and `file` for a custom folder. Reason: with `keyring`, gws v0.22.5 keeps one keychain item (service `gws-cli`, account `$USER`) for every config folder, so a custom folder can collide with another gws setup on the same machine. The resolved backend is passed to every `gws` call.
- "Credentials exist" means `<dir>/credentials.enc` or `<dir>/credentials.json` exists.

### Agents

- Agents use `gws` commands and the `gws` skills from the pinned `gws` release.
- The app never writes agent config files and never writes into agent skill folders. Agents connect themselves, or the person follows manual steps.
- **Connecting an agent.** The tray item **Conectar un agente…** (one item, no submenu) opens a window with the same name. The window shows short steps, the **install prompt** in a `<pre>`, and a **Copiar** button. The prompt is shown even when no Google account is connected.
- The core builds the prompt in Rust each time the window opens. It contains the absolute executable path (`std::env::current_exe()`) and these parts:
  - Claude Code: `claude mcp add --scope user haulmer-connect -- "<exe>" mcp`, checked with `claude mcp get haulmer-connect`.
  - Codex: `codex mcp add haulmer-connect -- "<exe>" mcp`, checked with `codex mcp get haulmer-connect`.
  - Other agents (Hermes, Cursor): a generic stdio entry.
  - Skills step (optional): `npx skills add`, pinned to gws v0.22.5, with the 19 skills of the "Services" table in gws `docs/skills.md` (`gws-shared` plus one core skill per Google API; no helpers, personas or recipes). The list does not depend on the granted services. If `npx` fails, the agent says so and points to the ZIP.
  - Manual steps for Chat apps: Claude Desktop (Edit Config, with a JSON block) and ChatGPT desktop (Settings > MCP servers).
  - A statement that Web chats (claude.ai, chatgpt.com) cannot work.
- **Path checks (macOS).** If the app runs from a translocated path (`/AppTranslocation/`, so it was not moved to Applications), the window shows only an error that asks the person to move it. If the app runs from `/Volumes/` (the mounted DMG), the window shows a warning and the prompt.
- **Skills come from two places:** the install prompt, or the Skills window (**Abrir skills**). The Skills window has the ZIP (**Descargar .zip**) and, under **Avanzado**, the same `npx skills add` command as the prompt. The ZIP covers only the granted services; the command always lists the core service skills.
- Skill source: the `skills/` folder of the gws repository at tag `v0.22.5` (`https://github.com/googleworkspace/cli/archive/refs/tags/v0.22.5.tar.gz`). Skills are not in the release assets, and `gws generate-skills` is not pinned. Per service: `gws-shared` always; `gws-gmail*`, `gws-calendar*`, `gws-chat*`, `gws-meet`, `gws-drive*`. Each skill reads `../gws-shared/SKILL.md`, so the shared skill must be installed with them.
- When the config folder or key storage is not the default, the `gws-shared/SKILL.md` in the ZIP gets an "Environment" section that tells agents to export `GOOGLE_WORKSPACE_CLI_CONFIG_DIR` and `GOOGLE_WORKSPACE_CLI_KEYRING_BACKEND` before every `gws` call. Skills installed with `npx skills add` are not edited. The MCP `gws` tool sets both variables itself.
- Accepted risk: any agent that can run `gws` can also run `gws auth export --unmasked` and read the refresh token. This is accepted for the pilot and must be reviewed before company-wide rollout.

### MCP

- `haulmer-connect mcp` runs an MCP server on stdin/stdout (newline-delimited JSON-RPC 2.0, no window). It uses the same folders as the app (home and `<data dir>/com.haulmer.connect`).
- Tool `gws`, with input `{args: string[]}`. It runs the pinned `gws` through the same environment as the app and returns stdout (stderr when stdout is empty); a non-zero exit sets `isError`. It never downloads gws.
- **Errors tell the next step.** When a `gws` call fails, the server checks the app status and returns a Spanish message with the next step. The message quotes the exact UI labels (**Abrir configuración**, **Conectar** or **Guardar**). The cases are: `gws` missing, no account, expired session, and a missing permission for the service. In any other case the server returns the raw `gws` output, plus a hint about the Write level when the service is read-only.
- Tool `open_haulmer_connect` (no arguments) opens the desktop app on its settings window. The error messages suggest it.
- `auth` commands are blocked over MCP (`args[0] == "auth"`), so a model cannot call `gws auth export --unmasked` through this tool. Login and revoke stay in the app. This does not close the accepted risk above: an agent with a shell can still run `gws` directly.
- **Single instance.** The app uses `tauri-plugin-single-instance` 2.5.5. A second launch (from Finder, the Start menu, or `open_haulmer_connect`) shows the settings window instead of starting a second tray icon. The MCP server launches the app with a `--show` argument. The `mcp` subcommand itself is dispatched before Tauri starts (`docs/adr/0001-mcp-dispatch-before-tauri.md`), so it never starts a second instance.

### Distribution

- Public repository `github.com/haulmer/haulmer-connect`.
- CI builds a Windows x64 installer (`.exe`) and macOS `.dmg` files for Apple Silicon and Intel, and publishes them with SHA-256 checksums to GitHub Releases.
- The pilot builds are **not code-signed**. Signing (Apple Developer ID + notarization, Windows code-signing certificate) is a precondition for company-wide rollout.
- In-app updates use the Tauri updater plugin with a static `latest.json` on GitHub Releases (`https://github.com/haulmer/haulmer-connect/releases/latest/download/latest.json`, so it follows the latest published release). The release workflow builds it with the update files and their signatures (`darwin-aarch64`, `darwin-x86_64`, `windows-x86_64`).
- Update files are signed with a minisign key. The public key is committed in `tauri.conf.json`; the private key and its password are CI secrets (`TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`) and a copy is kept outside the repo. Losing it means installed apps cannot update; users reinstall by hand.
- The app checks for an update on start. A failed check is silent. When one exists, a banner offers "Install and restart"; nothing installs without the person's click. This signature check is separate from code signing: it proves an update came from the Haulmer key, but the OS still shows the unsigned-app warnings on first install.

### Third-party due diligence (checked 2026-11-16)

Every dependency was checked for deprecation, last release, popularity and known vulnerabilities before selection. Repeat this check before adding any new dependency.

- tauri 2.12.6 (113k stars, release 2026-11-16; all recorded advisories fixed in earlier versions)
- React 19.3.1, Vite 8.3.8, Tailwind CSS 4.3.5: active, 0 known vulnerabilities in the current version
- gws 0.22.5: 32k stars, 0 known vulnerabilities; README states it is **not an officially supported Google product**; last release March 2026
- Rust crates approved by the owner 2026-11-16 (crates.io + osv.dev checked that day; 0 advisories, none yanked): serde 1.0.229, serde_json 1.0.154, thiserror 2.0.21, sha2 0.11.1, ureq 3.5.2, flate2 1.1.11, tar 0.4.46, zip 9.6.1, tauri-plugin-opener 2.7.3, tauri-plugin-dialog 2.8.4, tauri-plugin-updater 2.14.1, tauri-plugin-process 2.4.0; dev: tempfile 3.27.0. npm: @tauri-apps/api 2.12.5, @tauri-apps/plugin-opener 2.7.3, @tauri-apps/plugin-dialog 2.8.4, @tauri-apps/plugin-updater 2.14.1, @tauri-apps/plugin-process 2.4.0. The updater and process plugins were approved by the owner 2026-11-16 (crates.io, npm and osv.dev checked that day; 0 advisories). Any other dependency needs the same check and the owner's OK.
- `tauri-plugin-single-instance` 2.5.5 approved by the owner 2026-11-18 (crates.io and osv.dev checked that day): released 2026-10-28, official tauri-apps plugin, about 12.8M downloads, 0 advisories. It supports macOS and Windows and needs Rust 1.90 or newer (the toolchain is 1.99.1). Rust only, no npm package.
- Toolchain dependencies approved by the owner 2026-11-16: npm dev @tauri-apps/cli, @vitejs/plugin-react, @tailwindcss/vite, typescript 7.x, @types/react, @types/react-dom; cargo build-dep tauri-build.
- Rejected: `reqwest` (21 dependencies, async; `ureq` is enough behind the HTTP port), `hex` (inline hex encoding instead), `arctic` (deprecated), `googleapis` npm (246 MB), `tiny_http` (no release since 2022)

## Testing Decisions

- **Good tests check external behaviour only:** what the person sees and what the app does to the outside world (which `gws` commands run with which arguments and environment, which HTTP calls go to Google, which files are written). They do not check internal functions or data structures.
- **One seam (confirmed by the owner 2026-11-16):** the public Rust core API (`status`, `save`, `revoke_all`, `set_config_dir`) with two injected ports:
  - a **process runner** for `gws` (a fake records the command, arguments and environment, and returns canned output and exit codes);
  - an **HTTP client** for the `gws` download, the checksum file and the revoke call (a fake returns canned responses).
  - a **real filesystem in a temp dir**: the home folder, the config folder and the app data folder are paths injected into the core, so tests check written files directly.
  Tests call the core API exactly as the Tauri commands do. The Tauri layer and the React UI stay thin enough that they need no tests in the pilot.
- **Behaviours to cover:**
  - selection → scopes mapping, including "Write implies Read" and the defaults;
  - `save` that only adds scopes runs login once with the full new set and does not revoke;
  - `save` that removes any granted level revokes first, then runs login with the reduced set;
  - `revoke_all` calls the revoke endpoint and `gws auth logout`, and still clears local credentials if Google is unreachable (reporting the error);
  - checksum mismatch on download stops the flow and never runs the binary;
  - existing credentials in the default folder trigger the replace question, and "No" changes nothing;
  - a custom config folder is passed to every `gws` call;
  - the install prompt contains the executable path and the commands for Claude Code, Codex and other agents;
  - the install prompt returns only an error for a `/AppTranslocation/` path, and a warning plus the prompt for a `/Volumes/` path;
  - a failed MCP `gws` call returns the Spanish next-step message for each case (`gws` missing, no account, expired session, missing permission), and the raw output otherwise.
- **Prior art:** none; this is a new repository.
- **Manual pilot checks** (cannot be automated): real Google sign-in, Admin console log entries, and the success criteria in Further Notes.

## Out of Scope

- A local MCP server that hides tokens from agents (possible v2).
- Google's official remote Workspace MCP servers (developer preview, no Meet, Gmail cannot send).
- Domain-wide delegation or any service account impersonation.
- Any Haulmer server that stores or proxies tokens.
- Usage statistics inside the app; admin visibility comes from the Workspace Admin console only.
- Silent auto-install, code signing, Linux builds and Windows ARM builds (all for company-wide rollout).
- Commands of our own for Google APIs; agents use `gws` commands.
- Any app code that writes agent config files or agent skill folders.

## Further Notes

### Pilot

- Pilot user: the project owner only, on macOS, with a custom config folder (because that machine already has another `gws` setup that must not be touched).
- Success criteria:
  1. Connect in under 5 minutes without help.
  2. An agent reads Gmail, Calendar, Chat and Meet, and sends one email after Gmail Write is ticked.
  3. Unticking a service removes the agent's access to it; **Revoke all** removes all access.
  4. No token leaves the machine; the grants appear in the Workspace Admin console OAuth log.
- Not covered by the pilot: Windows and a non-technical person. Test both before company-wide rollout.

### Verified facts (spike 2026-11-16, gws v0.22.5 source + probes)

1. Login uses a loopback redirect (`http://localhost:<random port>`, bound on 127.0.0.1) and **no PKCE**.
2. `--scopes` takes full URLs, comma-separated. With `--scopes`, login never prompts and never reads stdin. gws prints the sign-in URL and does not open the browser.
3. The client ID and secret are only needed for the first login: gws stores them in `client_secret.json` and inside `credentials.enc`. Agents do not need the env vars.
4. Meet transcripts: `meetings.space.readonly` covers listing transcripts and reading their entries. `drive.meet.readonly` is only for downloading the Docs file or recording. Not added.
5. Drive Write is `drive`. `drive.file` needs a file picker, so an agent could only touch files gws created.
6. The admin must trust the client ("Trust internal apps" or trust this client ID) only if Gmail, Drive or Chat are restricted in the Admin console. `chat.spaces.readonly` and `chat.memberships.readonly` are not high-risk.
7. `gws auth status` returns JSON with `user` and the granted `scopes`, read live from Google.
8. Still unverified (needs the real pilot login): the live `auth status` and login success JSON.
9. Admin console item for the pilot: ask the Workspace admin to tick "Trust internal apps" or trust client `252304543508-fu07n7mngakrpuvmvdo28rek975lkccn.apps.googleusercontent.com` if a restriction blocks the login.

Full notes with source links: `docs/research/gws-spike.md`.

### Known limits

- Google keeps at most 100 refresh tokens per account per client; the oldest is dropped silently.
- A refresh token dies after revocation, 6 months unused, or a password change while Gmail scopes are granted. The person then saves again.
- All users share the project's per-API quotas.
- Windows: `gws.exe` runs without `CREATE_NO_WINDOW`, so a console window may flash on each `gws` call. Known follow-up, not in the single-instance and install-prompt change.
