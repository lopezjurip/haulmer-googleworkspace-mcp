# Tickets — haulmer-connect pilot

Local tracker. A ticket is closed by setting `status: done` in its front matter and merging its branch into the integration branch `pilot`.

| Ticket | Blocked by |
|---|---|
| [T01-scaffold](T01-scaffold.md) — Scaffold the app and the core seam | — |
| [T02-scopes](T02-scopes.md) — Scope catalogue and selection mapping | T01 |
| [T03-gws-binary](T03-gws-binary.md) — Find or download the pinned gws binary | T01 |
| [T04-settings-env](T04-settings-env.md) — Settings and the environment for every gws call | T01 |
| [T05-status](T05-status.md) — status(): what is connected | T02, T03, T04 |
| [T06-revoke](T06-revoke.md) — revoke_all(): revoke at Google and clear local credentials | T03, T04 |
| [T07-save](T07-save.md) — save(): connect, add and remove services | T02, T05, T06 |
| [T08-skills](T08-skills.md) — install_skills(): install gws skills into agents | T02, T04 |
| [T09-ui](T09-ui.md) — Tauri wiring and the one-screen UI | T01 |
| [T10-release-ci](T10-release-ci.md) — CI builds and GitHub Releases | T01 |
| [T11-review-debt](T11-review-debt.md) — Review debt deferred from the pilot code review | — |
