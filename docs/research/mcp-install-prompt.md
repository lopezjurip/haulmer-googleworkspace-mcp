# haulmer-connect: MCP install prompt research

Date: 2026-11-18. Sources: Tauri `tauri-v2.12.6` (the crate in `Cargo.lock`), `tauri-cli-v2.12.4` (the CLI in `node_modules`, bundler 2.10.4), `tauri-utils-v2.10.3`, plugins-workspace `cli-v2.5.0` (commit `a2364a5f`, also holds single-instance 2.5.0) and `updater-v2.14.1`, Rust `1.99.1` std, libuv `v1.53.0`, MCP TypeScript SDK `v1.29.0`, MCPB `v2.1.2`, Codex `rust-v0.169.1`, VS Code `1.147.0`, plus official docs fetched on 2026-11-18.
Short links:
- `TA` = https://github.com/tauri-apps/tauri/blob/tauri-v2.12.6/crates/tauri/src/app.rs
- `CLI` = https://github.com/tauri-apps/plugins-workspace/blob/cli-v2.5.0/plugins/cli/src
- `SI` = https://github.com/tauri-apps/plugins-workspace/blob/cli-v2.5.0/plugins/single-instance
- `NSI` = https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi
- `TU` = https://github.com/tauri-apps/tauri/blob/tauri-utils-v2.10.3/crates/tauri-utils/src
- `STD` = https://github.com/rust-lang/rust/blob/1.99.1/library/std/src
- `UV` = https://github.com/libuv/libuv/blob/v1.53.0/src/win/process.c
- `SDK` = https://github.com/modelcontextprotocol/typescript-sdk/blob/v1.29.0/src/client/stdio.ts
- `MCPB` = https://github.com/modelcontextprotocol/mcpb/blob/v2.1.2
- `OAI` = https://learn.chatgpt.com/docs/extend/mcp (the target of a 308 redirect from https://developers.openai.com/codex/mcp)
- `CC` = https://code.claude.com/docs/en/mcp

Local probes (read-only): `claude --version` (2.1.311), `claude mcp --help`, `claude mcp add --help`, `claude mcp add-json --help`, `codex --version` (0.167.2), `codex mcp --help`, `codex mcp add --help`, `defaults read` of installed apps' `Info.plist`, a file listing inside `/Applications/ChatGPT.app`, `man 3 dyld`. No config file was read or written, and no `mcp add` ran.

Release targets: `.github/workflows/release.yml:26-39` builds Windows x64 NSIS and macOS DMGs for Apple Silicon **and Intel**. Nothing in this note depends on the CPU.

**Summary**
- The app already knows its executable path (`src-tauri/src/lib.rs:211`, `:223`). The prompt is incomplete because that path never reaches the prompt text. The tray emits `copy-install-prompt` with no payload (`lib.rs:259-263`), and the webview copies a fixed string (`src/App.tsx:42`, `:363-370`).
- `tauri-plugin-cli` does not fix this. It does not expose the path. It also needs a built Tauri app before it can parse arguments, which is worse for a stdio subcommand. Keep the hand-rolled dispatch in `src-tauri/src/main.rs:13-22`.
- Fix: build the prompt in Rust with `std::env::current_exe()`. Do not copy it when the path contains `/AppTranslocation/`. Copy it with a warning when the path starts with `/Volumes/`. See section 7.

---

## 1. Is `current_exe()` a stable path to give to an MCP client?

**ANSWER: Yes for an installed app. No for an app that macOS translocated or that runs from the mounted DMG.** Installed paths survive relaunches and in-app updates:
- macOS: `/Applications/haulmer-connect.app/Contents/MacOS/haulmer-connect`
- Windows, default per-user install: `%LOCALAPPDATA%\haulmer-connect\haulmer-connect.exe`
- Windows, per-machine install: `%ProgramFiles%\haulmer-connect\haulmer-connect.exe`

The app can detect the two bad macOS cases only with a string check on the path. Apple has no supported API for this. Use plain `std::env::current_exe()` and never canonicalize the result on Windows.

