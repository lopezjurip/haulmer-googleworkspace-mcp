---
id: T10
title: CI builds and GitHub Releases
status: done
blocked_by: [T01]
---

# T10 — CI builds and GitHub Releases

Spec: `docs/spec.md`. Research: `docs/research/gws-spike.md`. Build with TDD (`tdd` skill) against the core seam: fake `ProcessRunner`, fake `HttpClient`, real filesystem in a temp dir. Test external behaviour only. Code, comments and commits in English; JSDoc / rustdoc with a usage example on every function.

## What to build
GitHub Actions in `.github/workflows/`.

- On pull request and push to main: `cargo test --workspace`, `pnpm build`.
- On tag `v*`: build Windows x64 NSIS `.exe`, macOS `.dmg` for aarch64 and x86_64 (tauri-action or plain `pnpm tauri build`), generate SHA-256 files, publish to a GitHub Release. Not code-signed (spec).
- Pin third-party actions to a commit SHA.

## Acceptance criteria
- [ ] Workflows are valid YAML and `actionlint` passes if available.
- [ ] README documents how to cut a release.
