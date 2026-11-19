---
id: T04
title: Settings and the environment for every gws call
status: done
blocked_by: [T01]
---

# T04 — Settings and the environment for every gws call

Spec: `docs/spec.md`. Research: `docs/research/gws-spike.md`. Build with TDD (`tdd` skill) against the core seam: fake `ProcessRunner`, fake `HttpClient`, real filesystem in a temp dir. Test external behaviour only. Code, comments and commits in English; JSDoc / rustdoc with a usage example on every function.

## What to build
Modules `settings` and `gws_env`.

- `settings`: persisted in `<app_data>/settings.json`: `config_dir` (None = gws default `<home>/.config/gws`) and `key_storage` (`auto` default). `set_config_dir` and `set_key_storage` write it; values survive a new `Core`.
- Resolved key storage: `auto` → `keyring` for the default folder, `file` for a custom folder.
- `gws_env`: one builder used by every gws call:
  - `GOOGLE_WORKSPACE_CLI_CONFIG_DIR` only when the folder is custom;
  - `GOOGLE_WORKSPACE_CLI_KEYRING_BACKEND` = resolved backend;
  - `GOOGLE_WORKSPACE_CLI_CLIENT_ID` / `_CLIENT_SECRET` from `oauth_client` (login only is enough, but passing always is fine);
  - `NO_COLOR=1`; remove `GOOGLE_APPLICATION_CREDENTIALS` and `GOOGLE_WORKSPACE_CLI_CREDENTIALS_FILE`, `GOOGLE_WORKSPACE_CLI_TOKEN` from the child env;
  - cwd = an app-controlled folder `<app_data>/run` (gws loads `.env` from its cwd).
- Helper `credentials_exist(dir)`: `credentials.enc` or `credentials.json` exists.
- Ownership marker: after a successful login the app writes `<config_dir>/haulmer-googleworkspace-mcp.json`; "foreign credentials" = credentials exist and no marker.

## Acceptance criteria
- [x] Custom folder → every built env has CONFIG_DIR and KEYRING_BACKEND=file (auto).
- [x] Default folder → no CONFIG_DIR, KEYRING_BACKEND=keyring (auto); explicit `file` / `keyring` override works.
- [x] Settings persist across `Core` instances.
- [x] Foreign-credential detection: creds without marker = true; with marker = false; none = false.
