---
id: T08
title: install_skills(): install gws skills into agents
status: open
blocked_by: [T02, T04]
---

# T08 — install_skills(): install gws skills into agents

Spec: `docs/spec.md`. Research: `docs/research/gws-spike.md`. Build with TDD (`tdd` skill) against the core seam: fake `ProcessRunner`, fake `HttpClient`, real filesystem in a temp dir. Test external behaviour only. Code, comments and commits in English; JSDoc / rustdoc with a usage example on every function.

## What to build
Module `skills`: `Core::install_skills()`.

- GET `https://github.com/googleworkspace/cli/archive/refs/tags/v0.22.5.tar.gz`, extract only `skills/` entries (reject paths with `..` or absolute paths).
- Install `gws-shared` plus the skills for the services granted now (from status / settings): gmail → `gws-gmail*`, calendar → `gws-calendar*`, chat → `gws-chat*`, meet → `gws-meet`, drive → `gws-drive*`. Nothing else.
- Targets: `<home>/.claude/skills/` and `<home>/.agents/skills/`, when the parent folder (`.claude` / `.agents`) exists. Overwrite the same skill folders; remove previously installed `gws-*` service skills that are no longer granted.
- Non-default config folder or key storage → append an "Environment" section to the installed `gws-shared/SKILL.md` with the exact `export GOOGLE_WORKSPACE_CLI_CONFIG_DIR=...` and `export GOOGLE_WORKSPACE_CLI_KEYRING_BACKEND=...` lines.
- Return the list of installed skill folders per target.

## Acceptance criteria
- [ ] Only shared + enabled services' skills are written, in both targets that exist.
- [ ] Custom folder → Environment section present with the folder and backend; default → absent.
- [ ] Un-granted service skills from a previous install are removed.
