---
title: haulmer-googleworkspace-mcp — handoff
date: 2026-11-16
next: implement the pilot from docs/spec.md
---

# Handoff

The spec is complete: read `docs/spec.md` first. This file only adds what the spec does not say: state, open items and how to work with the owner.

## State

- Local repo `~/repositories/haulmer-googleworkspace-mcp` exists with `docs/` only. No code yet. No remote: the owner creates `github.com/haulmer/haulmer-googleworkspace-mcp` (public) later.
- The Google Cloud project `haulmer-googleworkspace-mcp` (project ID `haulmer-connect`), the Internal Auth Platform config, the Desktop OAuth client, the 6 APIs and the Chat app **exist** (2026-11-16). Second project owner: pending.
- The test seam is **confirmed**: core API + fake process runner + fake HTTP client + real filesystem in a temp dir.
- Spike done: `docs/research/gws-spike.md`; answers merged into the spec. Dependencies approved (spec, "Third-party due diligence").
- Tickets: `docs/tickets/`. Integration branch: `pilot`.
- The test machine already has `gws` 0.22.5 installed and an existing `gws` config in the default folder that **must not be touched**. Always use a custom `GOOGLE_WORKSPACE_CLI_CONFIG_DIR` on it.

## Next steps

1. Implement the tickets in `docs/tickets/` with TDD on branch `pilot`.
2. Real pilot login on the owner's Mac with a custom config folder; confirm the live `gws auth status` output.
3. Add the second project owner.

## Working with the owner

- Reply in Spanish; write code, comments, commits and docs in English.
- The owner picks every technology. Recommend options, do not decide.
- Before you recommend or add any third-party dependency, check: deprecated or archived, last release, stars and downloads, number of dependencies, known vulnerabilities (osv.dev / RustSec / GitHub advisories). Show these facts with the recommendation.
- Prefer the smallest complete change (ponytail). The owner rejected cutting the checkbox UI from the pilot, so the UI is in scope.
- Ask one question per message, with a recommendation.

## Suggested skills

- `ponytail`: keep the implementation minimal.
- `prototype` or `spike`: for step 2.
- `wizard`: for the Google Cloud console setup in step 3.
- `to-tickets`, then `implement` (with `tdd`) or `implement-spec`: for step 4.
- `code-review` and `pr`: before merging.
