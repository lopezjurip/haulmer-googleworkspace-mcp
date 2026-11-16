---
id: T11
title: Review debt deferred from the pilot code review
status: open
blocked_by: []
---

# T11 — Review debt deferred from the pilot code review

Findings the owner deferred (token budget) when the review fixes landed on `review-fixes`.
Same rules as the other tickets: TDD against the core seam, English, rustdoc/JSDoc with a
usage example on every function.

## Deferred findings

- **S1** — add usage examples (```` ```ignore ```` for private fns) to every function still lacking one in `crates/core/src` (mostly `gws_binary`, `settings`, `skills` extraction helpers, `revoke`, `system_runner`, `system_http`).
- **S2** — remove the stale `#[allow(dead_code)]` + "used by T0x" comments on `Core` fields in `crates/core/src/lib.rs`.
- **S7** — make errors typed for the UI: serialise `CoreError` as `{ kind, message }` (kind = variant name) and have `src/core.ts` / `App.tsx` check `kind === "Cancelled"` instead of matching the message text.
- **S9** — document why `Core::install_skills_for` is public (testable entry point taking an explicit selection).
- **P6** — Environment section of `gws-shared/SKILL.md`: mention the custom folder only when it is custom, and key storage only when the backend is not the default keyring.
- **P7** — `docs/spec.md` Config folder: add one line stating the replace check applies to any folder, default or custom.
- **Spec** — update `docs/spec.md` for the contract changes made in `review-fixes`: `revoke_all(confirm_foreign)` + `RevokeOutcome`, `AccessRemoved` error, skills pinned by commit `705fb0ec…` + `SKILLS_TARBALL_SHA256`, manifest-owned skills (`installed-skills.json`, `skipped`), extra gws lookup paths.

## Already done on `review-fixes` before the scope cut (no action needed)

S4/S8 (`KeyStorage::backend`, `Service::slug`), S5 (`gws_env::gws_error_message`), S6 (`ports::download`), P2 (`--version` probe uses `gws_command`).
