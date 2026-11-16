---
id: T05
title: status(): what is connected
status: open
blocked_by: [T02, T03, T04]
---

# T05 — status(): what is connected

Spec: `docs/spec.md`. Research: `docs/research/gws-spike.md`. Build with TDD (`tdd` skill) against the core seam: fake `ProcessRunner`, fake `HttpClient`, real filesystem in a temp dir. Test external behaviour only. Code, comments and commits in English; JSDoc / rustdoc with a usage example on every function.

## What to build
Module `status`: `Core::status()`.

- Resolve the gws binary (T03) without downloading: if not available, report `gws: missing` (the download happens on Save).
- Run `gws auth status` with the T04 env; parse JSON: `user` → account, `scopes` → granted `Selection` (T02), `token_valid`.
- No credentials → not connected, empty selection.
- Report active config folder, key storage (setting and resolved) and `foreign_credentials`.

## Acceptance criteria
- [ ] Connected account with granted scopes → account + selection levels.
- [ ] No credentials → not connected; gws missing → reported without HTTP calls.
- [ ] Custom folder → the status call carries CONFIG_DIR + KEYRING_BACKEND.
