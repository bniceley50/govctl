# Changelog

## 0.2.0 - 2026-06-03

Drift detection - the centerpiece.

- `validate` now detects:
  - orphaned `D###` references in source files and git commit messages
  - broken supersede chains (SUPERSEDED entries with missing/absent successors)
  - `honors_decisions` that point at non-LOCKED or undefined decisions
  - dangling LOCKED decisions (warning)
- Comment-aware DECISIONS.md parser (template instruction blocks no longer parsed as decisions).
- `.govctlignore` (glob-lite) plus inline `govctl:ignore` / `ignore-start` / `ignore-end`.
- `--strict` flag: warnings become failures (for CI).
- CI integration: reusable composite action, drop-in workflow, cross-platform release pipeline.

## 0.1.0

- `init`: scaffold the seven-file governance stack plus `.govctl/manifest.toml`.
- `validate`: presence checks and basic sprint/decision consistency.
