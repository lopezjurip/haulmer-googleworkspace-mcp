---
id: T09
title: Tauri wiring and the one-screen UI
status: done
blocked_by: [T01]
---

# T09 — Tauri wiring and the one-screen UI

Spec: `docs/spec.md`. Research: `docs/research/gws-spike.md`. Build with TDD (`tdd` skill) against the core seam: fake `ProcessRunner`, fake `HttpClient`, real filesystem in a temp dir. Test external behaviour only. Code, comments and commits in English; JSDoc / rustdoc with a usage example on every function.

## What to build
The UI and the thin Tauri layer. No tests needed (spec: Testing Decisions).

- One screen: connected account (or "Not connected"); one row per service with Read and Write checkboxes and the plain-language descriptions from the core; ticking Write ticks Read.
- **Save**; a small subtle note under it: removing a service needs a new Google sign-in. While login runs: "Waiting for Google sign-in…" + **Cancel**.
- Replace dialog on `NeedsReplaceConfirmation`: "You already have gws connected with another app. Replace?" Yes → save with confirm; No → nothing.
- **Revoke all** (with confirm), **Install in my agents**, link to `https://myaccount.google.com/connections` (Google third-party apps page).
- **Advanced** (collapsed): config folder (folder picker via tauri-plugin-dialog, reset to default) and Key storage (Auto / Keyring / File).
- Success and error messages after each action.
- Tauri: `on_auth_url` opens the URL with tauri-plugin-opener; long-running core calls run off the main thread.
- Build against the T01 command contract; the commands work end to end once T05–T08 are merged.

## Acceptance criteria
- [ ] `pnpm build` and `cargo check` pass; `pnpm tauri dev` shows the screen.
- [ ] All controls above exist and call the matching command.
