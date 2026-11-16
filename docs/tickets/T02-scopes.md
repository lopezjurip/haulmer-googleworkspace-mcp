---
id: T02
title: Scope catalogue and selection mapping
status: open
blocked_by: [T01]
---

# T02 — Scope catalogue and selection mapping

Spec: `docs/spec.md`. Research: `docs/research/gws-spike.md`. Build with TDD (`tdd` skill) against the core seam: fake `ProcessRunner`, fake `HttpClient`, real filesystem in a temp dir. Test external behaviour only. Code, comments and commits in English; JSDoc / rustdoc with a usage example on every function.

## What to build
Module `scopes`: the selection → scopes mapping from the spec table, and the reverse (granted scopes → `Selection`).

- Defaults: Read for Gmail, Calendar, Chat, Meet; Drive and all Write levels off.
- Write implies Read (normalising a selection adds Read).
- Output: full scope URLs (`https://www.googleapis.com/auth/...`), comma-joined form for `gws auth login --scopes`.
- Reverse: a level counts as granted only if all its scopes are granted; ignore `openid`, `userinfo.email`, `userinfo.profile`, `email`, `profile`.
- Plain-language descriptions per service and level for the UI (e.g. "Write: send and modify email"), exposed through the core so the UI holds no scope logic.

## Acceptance criteria
- [ ] Default selection matches the spec.
- [ ] Every row of the spec table maps to exactly those scopes.
- [ ] Write implies Read; reverse mapping round-trips; extra identity scopes are ignored.
