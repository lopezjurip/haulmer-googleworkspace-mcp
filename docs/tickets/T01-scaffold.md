---
id: T01
title: Scaffold the app and the core seam
status: done
blocked_by: []
---

# T01 — Scaffold the app and the core seam

Spec: `docs/spec.md`. Research: `docs/research/gws-spike.md`. Build with TDD (`tdd` skill) against the core seam: fake `ProcessRunner`, fake `HttpClient`, real filesystem in a temp dir. Test external behaviour only. Code, comments and commits in English; JSDoc / rustdoc with a usage example on every function.

## What to build
The empty, compiling project that every other ticket fills in.

- Cargo workspace at the repo root: `crates/core` (lib `haulmer_googleworkspace_mcp_core`) and `src-tauri` (Tauri v2 app, depends on core). UI at the repo root: Vite + React 19 + TypeScript + Tailwind CSS v4, pnpm.
- Only the dependencies approved in the spec ("Third-party due diligence"). Add each one only in the ticket that uses it; this ticket needs serde, serde_json, thiserror, tauri, and tempfile (dev).
- `crates/core` public contract (other tickets implement behind it, do not rename):
  - `Service { Gmail, Calendar, Chat, Meet, Drive }`, `Level { Read, Write }`, `Selection` (set of granted service levels).
  - `KeyStorage { Auto, Keyring, File }`.
  - `Paths { home, app_data }` injected at construction (tests pass temp dirs).
  - Ports: `ProcessRunner` (run to completion with program, args, env, cwd → exit code, stdout, stderr; and spawn a long-running process that yields output lines from stdout and stderr and can be killed) and `HttpClient` (GET bytes, POST form). Real implementations live in core behind these traits (std::process; `ureq` is added by T03).
  - `Core::new(paths, runner, http)` with methods `status()`, `save(selection, confirm_replace, on_auth_url)`, `cancel_login()`, `revoke_all()`, `install_skills()`, `set_config_dir(Option<PathBuf>)`, `set_key_storage(KeyStorage)`. Return types: `Status`, `SaveOutcome { Connected(Status) | NeedsReplaceConfirmation }`, `Result<_, CoreError>`.
  - One module per ticket, each a stub returning a `todo!()` or `CoreError::NotImplemented`: `scopes`, `gws_binary`, `settings`, `gws_env`, `status`, `save`, `revoke`, `skills`.
  - `oauth_client` module with the public client ID and secret constants from `docs/research/` notes / spec (public client, RFC 8252 §8.5). The implementing agent gets the values from the owner-provided file path in its context.
  - Test support: `FakeRunner` (records program, args, env, cwd; canned replies per command) and `FakeHttp` (canned responses per URL; records requests), usable by all tickets.
- `src-tauri`: registers Tauri commands that call the core methods 1:1 (no logic). Window only, no local HTTP server.
- `README.md`: what the app is, how to run `pnpm tauri dev`, how to run tests.
- `.gitignore` for Rust, Node, Tauri.

## Acceptance criteria
- [x] `cargo test --workspace` passes (one smoke test that builds `Core` with fakes and temp dirs).
- [x] `pnpm install && pnpm build` builds the UI; `cargo check -p src-tauri` (or the package name) passes.
- [x] The public contract above exists in `crates/core/src/lib.rs`.
