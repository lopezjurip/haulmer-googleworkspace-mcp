---
title: haulmer-connect — handoff
date: 2026-11-16
next: implement the pilot from docs/spec.md
---

# Handoff

The spec is complete: read `docs/spec.md` first. This file only adds what the spec does not say: state, open items and how to work with the owner.

## State

- Local repo `~/repositories/haulmer-connect` exists with `docs/` only. No code yet. No remote: the owner creates `github.com/haulmer/haulmer-connect` (public) later.
- The Google Cloud project `haulmer-connect` and the shared OAuth client **do not exist yet**. Creating them is a manual console task for a human owner.
- The spec's test seam (one Rust core API with a fake process runner and a fake HTTP client) is **proposed, not yet confirmed** by the owner. Confirm it before writing tests.
- The test machine already has `gws` 0.22.5 installed and an existing `gws` config in the default folder that **must not be touched**. Always use a custom `GOOGLE_WORKSPACE_CLI_CONFIG_DIR` on it.

## Next steps

1. Confirm the test seam with the owner.
2. Spike (throwaway, outside `main`): answer "Facts to verify during implementation" in the spec, items 1-7, using real `gws` 0.22.5 and the shared client once it exists. Update the spec with the answers.
3. Guide the owner through the Google Cloud setup (project, Internal audience, Desktop client marked for an AI agent, APIs, Chat app). Use a wizard script; never handle the client secret in chat.
4. Break the spec into tickets, then implement with TDD.

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