**What `current_exe()` returns**
- The std docs say that, through a symlink, "some platforms will return the path of the symbolic link and other platforms will return the path of the symbolic link's target" ([`STD/env.rs#L710-L757`](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/env.rs#L710-L757)).
- macOS calls `_NSGetExecutablePath` ([`STD/sys/paths/unix.rs#L296-L313`](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/sys/paths/unix.rs#L296-L313)). `man 3 dyld` says it returns "a path" to the executable, not "a real path", so the result "may be a symbolic link".
- Windows calls `GetModuleFileNameW` ([`STD/sys/paths/windows.rs#L98-L100`](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/sys/paths/windows.rs#L98-L100)). The NSIS installer creates no symlinks.

**macOS App Translocation** (primary source: Apple DTS, Quinn, ["App Translocation Notes"](https://developer.apple.com/forums/thread/724969), Feb 2023)
- "When you run a newly downloaded app, the system executes the app from a randomised path."
- Example path: `/private/var/folders/wk/…/T/AppTranslocation/C863FADC-…/d/NetNewsWire.app/Contents/MacOS/NetNewsWire`.
- "The exact circumstances where the system translocates an app is not documented and has changed over time."
- Moving the app with Finder "has cleared the state that triggered app translocation".
- ⚠ **Correction to the brief:** in Quinn's test, the app was quit and relaunched, and the *same* randomized path (same UUID) came back. So "does not survive a relaunch" does not match the source. The real problems are these:
  - the path is in a system temp folder;
  - it stops being used as soon as the person moves the app to `/Applications`, which they should do anyway;
  - **UNVERIFIED:** whether the translocated path still exists while the app is *not* running. That is exactly when an MCP client spawns it. Apple does not document the lifetime of the translocation mount.
- **Detection: no supported API.**
  - The DTS note says: "There is no supported way to detect if your app is being run translocated."
  - `SecTranslocateIsTranslocatedURL` and `SecTranslocateCreateOriginalPathForURL` exist as symbols in `Security.framework/…/Security.tbd` (local SDK 26.5). No `SecTranslocate.h` header ships in the local 26.5 or 27.0 SDKs.
  - DTS on these functions ([thread 728725](https://developer.apple.com/forums/thread/728725), Apr 2023): "For something to be a public API it must appear in a system header" and "Stuff you find in Darwin is not considered API."
  - **Use a path string check instead.** A miss costs little: the person gets a prompt with a fragile path, which is today's risk anyway. Two outcomes:
    - the path contains `/AppTranslocation/`: do not copy the prompt. Ask the person to move the app to `/Applications` and open it again;
    - the path starts with `/Volumes/`: copy the prompt, but show a warning. An app installed on an external drive also lives under `/Volumes/` and works, so a refusal there would be a false positive.

**Running from the mounted DMG**
- A `/Volumes/<volume>/haulmer-connect.app/...` path exists only while the image is mounted. Any client config that points there breaks after the person ejects the image.
- **UNVERIFIED:** whether macOS translocates an app that is launched from a quarantined, unsigned DMG. The DTS note does not mention disk images. The path check above covers both outcomes.

**Windows NSIS**
- haulmer-connect sets no `bundle.windows.nsis` options (`src-tauri/tauri.conf.json`). So the install mode is the default, `CurrentUser` ([`TU/config.rs#L821-L843`](https://github.com/tauri-apps/tauri/blob/tauri-utils-v2.10.3/crates/tauri-utils/src/config.rs#L821-L843)).
- Default folders ([`NSI#L532-L551`](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi#L532-L551)):
  - `currentUser`: `$LOCALAPPDATA\${PRODUCTNAME}`, which needs no admin rights (`RequestExecutionLevel user`, [`NSI#L106-L111`](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi#L106-L111)).
  - `perMachine`: `$PROGRAMFILES64\${PRODUCTNAME}`.
  - The [Tauri docs](https://v2.tauri.app/distribute/windows-installer/) say the same: by default "the app is installed in the `%LOCALAPPDATA%` folder instead of `C:/Program Files`".
- The person can choose another folder on the directory page ([`NSI#L413-L415`](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi#L413-L415)).
- Later installs and updates reuse the saved folder: `RestorePreviousInstallLocation` reads it from the registry ([`NSI#L942-L946`](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi#L942-L946)).
- The Windows release is only the NSIS `-setup.exe` (`.github/workflows/release.yml:28-31`), so there is no portable exe that could run from `Downloads`. On Windows, `current_exe()` is always the installed path.

**Updates keep the path**
- The updater gets the path from `current_exe()` ([`updater.rs#L455-L461`](https://github.com/tauri-apps/plugins-workspace/blob/updater-v2.14.1/plugins/updater/src/updater.rs#L455-L461)).
- On macOS it replaces the `.app` that contains the running binary ([`updater.rs#L1793-L1818`](https://github.com/tauri-apps/plugins-workspace/blob/updater-v2.14.1/plugins/updater/src/updater.rs#L1793-L1818)).
- On Windows it runs the NSIS installer, which restores the saved folder.
- The path changes only if `productName` changes. Keep `productName` fixed.

**Symlinks and canonicalization**
- ⚠ **Never call `std::fs::canonicalize` on the Windows path.** It returns the `\\?\` extended-length form. The std docs warn that this form "may be incompatible with other applications (if passed to the application on the command-line, or written to a file another application may read)" ([`STD/fs.rs#L3071-L3083`](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/fs.rs#L3071-L3083)).
- Do not use `tauri::utils::platform::current_exe()` for the prompt:
  - it canonicalizes ([`TU/platform/starting_binary.rs#L40-L41`](https://github.com/tauri-apps/tauri/blob/tauri-utils-v2.10.3/crates/tauri-utils/src/platform/starting_binary.rs#L40-L41)), so it also returns `\\?\` on Windows;
  - on macOS it returns an error for any path with a symlink ancestor ([`#L66-L83`](https://github.com/tauri-apps/tauri/blob/tauri-utils-v2.10.3/crates/tauri-utils/src/platform/starting_binary.rs#L66-L83)).
- Plain `std::env::current_exe()`, as `lib.rs:211` and `:223` use today, is correct.
- In `pnpm tauri dev` the path is `target/debug/haulmer-connect`. That is fine for testing.

## 2. What does `tauri-plugin-cli` actually do?

**ANSWER: Verdict: it does not solve this problem. Do not add it. Keep the hand-rolled `mcp` dispatch before `tauri::Builder`.**
- The plugin is a thin `clap` wrapper with its schema in `tauri.conf.json`.
- It parses only after you have a built Tauri `App`. By then the event loop exists, and if you parse inside `.setup()`, both webviews already exist.
- It has no API for the executable path.
- For a stdio subcommand it adds risk: a native runtime and webviews start in the same process whose stdout must stay pure JSON-RPC.

**API surface**
- `Cli::matches()` and `Cli::matches_from()`, reached through `app.cli()` ([`CLI/lib.rs#L28-L56`](https://github.com/tauri-apps/plugins-workspace/blob/cli-v2.5.0/plugins/cli/src/lib.rs#L28-L56)), plus the JS `getMatches()` command (`#L58-L61`).
- `init()` only stores the state in the plugin's `setup` hook ([`#L63-L73`](https://github.com/tauri-apps/plugins-workspace/blob/cli-v2.5.0/plugins/cli/src/lib.rs#L63-L73)).
- The `parser` module is private. Only the data types are re-exported ([`#L21`, `#L26`](https://github.com/tauri-apps/plugins-workspace/blob/cli-v2.5.0/plugins/cli/src/lib.rs#L21-L26)). So you cannot parse without an `App`.
- Nothing returns or computes the executable path.

**When parsing happens**
- Parsing is lazy. `matches()` calls clap `try_get_matches()` ([`CLI/parser.rs#L82-L131`](https://github.com/tauri-apps/plugins-workspace/blob/cli-v2.5.0/plugins/cli/src/parser.rs#L82-L131)).
- The earliest point is after `Builder::build()`:
  - `build()` creates the runtime and event loop (`R::new`, [`TA#L2504-L2511`](https://github.com/tauri-apps/tauri/blob/tauri-v2.12.6/crates/tauri/src/app.rs#L2504-L2511));
  - then it runs plugin setup (`initialize_plugins`, [`TA#L2607`](https://github.com/tauri-apps/tauri/blob/tauri-v2.12.6/crates/tauri/src/app.rs#L2607)).
- The [plugin docs](https://v2.tauri.app/plugin/cli/) call `app.cli().matches()` inside `.setup()`.
  - `.setup()` runs on `RuntimeRunEvent::Ready`, after the loop starts ([`TA#L1441-L1444`](https://github.com/tauri-apps/tauri/blob/tauri-v2.12.6/crates/tauri/src/app.rs#L1441-L1444)).
  - Before the user's closure, Tauri builds every window in the config ([`TA#L2688-L2701`](https://github.com/tauri-apps/tauri/blob/tauri-v2.12.6/crates/tauri/src/app.rs#L2688-L2701)).
  - haulmer-connect declares two windows (`tauri.conf.json:13-28`), and `create` defaults to `true` ([`TU/config.rs#L1976-L1977`](https://github.com/tauri-apps/tauri/blob/tauri-utils-v2.10.3/crates/tauri-utils/src/config.rs#L1976-L1977)). So two hidden webviews exist before an `mcp` branch could run.

**`--help` and `--version`**
- The plugin returns them as matches and prints nothing ([`CLI/parser.rs#L106-L127`](https://github.com/tauri-apps/plugins-workspace/blob/cli-v2.5.0/plugins/cli/src/parser.rs#L106-L127)).
- On Windows the docs add: "production apps are not able to write text back to the calling console by default". They point to [tauri#8305](https://github.com/tauri-apps/tauri/issues/8305#issuecomment-1826871949), whose workaround is `AttachConsole(ATTACH_PARENT_PROCESS)`.

**Stdout cleanliness**
- The MCP spec says: "The server **MUST NOT** write anything to its `stdout` that is not a valid MCP message" ([transports, 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports)).
- Today `main.rs:13-22` calls `run_mcp()` before any Tauri code, so only `mcp::serve` writes to stdout (`crates/core/src/mcp.rs:35-51`).
- **UNVERIFIED:** whether the Tauri runtime or WebView startup writes to stdout. The current dispatch makes the question irrelevant.

**`tauri-plugin-single-instance`**
- If it is added later, its plugin setup exits a second process at once:
  - macOS: `notify_singleton`, then `std::process::exit(0)` ([`SI/src/platform_impl/macos.rs#L20-L29`](https://github.com/tauri-apps/plugins-workspace/blob/cli-v2.5.0/plugins/single-instance/src/platform_impl/macos.rs#L20-L29));
  - Windows: it forwards the args, then calls `exit(0)` ([`windows.rs#L84-L105`](https://github.com/tauri-apps/plugins-workspace/blob/cli-v2.5.0/plugins/single-instance/src/platform_impl/windows.rs#L84-L105)).
  - Its README says to register it first ([`README.md#L60`](https://github.com/tauri-apps/plugins-workspace/blob/cli-v2.5.0/plugins/single-instance/README.md#L60)).
- If `mcp` went through the Builder, every MCP spawn would exit while the tray app runs. The current dispatch before the Builder avoids this.

**What the plugin would add:** a declarative schema and JS access to the args. Neither helps with one stdio subcommand. If more subcommands come later, call `clap` directly in `main.rs`, before the Builder.

## 3. Windows stdio with `windows_subsystem = "windows"`

**ANSWER: Yes, stdio works.**
- MCP clients pass pipe handles through `STARTF_USESTDHANDLES`, and a GUI-subsystem process uses them.
- No `AttachConsole` is needed, and `eprintln!` is safe.
- The `mcp` process never gets a console, so it cannot flash one.
- Two side effects, both inferred from Microsoft docs and **not observed** (no Windows machine was used):
  1. a "working in background" cursor for up to about 2 s on each spawn;
  2. ⚠ **a bigger risk:** `gws.exe` children start without `CREATE_NO_WINDOW`, so each call can open a new console window.

**Handles reach a GUI process**
- [`GetStdHandle`](https://learn.microsoft.com/en-us/windows/console/getstdhandle):
  - "/SUBSYSTEM:CONSOLE requests that the operating system fill the handles with a console session on startup, if the parent didn't already fill the standard handle table by inheritance."
  - "Graphical user interface (GUI) applications can be started without the standard handles and they will not be automatically filled."
  - With no handles, the return value is `NULL`.
- [`STARTUPINFOW`](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/ns-processthreadsapi-startupinfow): "If dwFlags specifies STARTF_USESTDHANDLES, this member is the standard output handle for the process." There is no subsystem condition.
- The clients set that flag:
  - libuv, used by Node, so by Claude Code and by Claude Desktop through the MCP SDK: `dwFlags = STARTF_USESTDHANDLES | STARTF_USESHOWWINDOW` ([`UV#L1039`](https://github.com/libuv/libuv/blob/v1.53.0/src/win/process.c#L1039));
  - Rust std, used by Codex: sets it when any stdio is set ([`STD/sys/process/windows.rs#L359-L368`](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/sys/process/windows.rs#L359-L368));
  - the MCP TS SDK spawns with `stdio: ['pipe', 'pipe', stderr ?? 'inherit']` and `windowsHide: process.platform === 'win32'` ([`SDK#L120-L128`](https://github.com/modelcontextprotocol/typescript-sdk/blob/v1.29.0/src/client/stdio.ts#L120-L128)).
- Rust std reads the handle with `GetStdHandle` ([`STD/sys/stdio/windows.rs#L68-L77`](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/sys/stdio/windows.rs#L68-L77)). Pipes take the plain `WriteFile` path, not the console path (`#L101-L113`).

**No handles (someone starts it by hand from cmd)**
- Std docs: with `windows_subsystem = "windows"` the handle "will be null. In such cases, the standard library's `Read` and `Write` will do nothing and silently succeed" ([`STD/io/stdio.rs#L224-L234`](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/io/stdio.rs#L224-L234), [`#L595-L605`](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/io/stdio.rs#L595-L605)).
- So stdin reads EOF at once, `serve` returns, and the process exits. It does not crash. `eprintln!` in `main.rs:16` is also a silent no-op there.

**`eprintln!` and `AttachConsole`**
- Claude Desktop pipes server stderr into `mcp-server-SERVERNAME.log` ([connect-local-servers](https://modelcontextprotocol.io/docs/develop/connect-local-servers)). The spec allows stderr logging.
- `AttachConsole` adds nothing for MCP. Microsoft: "When attaching to a new console, standard handles are always replaced with console handles unless STARTF_USESTDHANDLES was specified during process creation" ([`GetStdHandle`](https://learn.microsoft.com/en-us/windows/console/getstdhandle)). Do not add it to the `mcp` path.

**Busy cursor (cosmetic, inferred)**
- `STARTUPINFOW`: "If a GUI process is being started and neither STARTF_FORCEONFEEDBACK or STARTF_FORCEOFFFEEDBACK is specified, the process feedback cursor is used." The cursor stays on for two seconds unless the process makes a GUI call.
- libuv never sets either flag ([`UV#L1039`](https://github.com/libuv/libuv/blob/v1.53.0/src/win/process.c#L1039)). Rust std sets one only on request ([`STD/sys/process/windows.rs#L384-L391`](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/sys/process/windows.rs#L384-L391)).

**⚠ Console flash: the real risk is `gws.exe`, not `haulmer-connect.exe`**
- "GUI processes are not attached to a console when they are created", and "by default, a console process inherits its parent's console" ([Creation of a Console](https://learn.microsoft.com/en-us/windows/console/creation-of-a-console)). "The system creates a new console when it starts a console process."
- `SystemRunner` builds `gws` commands with no creation flags (`crates/core/src/system_runner.rs:27-40`). The parent is either the GUI app or the GUI-subsystem `mcp` process, and neither has a console. So the inference is that every `gws` call opens a new console window. **Not reproduced.**
- [`CREATE_NO_WINDOW`](https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags) (`0x08000000`): "The process is a console application that is being run without a console window."
- Fix: `CommandExt::creation_flags` ([`STD/os/windows/process.rs#L169-L175`](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/os/windows/process.rs#L169-L175)). libuv does the same for hidden children ([`UV#L1099-L1107`](https://github.com/libuv/libuv/blob/v1.53.0/src/win/process.c#L1099-L1107)).

```rust
// crates/core/src/system_runner.rs, inside to_command()
#[cfg(windows)]
{
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW: gws is a console program
}
```

**Windows check to run once** (the client health check is the real test):

```powershell
claude mcp get haulmer-connect   # spawns the server and health-checks it
'{"jsonrpc":"2.0","id":1,"method":"ping"}' | & "$env:LOCALAPPDATA\haulmer-connect\haulmer-connect.exe" mcp
# expected: {"jsonrpc":"2.0","id":1,"result":{}}
# UNVERIFIED: whether PowerShell waits for a GUI-subsystem exe when its output is captured.
```

## 4. Exact install mechanics per client

**ANSWER:**
- Terminal agents have one-line CLI commands.
- Claude Desktop needs a JSON edit plus a full restart, or an `.mcpb`.
- **The claim in `docs/spec.md:157` is true:** current OpenAI docs say the ChatGPT desktop app reads `~/.codex/config.toml`. The ChatGPT web app does not.
- claude.ai and chatgpt.com cannot run local stdio servers.

**Claude Code** ([`CC`](https://code.claude.com/docs/en/mcp), local `claude mcp add --help`, v2.1.311)

```sh
claude mcp add --scope user haulmer-connect -- "/Applications/haulmer-connect.app/Contents/MacOS/haulmer-connect" mcp
claude mcp add --scope user haulmer-connect -- "C:\Users\ana\AppData\Local\haulmer-connect\haulmer-connect.exe" mcp
claude mcp add-json --scope user haulmer-connect '{"type":"stdio","command":"/Applications/haulmer-connect.app/Contents/MacOS/haulmer-connect","args":["mcp"]}'
claude mcp get haulmer-connect   # health-checks the server
claude mcp list                  # health-checks all servers
```

- The `--` separator: "Everything after `--` is passed to the server untouched." Without it, Claude Code parses the server's flags as its own.
- Scopes:

  | Scope | Loads in | Stored in |
  |---|---|---|
  | `local` (default) | current project only | `~/.claude.json` |
  | `project` | current project, shared | `.mcp.json` |
  | `user` | all your projects | `~/.claude.json` |

  **Use `--scope user`.** With the default `local`, the server loads only in the folder where the person pasted the prompt.
- Local help text: `get` and `list` health-check approved servers. That makes them a real spawn test.
- `add-json` stdio example in the docs: `'{"type":"stdio","command":"/path/to/weather-cli","args":[...]}'`.
- **UNVERIFIED:** whether `claude mcp add` replaces or refuses an existing name. The prompt below asks the agent to check first.

**Codex CLI** (`OAI`, local `codex mcp add --help`, v0.167.2; source `rust-v0.169.1`)

```sh
codex mcp add haulmer-connect -- "/Applications/haulmer-connect.app/Contents/MacOS/haulmer-connect" mcp
codex mcp get haulmer-connect
codex mcp list
```

```toml
# ~/.codex/config.toml
[mcp_servers.haulmer-connect]
command = "/Applications/haulmer-connect.app/Contents/MacOS/haulmer-connect"
args = ["mcp"]
# Windows: a TOML literal string (single quotes) needs no backslash escaping:
# command = 'C:\Users\ana\AppData\Local\haulmer-connect\haulmer-connect.exe'
```

- Local help syntax: `codex mcp add [OPTIONS] <NAME> (--url <URL> | -- <COMMAND>...)`.
- It always writes the global file and replaces a server with the same name: `servers.insert(...)`, then `replace_mcp_servers`, then it prints "Added global MCP server" ([`mcp_cmd.rs#L446-L456`](https://github.com/openai/codex/blob/rust-v0.169.1/codex-rs/cli/src/mcp_cmd.rs#L446-L456)).
- Stdio keys from `OAI`: `command` (required), `args`, `env`, `env_vars`, `cwd`.
- Verify: `codex mcp list`, or `/mcp` in the TUI.
- Windows paths: use [TOML literal strings](https://toml.io/en/v1.0.0#string). The existing `add_to_chatgpt` escapes backslashes inside a basic string, which is also correct (`crates/core/src/mcp_clients.rs`).

**Claude Desktop** ([connect-local-servers](https://modelcontextprotocol.io/docs/develop/connect-local-servers))
- Config paths:
  - macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
  - Windows: `%APPDATA%\Claude\claude_desktop_config.json`
- Open the file from Settings → Developer → **Edit Config**.
- Shape: `{"mcpServers": {"haulmer-connect": {"command": "<exe>", "args": ["mcp"]}}}`.
- Restart: "completely quit Claude Desktop and restart it". The troubleshooting list adds that paths must be "absolute and not relative".
- Logs: `~/Library/Logs/Claude` or `%APPDATA%\Claude\logs`. `mcp-server-haulmer-connect.log` holds the server's stderr.
- **Desktop Extensions (`.mcpb`):**
  - An MCPB is "a zip archive containing a local MCP server and a `manifest.json`", for "single-click installation" ([`MCPB/README.md#L11-L18`](https://github.com/modelcontextprotocol/mcpb/blob/v2.1.2/README.md#L11-L18)).
  - Install a custom one from Settings → Advanced settings → Extension Developer → **Install Extension…** ([support article](https://support.claude.com/en/articles/10949351-getting-started-with-local-mcp-servers-on-claude-desktop)).
  - Team and Enterprise owners can allow or block extensions.
  - To wrap a native binary:
    - set `server.type: "binary"` with an `entry_point` ([`MANIFEST.md#L380-L393`](https://github.com/modelcontextprotocol/mcpb/blob/v2.1.2/MANIFEST.md#L380-L393), [`#L462-L465`](https://github.com/modelcontextprotocol/mcpb/blob/v2.1.2/MANIFEST.md#L462-L465));
    - set `mcp_config.command`. "For binaries, apps will automatically append `.exe` on Windows", or use `platform_overrides.win32` ([`#L493-L527`](https://github.com/modelcontextprotocol/mcpb/blob/v2.1.2/MANIFEST.md#L493-L527));
    - `${__dirname}` resolves to the extension folder ([`#L529-L537`](https://github.com/modelcontextprotocol/mcpb/blob/v2.1.2/MANIFEST.md#L529-L537));
    - limit to macOS and Windows with `compatibility.platforms: ["darwin","win32"]`;
    - put the binary under `server/` ([`README.md#L95-L104`](https://github.com/modelcontextprotocol/mcpb/blob/v2.1.2/README.md#L95-L104)).

**ChatGPT desktop app: claim VERIFIED**
- `OAI`: "The ChatGPT desktop app, Codex CLI, and IDE extension support MCP servers and share MCP configuration for the same Codex host", and "The ChatGPT desktop app, Codex CLI, and IDE extension share this configuration." The default file is `~/.codex/config.toml`.
- UI path in the desktop app: Settings → MCP servers → **Add server** → choose STDIO and enter the command → Save → **Restart**. Then type `/mcp` in the composer to see connected servers.
- Local observation: `/Applications/ChatGPT.app` version 26.1111.21308 has bundle id `com.openai.codex` and bundles `Contents/Resources/codex-cli/bin/codex`.
- **UNVERIFIED:** whether the Windows ChatGPT desktop app behaves the same. The docs do not split by OS.

**chatgpt.com (web)**
- `OAI`: "ChatGPT web doesn't read local Codex configuration files or expose the local Codex command menu." The web app uses remote MCP tools through plugins.
- Custom servers: "Supported MCP protocols: SSE and streaming HTTP." They are added from chatgpt.com/plugins ([custom MCP server guide](https://developers.openai.com/api/docs/guides/custom-mcp-server)). There is no stdio option.
- A private or local server needs [Secure MCP Tunnel](https://developers.openai.com/api/docs/guides/secure-mcp-tunnels). See section 6.

**claude.ai (web)**
- [Custom connectors](https://support.claude.com/en/articles/11175166-getting-started-with-custom-connectors-using-remote-mcp) are remote only: "Claude connects to your remote MCP server from Anthropic's cloud infrastructure."
- The server "must be reachable over the public internet from Anthropic's IP ranges" on port 443.
- Servers from `claude_desktop_config.json` "aren't available in claude.ai or in Cowork sessions that run in the cloud."

**Cursor and VS Code** (docs only, not tested)
- [Cursor](https://cursor.com/docs/context/mcp): `~/.cursor/mcp.json` (global) or `.cursor/mcp.json` (project), with `{"mcpServers": {"<name>": {"command", "args", "env"}}}`.
- [VS Code](https://code.visualstudio.com/docs/agents/reference/mcp-configuration): `code --add-mcp '{"name":"haulmer-connect","type":"stdio","command":"<exe>","args":["mcp"]}'` adds the server to the user profile. The format uses `"servers"`, not `"mcpServers"`. `command` "must be available on your system path or contain its full path."

**Documentation drift in this repo**
- `docs/spec.md:157` ("behind buttons in the app") and `README.md:164` ("The UI can write the configuration") are stale. Commit `9de9e8a` removed those buttons.
- The Rust commands are still registered (`src-tauri/src/lib.rs:298-299`).
- The ChatGPT part of both claims is correct.

## 5. What a pasted prompt can achieve per surface

**ANSWER:** A pasted prompt can install the server by itself only in terminal agents: Claude Code, Codex, and IDE agents. In ChatGPT desktop and Claude Desktop the human must click through settings, or the app must write the config. In the web apps nothing local can work. So the prompt must carry the absolute path, the exact command for each agent, and a manual block that a chat model can relay to the human.

| Surface | Runs a local stdio MCP? | Can a pasted prompt install it alone? | Best mechanism | Verify |
|---|---|---|---|---|
| Claude Code | Yes | Yes: it runs `claude mcp add`, and the person approves the Bash call | Prompt with the absolute path and the exact `--scope user` command | `claude mcp get haulmer-connect` shows Connected; `/mcp` in a new session |
| Codex CLI / IDE | Yes | Yes: it runs `codex mcp add`. **UNVERIFIED:** whether the sandbox asks before writing `~/.codex` | Same prompt, Codex branch | `codex mcp get haulmer-connect`; `/mcp` |
| ChatGPT desktop | Yes (shared Codex config) | Not by itself in a normal chat. **UNVERIFIED:** whether a Codex-mode chat in the app can run `codex mcp add` | Nothing more if `codex mcp add` already ran; else Settings → MCP servers → Add server (STDIO, `<exe>`, `mcp`) → Restart | `/mcp` in the composer |
| Claude Desktop | Yes | No: a chat cannot edit its own config file. **UNVERIFIED:** with a filesystem extension installed | Human: Edit Config, paste the JSON, quit and restart. Later: `.mcpb` or an opt-in writer button | Connectors list; `mcp-server-haulmer-connect.log` |
| claude.ai web | No | No | None for stdio. A remote connector needs a public HTTPS server | n/a |
| chatgpt.com | No | No | None for the pilot. Secure MCP Tunnel is possible (section 6) | n/a |
| Cursor / VS Code | Yes | Probably yes: the agent edits `~/.cursor/mcp.json` or runs `code --add-mcp` (not tested) | The same path data | MCP logs or the server list |

**Prompt shapes (choose A now)**
- **A. One prompt** with the path built in, a Claude Code command, a Codex command, and a manual JSON block for Claude Desktop and ChatGPT desktop.
  - Pros: smallest change; one tray item; works everywhere a model can act or relay steps.
  - Cons: a longer text. The model must pick its own branch, and agents know which agent they are.
- **B. A tray submenu per client** ("Claude Code", "Codex / ChatGPT", "Claude Desktop").
  - Pros: shorter text; each item can carry an exact snippet (shell, JSON or TOML).
  - Cons: four menu items, and the person must know which client they use.
- **C. Copy only a ready command** (`claude mcp add --scope user haulmer-connect -- "<exe>" mcp`).
  - Pros: no model in the loop.
  - Cons: only one client at a time; no conflict check; no help for GUI apps.

**Escaping rules for the generated text**
- Shell: double quotes. They work in POSIX shells, Git Bash and PowerShell for normal paths. They also survive apostrophes such as `C:\Users\O'Brien`. They fail only on `$` or a backtick.
- JSON: build it with `serde_json`.
- TOML: use literal strings.

## 6. Alternatives to a raw `current_exe()` path

**ANSWER: None of them beats "absolute installed path plus a translocation check" for the pilot.** An `.mcpb` is the only one worth a later spike, and only for Claude Desktop.

**A stable shim or symlink on PATH**
- Prior art, VS Code:
  - macOS: it asks for admin rights through `osascript`, then runs `ln -sf '<app>/bin/code' /usr/local/bin/code` ([`nativeHostMainService.ts#L620-L661`](https://github.com/microsoft/vscode/blob/1.147.0/src/vs/platform/native/electron-main/nativeHostMainService.ts#L620-L661), [`#L700-L711`](https://github.com/microsoft/vscode/blob/1.147.0/src/vs/platform/native/electron-main/nativeHostMainService.ts#L700-L711)). The link targets a `bin/code` *script*, not the main binary.
  - Windows: its Inno Setup installer adds `{app}\bin` to the user or system `Path` ([`code.iss#L1299-L1310`](https://github.com/microsoft/vscode/blob/1.147.0/build/win32/code.iss#L1299-L1310)).
- Tauri has [NSIS installer hooks](https://v2.tauri.app/distribute/windows-installer/) (`bundle.windows.nsis.installerHooks` with `NSIS_HOOK_POSTINSTALL` and `NSIS_HOOK_POSTUNINSTALL`, called at [`NSI#L778-L779`](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi#L778-L779)). The docs show no PATH helper, so the PATH edit and its undo would be custom NSIS code.
- Cons:
  - On Windows the installed path is already stable, so a shim adds nothing.
  - On macOS a shim does not help the one bad case: a symlink to a translocated path breaks too.
  - On macOS it needs admin rights for `/usr/local/bin`.
  - Tauri's own `current_exe()` rejects symlinked paths on macOS (section 1), and the updater uses that path. A symlink straight to the main binary therefore breaks updates if someone launches the GUI through it. `run_mcp` does not call it, so `mcp` itself is fine.
  - **UNVERIFIED:** whether GUI clients that start from Finder find a bare command name on PATH. Configs should use an absolute path in any case.
- Verdict: no.

**`.mcpb` Desktop Extension (Claude Desktop only)**
- Pros:
  - one-click install from a file;
  - no JSON editing;
  - admin allow and block controls for Team and Enterprise.
- Cons:
  - a `binary` bundle carries its own copy of the server inside the zip, so the extension's copy and the app's copy can drift apart on version;
  - it needs one bundle per OS, or both binaries plus `platform_overrides`;
  - it works only for Claude Desktop.
- `run_mcp` finds data from `HOME` and `APPDATA` (`src-tauri/src/lib.rs:535-554`), not from its own location, so a copy would still reach the app's `gws` and credentials. The MCP SDK passes `HOME` (Unix) and `APPDATA` and `USERPROFILE` (Windows) by default ([`SDK#L45-L62`](https://github.com/modelcontextprotocol/typescript-sdk/blob/v1.29.0/src/client/stdio.ts#L45-L62)).
- A "thin" bundle that the app generates, with `mcp_config.command` set to the installed exe, would avoid the copy. The open-source config builder passes `command` through unchanged ([`MCPB/src/shared/config.ts#L94-L179`](https://github.com/modelcontextprotocol/mcpb/blob/v2.1.2/src/shared/config.ts#L94-L179)). **UNVERIFIED:** whether Claude Desktop accepts a `binary` bundle whose `entry_point` is not inside the zip.
- **UNVERIFIED:** Gatekeeper behaviour for an unsigned binary that Claude Desktop extracts.
- Verdict: optional spike after the prompt fix.

**Localhost Streamable HTTP server**
- It does not reach the web apps:
  - claude.ai connects "from Anthropic's cloud infrastructure" to servers on the public internet;
  - ChatGPT web needs a remote server or a tunnel.
- Every local client that would use it already speaks stdio, and the spec says "Clients **SHOULD** support stdio whenever possible."
- Security cost: the spec requires `Origin` validation against DNS rebinding, says local servers **SHOULD** bind to `127.0.0.1`, and says they **SHOULD** authenticate every connection ([transports](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports)). An always-on port while the tray app runs exposes Google Workspace data to every local process and to rebinding pages, unless all of that is done right.
- Verdict: no.

**Secure MCP Tunnel (the only path to chatgpt.com)**
- From the [guide](https://developers.openai.com/api/docs/guides/secure-mcp-tunnels):
  - the `tunnel-client` can launch a stdio command with `--mcp-command`;
  - it is "an outbound-only connection from a host inside your network to an OpenAI-hosted MCP endpoint";
  - "Running tunnel-client … requires Tunnels Read + Use", and creating a tunnel requires "Tunnels Read + Manage";
  - requests have a 120-second default deadline and a limit of 600 requests per minute.
- Cost: company Google data goes through an OpenAI-hosted endpoint, with org-level setup.
- Verdict: a policy decision, out of scope for the pilot.

**Sidecar (`externalBin`) or a second Cargo `[[bin]]`**
- `externalBin` needs a `-$TARGET_TRIPLE` suffix ([sidecar docs](https://v2.tauri.app/develop/sidecar/)).
- Simpler: a second `[[bin]]`, for example `haulmer-connect-mcp`, built with the console subsystem and without Tauri. The CLI collects every Cargo bin ([`tauri-cli rust.rs#L928-L992`](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-cli/src/interface/rust.rs#L928-L992)). The bundler copies each one next to the main binary:
  - macOS: [`macos/app.rs#L166-L180`](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-bundler/src/bundle/macos/app.rs#L166-L180)
  - NSIS: [`nsis/mod.rs#L876-L905`](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-bundler/src/bundle/windows/nsis/mod.rs#L876-L905)
- Pros: a small binary; no busy cursor; visible `--help` in terminals.
- Cons:
  - the path problem is unchanged: it sits inside the same `.app`, so it is translocated too;
  - a console-subsystem child of a GUI client can open a console window unless the client hides it. libuv adds `CREATE_NO_WINDOW` only when no stdio is inherited ([`UV#L1099-L1107`](https://github.com/libuv/libuv/blob/v1.53.0/src/win/process.c#L1099-L1107)), and the MCP SDK inherits stderr by default.
- Verdict: not now. Revisit only if the Windows busy cursor bothers users.

## 7. Recommendation

**ANSWER: Build the prompt in Rust from `std::env::current_exe()`, with a Claude Code command, a Codex command and a manual block. Do not copy it when the path contains `/AppTranslocation/`. Copy it with a warning when the path starts with `/Volumes/`.**
- It needs no new dependency, which matters under the due-diligence rule in `docs/spec.md`.
- It needs no auto-writing of configs. The evidence does not require bringing the writer buttons back.
- Estimate: about 1 to 2 hours with tests.

**Step 1 (now): one pure function** in `crates/core/src/mcp_clients.rs` (the core crate already has `serde` and `serde_json`)

```rust
/// The text to copy, plus a warning to show when the path may not last.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallPrompt {
    pub text: String,
    pub warning: Option<String>,
}

/// The Spanish install prompt with `exe` as the MCP server command. Apple has no supported
/// API to detect translocation, so this checks the path text: it fails for a translocated
/// path, and warns for `/Volumes/` (the mounted DMG, or a working external drive).
///
/// # Example
///
/// ```
/// # use haulmer_connect_core::mcp_clients::install_prompt;
/// let installed = "/Applications/haulmer-connect.app/Contents/MacOS/haulmer-connect";
/// let prompt = install_prompt(installed.as_ref()).unwrap();
/// assert!(prompt.text.contains(&format!("-- \"{installed}\" mcp")));
/// assert!(prompt.warning.is_none());
/// let random = "/private/var/folders/x/T/AppTranslocation/AB12/d/haulmer-connect.app/Contents/MacOS/haulmer-connect";
/// assert!(install_prompt(random.as_ref()).is_err());
/// ```
pub fn install_prompt(exe: &Path) -> Result<InstallPrompt, CoreError> {
    let exe = exe.to_string_lossy();
    if exe.contains("/AppTranslocation/") {
        return Err(CoreError::UnstableExe(
            "macOS está ejecutando haulmer-connect desde una carpeta temporal. Muévela a \
             Aplicaciones, ábrela de nuevo y vuelve a copiar el prompt."
                .to_string(),
        ));
    }
    let mut warning = None;
    if exe.starts_with("/Volumes/") {
        warning = Some(
            "Si abriste haulmer-connect desde la imagen de disco (.dmg), muévela a Aplicaciones \
             y vuelve a copiar el prompt: esta ruta deja de existir al expulsar el disco."
                .to_string(),
        );
    }
    let desktop_json = serde_json::to_string_pretty(&json!({
        "mcpServers": {"haulmer-connect": {"command": exe, "args": ["mcp"]}}
    }))
    .expect("a JSON value always serializes");
    let text = format!(include_str!("install_prompt.es.txt"), exe = exe, desktop_json = desktop_json);
    Ok(InstallPrompt { text, warning })
}
```

**Step 1 (now): the command and the listener** in `src-tauri/src/lib.rs` and `src/App.tsx`

```rust
/// `mcp_clients::install_prompt` for this executable.
///
/// # Example
///
/// ```js
/// const { text, warning } = await invoke("install_prompt");
/// ```
#[tauri::command]
fn install_prompt() -> Result<InstallPrompt, CoreError> {
    // Plain current_exe(): canonicalize() would return `\\?\` paths on Windows.
    mcp_clients::install_prompt(&std::env::current_exe()?)
}
```

```ts
// src/App.tsx: replaces MCP_INSTALL_PROMPT; the tray event stays payload-free.
const unlistenCopyPrompt = listen("copy-install-prompt", async () => {
  try {
    const prompt = await invoke<{ text: string; warning: string | null }>("install_prompt");
    await navigator.clipboard.writeText(prompt.text);
    if (prompt.warning) {
      setNotice({ kind: "info", text: prompt.warning });
    } else {
      setNotice({ kind: "success", text: "Prompt copiado al portapapeles." });
    }
  } catch (error) {
    setNotice({ kind: "error", text: ERROR_TEXT(error) });
  }
});
```

- Add the `CoreError::UnstableExe(String)` variant, and register `install_prompt` in `generate_handler!`.
- `format!(include_str!(…), …)` compiles; this was checked with `rustc 1.99.1` on a throwaway file.
- The main window is hidden in tray mode, so the notice is invisible. When there is an error or a warning, also show the main window. One way: a small command that calls `show_main_window`. Another: move the call into the tray handler, show the window there, and emit the result as the event payload.

**Prompt text** (`install_prompt.es.txt`, shown with the macOS path filled in)

```text
Configura haulmer-connect como servidor MCP local (stdio) en este equipo.

Ejecutable (ruta absoluta, entregada por la app):
/Applications/haulmer-connect.app/Contents/MacOS/haulmer-connect
Argumento: mcp

1. Antes de ejecutar nada, muéstrame el comando y pídeme confirmación.
2. Si ya existe un servidor llamado "haulmer-connect", muéstrame su configuración y pregúntame antes de reemplazarlo. No modifiques otros servidores ni otros archivos.
3. Si eres Claude Code, ejecuta:
   claude mcp add --scope user haulmer-connect -- "/Applications/haulmer-connect.app/Contents/MacOS/haulmer-connect" mcp
   y verifica con: claude mcp get haulmer-connect (debe decir "Connected").
4. Si eres Codex (CLI, extensión de IDE o app de escritorio de ChatGPT), ejecuta:
   codex mcp add haulmer-connect -- "/Applications/haulmer-connect.app/Contents/MacOS/haulmer-connect" mcp
   y verifica con: codex mcp get haulmer-connect
   (la app de escritorio de ChatGPT usa la misma configuración que Codex).
5. Ajusta las comillas a tu shell si hace falta, pero no cambies la ruta.
6. Al final dime si debo abrir una sesión nueva o reiniciar la app, y cómo ver la herramienta "gws" con /mcp.

Si no puedes ejecutar comandos en este equipo, explícame estos pasos (los nombres de los menús pueden estar traducidos):
- Claude Desktop: Settings > Developer > Edit Config. Si el archivo está vacío, pega este bloque completo. Si ya tiene "mcpServers", agrega solo la entrada "haulmer-connect" dentro de él, sin borrar lo demás. Después cierra Claude Desktop por completo y ábrelo de nuevo.
{desktop_json}
- App de escritorio de ChatGPT: Settings > MCP servers > Add server. Elige STDIO. En el comando pega la ruta del Ejecutable de arriba, y en los argumentos escribe mcp. Guarda y pulsa Restart.
- claude.ai o chatgpt.com (web): haulmer-connect no funciona ahí, porque necesita un programa local. Usa Claude Code, Codex, ChatGPT de escritorio o Claude Desktop.
```

On Windows the same text carries `C:\Users\<user>\AppData\Local\haulmer-connect\haulmer-connect.exe`. `serde_json` escapes the backslashes in the JSON block.

**Step 2 (separate, Windows):** add `CREATE_NO_WINDOW` to `SystemRunner` (section 3). About 15 minutes, plus one test on a Windows machine.

**Step 3 (docs):** update `docs/spec.md:152-157` and `README.md:156-188`:
- the writer buttons are gone;
- the prompt carries the path;
- the ChatGPT desktop statement stays.

**Step 4 (optional, GUI apps):**
- a tray submenu per client (shape B);
- for Claude Desktop, either the existing `add_to_claude_desktop` behind an explicit confirm button, or an `.mcpb` spike (section 6).

**Unresolved questions**
1. Does the translocated path exist while the app is not running (section 1)?
2. On Windows, does `navigator.clipboard.writeText` succeed from the hidden main window (WebView2) when the tray item is clicked?
3. On Windows, does `gws` really flash a console today? A one-time test confirms the step 2 fix.
4. Does the ChatGPT desktop app on Windows also read `~/.codex/config.toml`?
5. Does `claude mcp add` overwrite an existing `haulmer-connect` entry, or fail?
