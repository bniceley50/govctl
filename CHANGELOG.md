# Changelog

## 0.3.4 - 2026-06-04

- `validate --format json`: stable machine-readable report for CI, bots, and agents (D008).
  Human output is unchanged and remains the default. JSON goes to stdout only; exit codes are
  identical across formats. Findings carry stable `code` / `suggestedFixKind` enums and nullable
  `decisionId` / `source` / `line`; top level includes `schemaVersion`, `ok`, `strict`, `summary`,
  and `exitReason` (PASSED | ERRORS | STRICT_WARNINGS).

## 0.3.3 - 2026-06-03

- Published to crates.io as `decision-drift` (`cargo install decision-drift`). The package name
  differs from the binary because `govctl` was already taken on crates.io; the installed command
  is still `govctl`. No functional changes.

## 0.3.2 - 2026-06-03

- Ignore dependency lockfiles by default (`*.lock`, `pnpm-lock.yaml`, `package-lock.json`,
  `npm-shrinkwrap.json`). Found validating a Next.js repo: a hash fragment in `pnpm-lock.yaml`
  was reported as an orphaned reference. Lockfiles are now skipped even without a project
  `.govctlignore`.

## 0.3.1 - 2026-06-03

- Precision fix (found validating a real Unity repo): a token now only counts as a decision
  reference if it is zero-padded to the decision-ID width (e.g. `D001`). Analytics-style notation
  like `D7` / `D30` (day-N retention) no longer masquerades as a reference or a false orphan.

## 0.3.0 - 2026-06-03

- `init --merge`: adopt govctl on an existing project by adding only the missing governance
  files; existing files are never touched. Mutually exclusive with `--force`; honors `--dry-run`.

## 0.2.1 - 2026-06-03

Parser robustness, found by testing against real decision logs.

- Recognize decision headings at any markdown level (`## D001` as well as `### D001`). Real-world
  logs use `##`; the parser previously matched only `###` and found zero decisions.
- Fix UTF-8 corruption: `strip_html_comments` no longer casts bytes to `char`, so em-dashes and
  other multibyte characters in titles survive intact.
- Release pipeline: race-free asset uploads and a resilient aarch64 cross-linker install.

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
