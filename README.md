# haulmer-connect

A small desktop app (Windows `.exe`, macOS `.dmg`) that lets a Haulmer employee give their
local AI agents (Claude Code, Codex, Hermes, …) access to their own Google Workspace data:
Gmail, Calendar, Chat, Meet and Drive. One screen with a Read and a Write checkbox per
service, **Save**, **Revoke all** and **Install in my agents**.

The app delegates login, token storage, refresh and all API calls to the
[Google Workspace CLI](https://github.com/googleworkspace/cli) (`gws`, pinned to v0.22.5),
using one shared, Internal OAuth client. No Haulmer server is involved; tokens never leave
the machine. Full design: [`docs/spec.md`](docs/spec.md).

## Layout

| Path | What |
|---|---|
| `crates/core` | `haulmer_connect_core`: all behaviour behind `Core` and two ports (`ProcessRunner`, `HttpClient`) |
| `src-tauri` | Tauri v2 shell: one window, IPC commands that call `Core` 1:1, no local HTTP server |
| `src` | UI: React 19 + TypeScript + Tailwind CSS v4, built with Vite |
| `docs` | Spec, research, tickets |

## Requirements

- Rust (stable) and the [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/)
- Node.js and pnpm

## Run

```sh
pnpm install
pnpm tauri dev
```

## MCP

The app doubles as an MCP server with one tool, `gws`. The **Use from desktop AI apps**
buttons (**Add to Claude Desktop**, **Add to ChatGPT**) write the config for you. To add it by
hand, point the client at the installed app with the argument `mcp`
(`/Applications/haulmer-connect.app/Contents/MacOS/haulmer-connect` on macOS):

```jsonc
// Claude Desktop: ~/Library/Application Support/Claude/claude_desktop_config.json
{ "mcpServers": { "haulmer-connect": { "command": "<path to haulmer-connect>", "args": ["mcp"] } } }
```

```toml
# ChatGPT desktop, Codex CLI and IDE extension: ~/.codex/config.toml
[mcp_servers.haulmer-connect]
command = "<path to haulmer-connect>"
args = ["mcp"]
```

Restart the app afterwards. `auth` commands are refused over MCP; sign in with the app first.

## Test

```sh
cargo test --workspace     # core behaviour with FakeRunner, FakeHttp and temp dirs
cargo clippy --workspace
pnpm build                 # type-check and build the UI
```

Core tests drive the public `Core` API with the shared test doubles in
`haulmer_connect_core::testing` (`FakeRunner`, `FakeHttp`), enabled for integration tests by
the `testing` feature.

## Release

CI (`.github/workflows/ci.yml`) runs `cargo test --workspace` and `pnpm build` on every pull
request and push to `main`. Pushing a `v*` tag runs `.github/workflows/release.yml`, which
builds a Windows x64 NSIS installer (`.exe`) and macOS `.dmg` files for Apple Silicon
(`aarch64`) and Intel (`x64`), writes a `.sha256` file per installer plus `SHA256SUMS.txt`,
signs the update files (`.app.tar.gz`, `*-setup.exe`) with the updater key, generates
`latest.json`, and attaches everything to a **draft** GitHub Release. The app reads
`latest.json` from the latest **published** release, so users see the update only after you
publish the draft.

Two repository secrets are required (Settings → Secrets and variables → Actions); the build
fails without them:

- `TAURI_SIGNING_PRIVATE_KEY`: contents of the updater private key.
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: its password.

The private key lives outside the repo; only the public key is committed
(`plugins.updater.pubkey` in `src-tauri/tauri.conf.json`). **If you lose the key, installed
apps can no longer update**: users must download and reinstall a new build by hand (a new
key means a new `pubkey`). A local `pnpm tauri build` also needs `TAURI_SIGNING_PRIVATE_KEY`
(or `TAURI_SIGNING_PRIVATE_KEY_PATH`) set, because updater files are always created.

To cut a release:

1. Bump `version` to the same value in `package.json`, `Cargo.toml` (`workspace.package`) and
   `src-tauri/tauri.conf.json`; merge to `main`. The release workflow fails if the tag does not
   match `tauri.conf.json`.
2. Tag and push:
   ```sh
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. Wait for the **Release** workflow, then review the draft under GitHub → Releases and
   publish it.

Builds are **not code-signed** (pilot). macOS users must right-click → Open the first time
(or run `xattr -dr com.apple.quarantine /Applications/haulmer-connect.app`); Windows
SmartScreen needs **More info → Run anyway**. Users can verify a download with
`shasum -a 256 -c <file>.sha256` (macOS) or `Get-FileHash <file>` (PowerShell).
