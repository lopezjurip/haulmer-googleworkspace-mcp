---
id: T06
title: revoke_all(): revoke at Google and clear local credentials
status: done
blocked_by: [T03, T04]
---

# T06 — revoke_all(): revoke at Google and clear local credentials

Spec: `docs/spec.md`. Research: `docs/research/gws-spike.md`. Build with TDD (`tdd` skill) against the core seam: fake `ProcessRunner`, fake `HttpClient`, real filesystem in a temp dir. Test external behaviour only. Code, comments and commits in English; JSDoc / rustdoc with a usage example on every function.

## What to build
Module `revoke`: `Core::revoke_all()`, also used by `save` (T07).

- `gws auth export --unmasked` → `refresh_token`; POST `token=<refresh_token>` to `https://oauth2.googleapis.com/revoke`; then `gws auth logout`; remove the ownership marker.
- Google unreachable or non-200 → still run logout and clear local state, then return an error that says local credentials were cleared but Google revocation failed.
- No credentials → logout only, no HTTP call, success.

## Acceptance criteria
- [ ] Happy path: export, revoke POST with the token, logout, in this order.
- [ ] Revoke fails → logout still runs, error reported.
- [ ] Custom folder env on every call.
