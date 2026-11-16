---
id: T03
title: Find or download the pinned gws binary
status: open
blocked_by: [T01]
---

# T03 — Find or download the pinned gws binary

Spec: `docs/spec.md`. Research: `docs/research/gws-spike.md`. Build with TDD (`tdd` skill) against the core seam: fake `ProcessRunner`, fake `HttpClient`, real filesystem in a temp dir. Test external behaviour only. Code, comments and commits in English; JSDoc / rustdoc with a usage example on every function.

## What to build
Module `gws_binary`: resolve the `gws` binary to run, pinned to v0.22.5.

- If `gws` on PATH reports `gws 0.22.5` (`gws --version`), use it.
- Else, if `<app_data>/bin/gws[.exe]` exists and reports 0.22.5, use it.
- Else download the platform asset and its `.sha256` from the v0.22.5 GitHub release (names in the spec), verify SHA-256 (trim CRLF on the Windows file), extract the binary from the archive root into `<app_data>/bin/`, set the executable bit on Unix.
- Checksum mismatch or download error: return a clear error, write nothing executable, never run the binary.
- Add the real `HttpClient` with `ureq` in this ticket. Dependencies: ureq, sha2, flate2, tar, zip.

## Acceptance criteria
- [ ] Pinned binary on PATH → no HTTP calls.
- [ ] Missing binary → GET asset + checksum, binary written to app data, then used.
- [ ] Checksum mismatch → error, no file at the binary path, runner never called with it.
- [ ] Wrong version on PATH → download path is taken.
