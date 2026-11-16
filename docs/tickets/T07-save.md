---
id: T07
title: save(): connect, add and remove services
status: done
blocked_by: [T02, T05, T06]
---

# T07 — save(): connect, add and remove services

Spec: `docs/spec.md`. Research: `docs/research/gws-spike.md`. Build with TDD (`tdd` skill) against the core seam: fake `ProcessRunner`, fake `HttpClient`, real filesystem in a temp dir. Test external behaviour only. Code, comments and commits in English; JSDoc / rustdoc with a usage example on every function.

## What to build
Module `save`: `Core::save(selection, confirm_replace, on_auth_url)` and `Core::cancel_login()`.

- Ensure the gws binary (T03, may download).
- Foreign credentials in the active folder and `confirm_replace == false` → return `NeedsReplaceConfirmation`, run nothing that changes files.
- Normalise the selection (Write implies Read). Compare with currently granted levels (T05).
- Any granted level removed → `revoke_all` logic first (T06), then login with the reduced set. Only additions → login once, no revoke.
- Empty selection → same as revoke all.
- Login: spawn `gws auth login --scopes <full URLs>` with the T04 env and stdin closed; read stdout + stderr lines; on the first `https://accounts.google.com/` URL call `on_auth_url(url)` (the Tauri layer opens the browser); wait for exit. Exit 0 → write the ownership marker and return `Connected(status())`. Non-zero → error with gws `error.message`. `cancel_login()` kills the running login.

## Acceptance criteria
- [x] Add-only save: one login with the full new scope set, no revoke call.
- [x] Remove a level: revoke + logout before login with the reduced set.
- [x] Foreign credentials + no confirmation → `NeedsReplaceConfirmation` and the runner only saw read-only calls.
- [x] Auth URL forwarded from stderr and from stdout; cancel kills the process and returns a cancelled error.
- [x] Custom folder env on the login call.
