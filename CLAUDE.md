# CLAUDE.md - govctl

Operating guidance for AI coding agents working on govctl.

## What this is

govctl scaffolds a seven-file governance stack (`init`) and detects drift between a project's
decision log and its actual code/git history (`validate`). It dogfoods its own rules.

## Ground rules

- Decisions live in `DECISIONS.md` and are enforced by `govctl validate . --strict`. Do not
  contradict a LOCKED decision; supersede it explicitly instead.
- Keep the source ASCII-only. Non-ASCII punctuation (em-dashes, smart quotes) has caused
  encoding mismatches on this toolchain; use plain hyphens and straight quotes.
- Verify before claiming done: `cargo test`, `cargo clippy --all-targets -- -D warnings`, and
  `govctl validate . --strict` must all pass.

## Project map

- `src/main.rs` - CLI (clap).
- `src/templates.rs` - embedded governance templates (`include_str!`).
- `src/decisions.rs` - DECISIONS.md parser (comment-aware; see D002).
- `src/repo_scan.rs` - reference scanner with ignore + suppression (see D003).
- `src/commands/{init,validate}.rs` - the two subcommands.
- `templates/` - the seven scaffolded files plus the default `.govctlignore`.
- `.github/` - CI: composite action, drop-in workflow, release pipeline (see D004).

_Last reviewed: 2026-06-03_
